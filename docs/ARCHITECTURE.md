
# AngaraBase — Architecture Overview

This document describes the high-level architecture of AngaraBase: design decisions,
system components, resource contracts, and strategic direction.

For full documentation and runbooks: [angarabase.dev](https://angarabase.dev)

---

## Contents

- [System map](#system-map)
- [Key architectural decisions](#key-architectural-decisions)
- [Resource boundaries — fail-closed contract](#resource-boundaries)
- [Storage architecture](#storage-architecture)
- [Strategic horizons](#strategic-horizons)

---

## System map

```
Clients / Drivers
      │ pgwire v3
      ▼
┌─────────────────────────────────────────────────────────────┐
│  angarabased — protocol adapter                             │
│  SCRAM-SHA-256 · pgwire · session context · RBAC            │
└────────────────────────┬────────────────────────────────────┘
                         │ SQL + session ctx
                         ▼
┌─────────────────────────────────────────────────────────────┐
│  Engine core                                                │
│  ┌────────────────┐   ┌─────────────────────────────────┐  │
│  │  Query Pipeline│   │  Transaction Manager            │  │
│  │  CBO · IR Exec │   │  UNDO MVCC · snapshots          │  │
│  │  SIMD kernels  │   │  watermarks · admission ctrl    │  │
│  └───────┬────────┘   └──────────────┬──────────────────┘  │
│          │ DML/DQL                   │ WAL                  │
│          ▼                           ▼                      │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Storage Manager — engine dispatch                     │ │
│  │  ┌──────────────┐ ┌─────────────┐ ┌─────────────────┐ │ │
│  │  │ HeapStore    │ │AngaraMemory │ │ AngaraColumn    │ │ │
│  │  │ BufferPool   │ │ in-memory   │ │ segments + cache│ │ │
│  │  │ UndoStore    │ │ tables      │ │ OLAP / HTAP     │ │ │
│  │  └──────┬───────┘ └─────────────┘ └─────────────────┘ │ │
│  └─────────┼──────────────────────────────────────────────┘ │
│            │                                                │
│  ┌─────────▼──────────────────────────────────────────────┐ │
│  │  WAL / Recovery — ARIES (Analysis → Redo → Undo + CLR)│ │
│  │  AngaraIO — io_uring / O_DIRECT · fsync contract       │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Key architectural decisions

| Decision | Model | Consequence |
|---|---|---|
| **MVCC** | UNDO-log (Oracle/InnoDB style) | No VACUUM · compact heap · bounded UNDO chain (max 15 deltas/row) |
| **Storage** | Unified per-DB `.adb` file, 48-bit PageId | Single file per database · shared BufferPool across tables |
| **Recovery** | ARIES (Analysis → Redo → Undo + CLR) | WAL-first · crash-safe · UNDO rolls back uncommitted transactions |
| **Engines** | Pluggable `TableEngine` trait | Row / Memory / Columnar / HTAP — same SQL, different physical layout |
| **Async model** | Hybrid: new components async (tokio), storage sync | `spawn_blocking` bridge · full async migration in v0.7 horizon |
| **Scale-out** | Coordinator + Shard nodes (v0.8+) | Single-node path unchanged · sharding is opt-in |
| **Security** | Fail-closed · TDE · Zero Trust layered | Security checklist required in every RFC touching I/O or auth |
| **Platform** | Linux-only | io_uring · eBPF · O_DIRECT · fsync guarantees |

---

## Resource boundaries

AngaraBase is **fail-closed by design**: every component has explicit resource limits.
When a limit is reached the system rejects the request with a documented SQLSTATE code
rather than silently degrading.

| Component | Config knob | On violation | SQLSTATE | Observable metric |
|---|---|---|---|---|
| Buffer Pool | `buffer_pool_size_mb` | Eviction (CLOCK) / WAL-first flush | — | `angarabase_buffer_evictions_total` |
| Transaction Write Set | `txn_max_write_set_mb` | Reject DML, rollback | `54023` | `angarabase_txn_writeset_rejects` |
| UNDO Store | `undo_max_size_mb` | Reject new writes | `53100` | `angarabase_undo_rejects_total` |
| Admission Controller | `max_concurrent_queries` | Reject query (overloaded) | `53300` | `angarabase_admission_rejects_total` |
| Connection pool | `max_connections` | Reject new connection | `53300` | `angarabase_conn_rejects_total` |
| Statement timeout | `statement_timeout_ms` | Cancel query | `57014` | `angarabase_query_timeouts_total` |
| Snapshot age | `max_snapshot_age` | Force-close stale snapshot | `40001` | `angarabase_snapshot_force_closed` |
| I/O queue | `io_queue_depth` | Backpressure | — | `angarabase_io_backpressure_events` |

**Reaction chain:** when a boundary is exceeded, the error propagates up the call stack with
a stable SQLSTATE code so the client application can implement an appropriate retry policy
or circuit breaker.

All metrics are exposed via Prometheus-compatible endpoint (`/metrics`).
All blocking waits emit wait events compatible with pg_wait_sampling-style tooling.

See `contracts/admission_control.rs` for the machine-readable contract specification.

---

## Storage architecture

### UNDO-log MVCC (the key distinction from PostgreSQL)

```
INSERT/UPDATE/DELETE
  → TxnWriteSet (per-txn in-memory staging, bounded by txn_max_write_set_mb)
  → on COMMIT:
      WAL record (durability first)
      → HeapFile (current version of the row, in .adb page)
      → old version moved to UndoStore (append-only, disk)

READ (any isolation level)
  → BufferPool.pin(page_id) → current row version on HeapPage
  → if the version is newer than my snapshot:
      walk back the UNDO chain to the visible version
  → merge with local TxnWriteSet (uncommitted changes of this transaction)
```

**Consequence:** there is no VACUUM process. Old row versions in the UNDO log are
reclaimed by a background GC worker as transactions that needed them complete.
Heap pages contain only one version of each row — no bloat accumulation.

### Pluggable engine dispatch

The `StorageType` of a table (set at `CREATE TABLE` time) determines which engine handles it:

| `storage_type` | Primary store | WAL | Use case |
|---|---|---|---|
| `Heap` | HeapStore + BufferPool | Yes | General-purpose OLTP |
| `Memory` | AngaraMemory (in-process) | No | Ephemeral / session tables |
| `Columnar` | AngaraColumn segments | Metadata only | Analytical / OLAP |
| `HtapRowColumn` | HeapStore + AngaraColumn | Yes | Hybrid OLTP + OLAP |

---

## Strategic horizons

### Current — v0.6 (active)

- UNDO MVCC fully operational (per-DB MVCC, ARIES recovery, bounded UNDO chain).
- B-tree index store (disk-backed, version-aware, MV-PBT style).
- Columnar storage (AngaraColumn): ManifestLog, zone maps, L0/L1 compaction, SIMD kernels.
- CBO (cost-based optimizer) with IR executor.
- HTAP routing (row + column, same SQL surface).
- `REINDEX CONCURRENTLY`.
- Streaming replication (physical, WAL-based).
- PostgreSQL pgwire v3 · SCRAM-SHA-256.
- Prometheus metrics · wait events · USDT probes.

### v0.7 (planned)

- Full async storage layer (complete hybrid migration).
- Partition routing v1 (RANGE / LIST, INSERT / SELECT / pruning).
- Auto-failover for streaming replication.
- Advanced security: TDE (transparent data encryption), per-role audit log.
- User-defined functions (scalar, read-only).
- Performance: adaptive query re-optimization, spill-to-disk for large aggregations.

### v0.8+ (horizon)

- Transparent sharding: AngaraCoord coordinator + AngaraShard data nodes.
- Globally distributed read replicas.
- Multi-region active-standby.
- Vector search integration (pgvector-compatible surface).

---

## Physical portability (cold DR)

AngaraBase data files are designed for physical portability — the same principle as
"copy data directory to another host." To move an instance:

1. Stop the server.
2. Copy the data directory (`.adb` files + WAL journal).
3. Set the paths in the config on the new host.
4. Start the same or a compatible binary.

The on-disk format includes a `format_version` marker. AngaraBase fails closed
(refuses to start) if the binary's expected format does not match the files on disk —
preventing silent corruption from version mismatches.

---

## See also

- `contracts/admission_control.rs` — machine-readable resource boundary contract
- `contracts/table_engine.rs` — `TableEngine` trait specification
- [RELEASES.md](../RELEASES.md) — how releases are verified (GPG, SHA-256, evidence pack)
- [angarabase.dev](https://angarabase.dev) — full documentation and runbooks
