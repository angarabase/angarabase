
# AngaraBase — Design Decisions & Unique Properties

This document explains what makes AngaraBase architecturally different from PostgreSQL,
TiDB, and the Postgres + ClickHouse stack. Every property described here is either
delivered in the current release or explicitly marked as roadmap.

---

## 1. Five storage engine types in one instance — mix per table, join across

Every table in AngaraBase is backed by a **`TableEngine`** trait implementation.
Tables with different engines can coexist in the same database and be joined in
a single query. The storage manager routes DML to the correct engine transparently.

```sql
-- Row store: durable OLTP
CREATE TABLE orders (
    id      BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    amount  NUMERIC(12,2)
) ENGINE = heap;

-- Ephemeral hot cache: fastest possible reads/writes, no durability
CREATE TABLE session_cache (
    session_id TEXT PRIMARY KEY,
    payload    BYTEA
) ENGINE = memory DURABILITY = none;

-- WAL-backed in-memory: survives restart without snapshot overhead
CREATE TABLE rate_limits (
    key   TEXT PRIMARY KEY,
    count BIGINT,
    ts    TIMESTAMPTZ
) ENGINE = memory DURABILITY = logged;

-- Columnar analytics: SIMD-accelerated scans, HTAP workloads
CREATE TABLE events_col (
    event_id  BIGSERIAL,
    user_id   BIGINT,
    action    TEXT,
    created   TIMESTAMPTZ
) ENGINE = column;

-- Single query across all engines simultaneously
SELECT o.user_id, COUNT(e.event_id), s.payload
FROM   orders o
JOIN   events_col e  ON e.user_id = o.user_id
JOIN   session_cache s ON s.session_id = o.user_id::text
WHERE  o.amount > 100
GROUP BY o.user_id, s.payload;
```

| Engine | Durability tier | MVCC | Best for |
|---|---|---|---|
| `heap` (row store, default) | Full WAL + fsync | UNDO-log | OLTP, systems of record |
| `memory DURABILITY = none` | None (lost on restart) | In-memory snapshot | Ephemeral caches, temp aggregates |
| `memory DURABILITY = logged` | WAL only (survives restart) | In-memory snapshot | Hot working sets, rate limiters |
| `memory DURABILITY = snapshotted` | WAL + periodic checkpoint | In-memory snapshot | Low-latency + crash recovery |
| `column` (AngaraColumn) | Full WAL + columnar format | Shared MVCC contour | HTAP analytics, bulk scans |

The engine is declared at `CREATE TABLE` time and is immutable (stored in catalog).
All engines expose the same SQL interface — there is no separate "columnar SQL dialect."

Reference: [`contracts/table_engine.rs`](../contracts/table_engine.rs)

---

## 2. Query-level service levels — `SET service_level`

**Delivered today.** Each session or query can declare its service level, which maps
directly to I/O scheduling priority:

```sql
-- Latency-critical path (billing, authentication)
SET service_level = 'critical';

-- Normal interactive queries
SET service_level = 'interactive';   -- default

-- Background analytics, batch jobs
SET service_level = 'background';
```

| Level | I/O priority | Intended workload |
|---|---|---|
| `critical` | High | Billing, auth, real-time user-facing queries |
| `interactive` | High | Normal application queries (default) |
| `background` | Low | Analytical scans, batch exports, maintenance |

Setting `critical` requires the `SET_CRITICAL_QOS` privilege — it cannot be abused
by unprivileged sessions to jump the priority queue. Privilege denial returns
`SQLSTATE 42601` (fail-closed, information-hiding).

**Coming in v0.7: named workload classes with resource quotas.** Beyond I/O priority,
each workload class will carry explicit CPU budget, memory cap, and UNDO quota.
When an analytical workload class hits its budget, it receives a `SQLSTATE` error
and rolls back — the OLTP path continues unaffected.

```sql
-- v0.7 planned syntax
SET angara.workload_class = 'analytics_tier';
-- or per-statement via hint
SELECT /*+ workload_class(analytics_tier) */ ...
```

---

## 3. Fail-closed resource contracts — errors before incidents

**Delivered today.** Every resource boundary is declared in source code, observable
as a Prometheus metric, and enforced by returning an explicit `SQLSTATE` *before*
the incident — not after silent degradation.

There are eight named boundaries:

| Boundary | SQLSTATE | When breached |
|---|---|---|
| UNDO store disk budget | `53100` | Reject DML; GC cycle; wait or error |
| Buffer pool memory | `53200` | Evict pages; OOM guard rejects |
| Concurrent query admission | `53300` | Reject immediately; no queuing |
| Connection limit | `53300` | Reject TCP connection |
| Per-transaction write set | `54023` | Reject DML; transaction must rollback |
| AngaraMemory row capacity | `53000` | Reject INSERT; no silent drop |
| Statement timeout | `57014` | Cancel statement; transaction rolls back |
| Snapshot age (stale txn) | `40001` | Force-close snapshot |

The critical property: **an analytical workload hitting its limit does not degrade
the OLTP path.** Each workload class has its own contract scope.

Reference: [`contracts/admission_control.rs`](../contracts/admission_control.rs)

---

## 4. UNDO-log MVCC — no VACUUM, no heap bloat

PostgreSQL uses heap-based MVCC: dead row versions accumulate in the heap and
VACUUM must periodically reclaim space. VACUUM competes with live queries, causes
heap bloat, and produces unpredictable maintenance windows at scale.

AngaraBase uses **UNDO-log MVCC** (Oracle / InnoDB model):
- The heap holds **only current (latest committed) row versions**
- Historical versions live in a **separate, bounded UNDO log**
- VACUUM does not exist. No autovacuum storms. No heap bloat. No maintenance windows.
- Snapshot visibility is deterministic: a read view is a function of its start LSN,
  not of whether any cleanup has run

**Writers never block readers.** A long-running `SELECT` does not prevent `INSERT`/`UPDATE`/`DELETE`,
and vice versa.

The UNDO log itself is a bounded resource (contract `undo_max_size_mb` / `SQLSTATE 53100`).
Long-running transactions that produce excessive UNDO volume are rejected before they
exhaust the budget — the system never silently runs out of UNDO space.

---

## 5. Tablespaces — per-table filesystem placement

**Delivered today.** Tables, indexes, and whole databases can be assigned to
**named tablespaces** that map to specific filesystem locations. This enables:

- **Hot/cold tiering** — place recent partitions on NVMe, historical data on HDD or
  network storage
- **I/O isolation** — billing tables on a dedicated SSD, analytics on a separate
  volume
- **Compliance** — certain data stored on encrypted volumes or specific mount points

```sql
-- Create a tablespace on a fast NVMe mount
CREATE TABLESPACE ts_nvme LOCATION '/mnt/nvme/angara_data';

-- Create a tablespace on archive storage
CREATE TABLESPACE ts_archive LOCATION '/mnt/archive/angara_data';

-- Assign tables at creation time
CREATE TABLE orders (...) TABLESPACE ts_nvme;
CREATE TABLE orders_2023 (...) TABLESPACE ts_archive;

-- Move a table to a different tablespace (online)
ALTER TABLE orders SET TABLESPACE ts_nvme;
```

The tablespace location is stored in the system catalog. On startup, AngaraBase
verifies that all declared tablespace locations are reachable — if a tablespace
directory is missing, affected databases are marked unavailable and reported as
`SQLSTATE` errors. No silent data loss.

---

## 6. Backup v2 — streaming archive with integrity verification

**Delivered today (Phase 1a).** AngaraBase ships with a built-in backup subsystem
(`angarabase-admin`) that supports:

- **Online backup** — taken without stopping the server, while writes continue
- **Columnar-aware backup** — the columnar engine (AngaraColumn) has a dedicated
  backup path that understands its internal format
- **Immutable archive** — backup artifacts are write-once; the archive layer rejects
  overwriting an existing artifact (fail-closed: `backup_archive_reject_total` counter)
- **Integrity verification** — each archive entry includes a checksum; `inspect` /
  `verify` commands confirm integrity before restore
- **Streaming restore** — restore from archive to a new instance without copying
  the full backup to local disk first

**Coming: PITR (Point-in-Time Recovery)** via continuous WAL archiving. ARIES
crash recovery already provides single-node PITR within the WAL retention window;
v0.7 extends this to long-term archive with configurable retention.

```bash
# Take an online backup
angara-admin backup create --output /mnt/backup/$(date +%Y%m%d)

# Verify archive integrity
angara-admin backup verify --archive /mnt/backup/20260601

# Restore to a new instance
angara-admin backup restore --archive /mnt/backup/20260601 --data-dir /var/lib/angara-restore
```

---

## 7. Table partitioning — declarative, with per-partition engine selection *(v0.8)*

**Coming in v0.8.** Declarative `PARTITION BY RANGE / LIST / HASH` with:

- **Automatic partition pruning** — the query planner skips partitions outside
  the filter range at planning time
- **Per-partition storage engine** — recent partitions in row store (fast writes),
  historical partitions in AngaraColumn (fast analytical scans). Automatic tiering
  within a single logical table.
- **Online partition management** — `ATTACH PARTITION` / `DETACH PARTITION` without
  locking the parent table

```sql
CREATE TABLE events (
    event_id BIGSERIAL,
    user_id  BIGINT,
    action   TEXT,
    created  TIMESTAMPTZ NOT NULL
) PARTITION BY RANGE (created);

-- Recent partition on row store (NVMe, fast ingest)
CREATE TABLE events_2026_06
    PARTITION OF events
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01')
    ENGINE = heap TABLESPACE ts_nvme;

-- Historical partition in columnar engine (SSD, fast analytics)
CREATE TABLE events_2026_01
    PARTITION OF events
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01')
    ENGINE = column TABLESPACE ts_archive;
```

---

## 8. AngaraStream — built-in event bus, no external broker

**Level 0 — LISTEN/NOTIFY (delivered today):**
Transactional pub/sub with a per-channel sharded dispatcher. Notifications are
deferred inside a transaction and dispatched on `COMMIT`; discarded on `ROLLBACK`.
At-most-once delivery guarantee. No external message broker required.

```sql
-- Publisher
NOTIFY orders_channel, '{"order_id": 42, "status": "paid"}';

-- Subscriber (any connected session)
LISTEN orders_channel;
```

**Level 1 — WAL-based CDC with delivery guarantees (v0.7):**
Logical decoding of the WAL with per-row CDC events. At-least-once delivery.
Connectors to Kafka and NATS. Replaces Debezium or a separate CDC pipeline.

---

## 9. ARIES crash recovery — deterministic, evidence-based

The recovery path follows the **ARIES** protocol (Analysis → Redo → Undo, with
Compensation Log Records):

| Recovery phase | What happens |
|---|---|
| **Analysis** | Scan WAL from last checkpoint; build dirty page table and active txn set |
| **Redo** | Replay all logged operations from Analysis start LSN to WAL end |
| **Undo** | Roll back all transactions that were active at crash time using CLR |

Key properties:
- Recovery time is **proportional to WAL since last checkpoint**, not data size
- Torn writes are detected via **CRC32C page checksums** before redo
- The same recovery contour covers both heap and columnar storage
- No manual intervention required after crash — recovery is automatic on next start

---

## 10. Linux-native observability — three layers, zero extra agents

**Layer 1 — Prometheus metrics:** every resource boundary, every engine, every
workload class has a gauge or counter. The endpoint is always active; there is no
opt-in toggle.

**Layer 2 — USDT probes:** User Statically Defined Tracing probes at:
`probe_query_start` / `probe_query_end`, `probe_phase_*`, `probe_operator_*`,
`probe_parallel_agg_*`. Attach with `bpftrace`, `perf record`, or eBPF — zero
overhead when not attached.

**Layer 3 — Structured logs:** JSON-compatible, stable field names. Log parsers
survive message-text changes.

No sidecar agent. No SDK. No proprietary plugin.

---

## 11. Evidence-gated release discipline

Every release train closes on a **24-hour soak test** and a **pinned benchmark**.
Evidence packs (logs, metrics, raw benchmark CSV, SHA-256 checksums) are shipped
inside the release tarball.

Starting v0.7 Open Beta, benchmark kits will be **publicly reproducible**:
exact hardware profile, dataset, isolation level, durability settings, commands.

Full comparison baseline: **PostgreSQL 18.3** (OLTP), **ClickHouse 24.x** (OLAP),
**TiDB 8.x** (HTAP).

---

## Summary — capabilities at a glance

| Property | Status | Unique aspect |
|---|---|---|
| Five storage engines, mixed in one query | ✅ | No other OLTP database offers `memory DURABILITY = none` alongside columnar HTAP |
| Query service levels (`SET service_level`) | ✅ | I/O priority scheduling per session, privilege-gated |
| Fail-closed resource contracts | ✅ | 8 named limits, each with `SQLSTATE` + Prometheus metric |
| UNDO-log MVCC (no VACUUM) | ✅ | Heap holds only current versions; maintenance-free |
| Tablespaces (per-table filesystem placement) | ✅ | NVMe/HDD/archive tiering without ETL |
| Backup v2 with integrity verification | ✅ | Online, columnar-aware, immutable archive |
| LISTEN/NOTIFY (transactional) | ✅ | No external broker for ephemeral pub/sub |
| ARIES crash recovery | ✅ | Automatic; proportional to WAL, not data size |
| Linux-native observability (USDT + Prometheus) | ✅ | No sidecar; zero-overhead probes |
| Named workload classes with resource quotas | 🔜 v0.7 | Full CPU/mem/UNDO isolation per workload class |
| HA auto-failover (built-in consensus) | 🔜 v0.7 | No external coordinator (Patroni/etcd) |
| WAL-based CDC (AngaraStream Phase 1) | 🔜 v0.7 | At-least-once; Kafka/NATS connectors |
| Table partitioning (RANGE/LIST/HASH) | 🔜 v0.8 | Per-partition engine selection (row + column) |
| Transparent horizontal sharding | 🔜 v0.9 | No application changes required |

---

*Questions about a design decision?
Open a [Discussion](../../discussions) — architectural proposals are welcome.*
