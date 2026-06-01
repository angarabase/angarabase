
# AngaraBase — Design Decisions & Unique Properties

This document explains the engineering choices that make AngaraBase different
from PostgreSQL, TiDB, and the Postgres-plus-ClickHouse stack — and why those
choices matter in practice.

---

## 1. Fail-closed resource contracts

Most databases degrade silently: under memory pressure they spill to disk without
warning; under connection pressure they queue invisibly; under write pressure they
slow down until someone notices on a dashboard.

AngaraBase takes the opposite approach. **Every resource boundary is a contract:**

- Declared in source code (`contracts/admission_control.rs`)
- Observable as a Prometheus metric (same name, always)
- Enforced by returning an explicit `SQLSTATE` error *before* the incident

There are no soft limits that gradually degrade. There is no silent queuing that hides
backpressure. When a boundary is reached, the offending operation fails fast with
a deterministic error code — and the rest of the system continues unaffected.

**Why this matters:** a billing service and an analytics job share the same instance.
The analytics job exhausts its write-set budget (`54023`) and rolls back. The billing
service never notices. This is the difference between *co-location* and *isolation*.

Reference: [`contracts/admission_control.rs`](../contracts/admission_control.rs)

---

## 2. UNDO-log MVCC — the Oracle/InnoDB model, not PostgreSQL's

PostgreSQL uses *heap-based MVCC*: old row versions accumulate in the heap, and
a background VACUUM process cleans them up. VACUUM competes with live queries,
causes heap bloat, and produces unpredictable maintenance windows.

AngaraBase uses *UNDO-log MVCC* — the same model as Oracle and InnoDB:

- The **heap holds only current (latest committed) row versions**
- Historical versions live in a **separate UNDO log**, outside the heap
- VACUUM does not exist — there is nothing to clean up in the heap
- Snapshot visibility is **deterministic**: a transaction's read view is a function
  of its start LSN, not of whether VACUUM has run

The UNDO log is itself a bounded resource (contract: `undo_max_size_mb` / SQLSTATE `53100`).
Long-running transactions that produce excessive UNDO volume are rejected before they
exhaust the budget, not after the instance runs out of disk.

**Why this matters:** predictable write amplification, no autovacuum storms, no
maintenance windows, no "table bloat" tickets at 3 AM.

Reference: [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) §Storage

---

## 3. Three storage engine types in a single instance — mix per table, join across

Every table in AngaraBase is backed by a **`TableEngine`** trait. The same query
can read from tables backed by different engines simultaneously. No ETL, no
materialized intermediate.

| Engine | Durability | Use case |
|---|---|---|
| **Row store** (default) | Full WAL + UNDO-log MVCC | OLTP, system of record |
| **AngaraMemory — `none` tier** | None (lost on restart) | Ephemeral hot working sets, caches |
| **AngaraMemory — `logged` tier** | WAL-backed (survives restart) | Session state, hot aggregations |
| **AngaraMemory — `snapshotted` tier** | Periodic snapshot + WAL | Pre-aggregated views, read-heavy |
| **AngaraColumn** | WAL + columnar format | HTAP analytics, SIMD-accelerated scans |

A single query can join `orders` (row store, OLTP) with `daily_summary` (AngaraColumn,
pre-aggregated) and `session_cache` (AngaraMemory/none, ephemeral) — without any
external data movement.

**Coming in v0.8: table partitioning** — declarative `PARTITION BY RANGE / LIST / HASH`
with automatic partition pruning and partition-local indexes. Partitions can use
different storage engines (e.g., recent partitions in row store, historical in
AngaraColumn). This enables automatic tiering of aging data within a single table.

Reference: [`contracts/table_engine.rs`](../contracts/table_engine.rs)

---

## 4. HTAP with contractual workload isolation — not just co-location

Most databases that claim "HTAP" put OLTP and OLAP in the same process and rely
on the scheduler to be fair. Under real workload pressure, long analytical scans
consume CPU and memory, causing OLTP latency to spike.

AngaraBase uses **per-workload-class resource quotas** backed by the fail-closed
contract model (see §1). The analytical workload class has its own CPU budget,
memory limit, and write-set cap. When the analytical job hits its budget, it receives
a `SQLSTATE` error and rolls back. The OLTP path — running under its own contract —
continues with no measurable impact.

The contracts are not soft hints. They are engine-level enforcement points.

**Planned: Query SLA contexts (v0.7+)**

Each session or query will be assignable to a named workload class with explicit
resource constraints:

```sql
SET angara.workload_class = 'analytics';
-- or per-statement:
SELECT /*+ workload_class(analytics) */ ...
```

The workload class defines CPU budget, memory cap, and UNDO quota.
Breaching any limit returns a deterministic `SQLSTATE` and does not affect
sessions in other workload classes.

---

## 5. AngaraStream — built-in event bus, no external broker

AngaraBase ships with two levels of pub/sub:

**Level 0 — Ephemeral LISTEN/NOTIFY (delivered today):**
The standard PostgreSQL `LISTEN` / `NOTIFY` / `UNLISTEN` commands, implemented
with a per-channel sharded dispatcher (64 shards). Notifications are transactional:
inside a transaction they are deferred to `COMMIT`; on `ROLLBACK` they are discarded.
Delivery guarantee: at-most-once (ephemeral — no persistence, no replay).

**Level 1 — WAL-based CDC with delivery guarantees (v0.7):**
AngaraStream Phase 1 provides WAL logical decoding, per-row CDC events with
at-least-once delivery, and connectors to Kafka and NATS. This replaces the need
for Debezium or a separate CDC pipeline entirely.

**Why this matters:** many architectures use Postgres + Kafka + Debezium just to get
reliable change events from the database. AngaraBase eliminates that layer — the
event bus is part of the engine, not a separate system.

---

## 6. Linux-native observability — metrics, probes, and structured logs

AngaraBase is instrumented at three levels:

**Prometheus metrics** — every resource boundary has a gauge or counter. The
endpoint is always on; there is no opt-in. Alerting rules can be written directly
against contract metrics (e.g., `angarabase_undo_store_bytes_used > 0.8 * limit`).

**USDT probes** — User Statically Defined Tracing probes at critical paths:
`probe_query_start` / `probe_query_end`, `probe_phase_*`, `probe_operator_*`,
`probe_parallel_agg_*`. These are zero-overhead when not attached and work with
`bpftrace`, `perf record`, and `eBPF` tracers without any instrumentation agent.

**Structured logs** — every log line has stable field names (JSON-compatible).
Log parsers do not need to be updated when message text changes; only the field
schema is stable.

This makes AngaraBase debuggable at the kernel level without adding an agent sidecar,
a tracing SDK, or a proprietary monitoring plugin.

---

## 7. Evidence-gated release discipline

"Fast" and "reliable" are claims. AngaraBase treats them as artifacts to be verified,
not adjectives to be asserted.

Every release train closes on:
- A **24-hour soak test** — production-representative workload, continuous
- A **pinned benchmark run** — specific hardware, dataset, scale factor, isolation level
- A **SHA-256 signed tarball** with evidence pack included

The evidence pack ships inside the release archive and is archived in GitHub Releases.
No release is published without a passing soak gate.

Starting v0.7 Open Beta, benchmark kits will be **publicly reproducible**: hardware
profile, dataset, exact commands, raw CSV results. Not "we ran it and it was fast" —
"here is how you reproduce this on equivalent hardware."

---

## Coming: what makes the next releases different

| Feature | Version | Design note |
|---|---|---|
| Query SLA workload contexts | v0.7 | Per-session/query workload class assignment with CPU/mem/UNDO quotas |
| Table partitioning (RANGE, LIST, HASH) | v0.8 | Partition-local indexes; mixed storage engines per partition |
| AngaraTuner — automatic memory advisor | v0.7 | Runtime `shared_buffers` / `work_mem` guidance without manual tuning |
| HA auto-failover (Raft) | v0.7 | No external coordinator (Patroni/etcd); built-in leader election |
| WAL-based CDC (AngaraStream Phase 1) | v0.7 | Kafka/NATS connectors; at-least-once delivery; replaces Debezium |
| Tiered storage (NVMe + S3) | v0.7 | Hot/warm/cold tier; automatic data migration |
| WASM UDF sandbox | v0.7 | Polyglot UDFs (Rust, C, Go) in a WASM-isolated environment |

Full roadmap: [ROADMAP.md](../ROADMAP.md)

---

*Have a question about a design decision?
Open a [Discussion](../../discussions) — architectural proposals are welcome.*
