
# AngaraBase — Reliability Guarantees & Failure Modes

This document covers what AngaraBase promises, what it does not promise, and
what happens when things go wrong. Reliability properties in AngaraBase are
**declared in source code as contracts** and enforced at engine level — not
stated only in documentation.

---

## Core guarantees

### G1 — Fail-closed resource contracts

AngaraBase enforces eight named resource boundaries at engine level.
When any boundary is reached, the engine returns a deterministic `SQLSTATE` error
**before** the incident — never after silent degradation.

| Resource | Config knob | Metric | `SQLSTATE` on breach | Behavior |
|---|---|---|---|---|
| UNDO store disk budget | `undo_max_size_mb` | `angarabase_undo_store_bytes_used` | `53100` | Reject DML; GC cycle; wait or return error |
| Buffer pool memory | `buffer_pool_size_mb` | `angarabase_buffer_pool_evictions_total` | `53200` | Evict pages; OOM guard rejects on overflow |
| Concurrent queries (admission gate) | `max_concurrent_queries` | `angarabase_admission_rejected_total` | `53300` | Reject immediately; no queuing by default |
| Connections | `max_connections` | `angarabase_connections_total` | `53300` | Reject new TCP connection; existing unaffected |
| Per-transaction write set | `txn_max_write_set_mb` | `angarabase_txn_write_set_bytes` | `54023` | Reject DML; transaction must roll back |
| AngaraMemory row capacity | `memory_engine_max_rows` | `angarabase_memory_engine_rows_total` | `53000` | Reject INSERT; no silent drop |
| Statement timeout | `statement_timeout_ms` | `angarabase_statement_timeout_total` | `57014` | Cancel statement; transaction must roll back |
| Snapshot age (stale txn) | `max_snapshot_age` | `angarabase_snapshot_force_close_total` | `40001` | Force-close snapshot; serialization failure |

The authoritative source for this table is [`contracts/admission_control.rs`](../contracts/admission_control.rs).
Every boundary is compiled into the engine as a `ResourceBoundaryDescriptor` constant.

Every metric is exposed on the Prometheus endpoint. Every `SQLSTATE` is
documented in `contracts/resource_boundaries.rs`.

> **Design principle:** a budget exhausted by an analytical query does not
> propagate to the OLTP path. Resource quotas are per-workload-class.

### G2 — UNDO-log MVCC: no VACUUM, no heap bloat

Historical row versions live in a separate UNDO log. The heap holds only
current (latest committed) row versions. There is no background maintenance
job that competes with live queries. Snapshot visibility is deterministic
and does not depend on VACUUM having run.

Long-running transactions are bounded by the UNDO log capacity contract (G1).

### G3 — ARIES crash recovery

Recovery follows the ARIES protocol: Analysis → Redo → Undo, with
Compensation Log Records (CLR) for transactions that must be rolled back.

| Scenario | Guarantee |
|---|---|
| Server crash (SIGKILL, power loss) | Automatic recovery on next start |
| Torn write (partial page) | Detected via CRC32C page checksums |
| WAL corruption | Detected; recovery halts at last consistent LSN |
| In-flight transaction at crash | Automatically rolled back via Undo pass |
| Committed transaction at crash | Data preserved; visible after recovery |
| Recovery time | Proportional to WAL since last checkpoint (not data size) |

### G4 — Evidence-gated releases

Every release train closes on a **24-hour soak test** and a pinned benchmark run.
Evidence packs (logs, metrics, benchmark results, SHA-256 checksums) are shipped
inside the release tarball and archived in GitHub Releases.

No release is published without a passing soak gate.

---

## Failure modes

### Transaction failures

| Failure | Engine behavior | Client behavior required |
|---|---|---|
| Write-set limit exceeded | `ROLLBACK` + `SQLSTATE 54023` | Retry with smaller batch |
| UNDO space exhausted | `ROLLBACK` + `SQLSTATE 53100` | Wait for GC cycle; reduce concurrent long transactions |
| Deadlock detected | One transaction aborted + `SQLSTATE 40P01` | Retry the aborted transaction |
| Serialization conflict | Aborted + `SQLSTATE 40001` | Retry the aborted transaction |
| Connection limit reached | Rejected + `SQLSTATE 53300` | Queue at application layer |
| Statement timeout | `ROLLBACK` + `SQLSTATE 57014` | Application handles timeout |

### Storage failures

| Failure | Engine behavior |
|---|---|
| Page CRC32C mismatch on read | Error returned to query; page flagged |
| WAL write failure (disk full) | Server halts writes; returns error to all active txns |
| Heap file corruption (partial) | Affected pages surfaced as errors; intact pages unaffected |
| Storage engine swap (crash mid-operation) | ARIES Undo pass rolls back; consistent state restored |

### Replication failures

| Failure | Behavior |
|---|---|
| Replica disconnect | Primary continues; replica falls behind |
| Replica reconnect | WAL streaming resumes from last received LSN |
| Replica too far behind (WAL recycled) | Replica must re-sync from base backup |
| Synchronous replica timeout | Configurable: wait or degrade to async |

### Not handled (known limitations, pre-v0.9)

- **Split-brain in multi-node** — auto-failover (Raft) is planned for v0.7; manual failover available now.
- **Gray failure (partial network partition)** — not yet covered by automated fencing.
- **Disk silent data corruption (bit rot)** — CRC32C detects most cases; scrubbing not yet automated.

---

## Benchmarks

> **Status: coming in v0.7 open beta.**

We publish reproducible benchmark kits with each release train starting v0.7.
Each kit includes:

- Hardware profile (CPU, RAM, NVMe specs)
- Dataset and scale factor
- Warm / cold cache conditions
- Durability settings (fsync on/off, WAL sync mode)
- Isolation level used
- Exact commands to reproduce
- Raw results (CSV) + summary (p50 / p95 / p99 / throughput)

**Planned benchmark scenarios:**

| Scenario | Target comparison |
|---|---|
| Single-threaded OLTP latency (simple SELECT, INSERT) | PostgreSQL 16 |
| TPC-C-like mixed OLTP (16–64 connections) | PostgreSQL 16, TiDB |
| Analytical (TPC-H subset) while OLTP running | ClickHouse (OLAP only), TiDB (HTAP) |
| p99 OLTP latency during heavy analytical scan | PostgreSQL (degradation baseline) |
| Crash recovery time (WAL replay) | PostgreSQL 16 |
| Replica lag under write-heavy load | PostgreSQL streaming replication |
| Ingest rate (bulk INSERT) | PostgreSQL 16, ClickHouse |

> Key claim to validate: *"A concurrent TPC-H scan does not degrade TPC-C p99 latency
> beyond the configured analytical CPU budget."*

Until v0.7 benchmarks are published, claims about performance are based on internal
soak tests (evidence packs in Releases) — not independently reproducible numbers.
We say this explicitly rather than in a footnote.

---

## Testing story

### What is tested today

| Layer | Coverage |
|---|---|
| Unit tests (per-crate) | ✅ Extensive: MVCC visibility, WAL write/read, recovery phases, planner rewrites, executor operators, admission control |
| Integration tests (SQL-level) | ✅ DQL / DML / DDL correctness; isolation anomalies (G0–G2 test suite); MVCC anomaly tests |
| Crash-recovery tests | ✅ SIGKILL at critical points; torn-write simulation; ARIES Undo pass verification |
| Soak test (24-hour) | ✅ Run before every release train close |
| HTAP isolation tests | ✅ Analytical quota enforcement; OLTP p99 under concurrent OLAP |
| Replication tests | ✅ Streaming replication, WAL gap handling, replica reconnect |

### What is coming

| Layer | Status |
|---|---|
| SQL logic tests (pgregress subset) | 🔜 v0.7 (Open Beta gate) |
| Jepsen-style distributed invariant tests | 🔜 v0.7 (with auto-failover) |
| Reproducible public benchmark kit | 🔜 v0.7 |
| Chaos engineering harness (network partition, node crash drills) | 🔜 v0.7 |
| fsync / power-loss style tests (disk-level) | 🔜 v0.7 |
| Fuzzing (SQL parser + executor) | 🔜 v0.8 |

---

## Production readiness matrix

| Dimension | v0.6.x (now) | v0.7 (next) | v0.9 (GA) |
|---|:---:|:---:|:---:|
| OLTP correctness | ✅ | ✅ | ✅ |
| ARIES crash recovery | ✅ | ✅ | ✅ |
| Streaming replication | ✅ | ✅ | ✅ |
| Auto-failover (HA) | ❌ manual only | ✅ Raft | ✅ |
| Reproducible benchmarks | ❌ internal only | ✅ public | ✅ |
| SQL logic test suite | ❌ | ✅ | ✅ |
| On-disk format stability | ❌ pre-release | ⚠️ beta | ✅ stable |
| SLA-backed commercial support | ❌ | ✅ Commercial | ✅ |
| Community Edition (license key) | ❌ | ⚠️ preview | ✅ GA |
| Recommended for | Research pilots, design partners | Open Beta, supervised production | GA production |

---

*Full failure-mode tables and runbooks: [angarabase.dev → Operations](https://angarabase.dev/operations/)*  
*Evidence packs from soak tests: [GitHub Releases](../../releases)*  
*Found an issue? [Open a bug report](../../issues)*
