
# AngaraBase — Schema & Runtime Introspection

> **Quick start:** [`INTROSPECTION_QUICKSTART.md`](INTROSPECTION_QUICKSTART.md)
> — 15 queries, 5 minutes, understand the full system without reading source code.

AngaraBase organizes its introspection surface into **four layers**, each with a distinct purpose.
All views are queryable via standard `pgwire v3` — connect with `psql`, JDBC, asyncpg, or any
PostgreSQL-compatible driver. No extension, no special mode, no proprietary protocol.

| Layer | Namespace | Purpose | Audience |
|---|---|---|---|
| **Portable SQL Metadata** | `information_schema.*` | SQL:2016-standard schema objects | ORMs, JDBC/ODBC drivers, cross-DB tooling |
| **PostgreSQL-Compatible Catalog** | `pg_catalog.*` | Drop-in PostgreSQL catalog compatibility | psql, pgAdmin, Flyway, pg_dump, pg drivers |
| **Runtime Observability Views** | `angara_stat_*` · `sys.health` · `sys.identity` | Live session, wait, QoS, and checkpoint state | DBAs, SREs, monitoring dashboards |
| **Engine Contract Views** | `sys.tables` · `sys.gc_tuning_status` · `sys.workload_stats` | Introspect engine-level guarantees and contracts | Platform engineers, HTAP workload designers |

---

## Layer 1 — Portable SQL Metadata

*Use this layer for ORM bootstrap, schema migration tools, and cross-database tooling.*
*These views follow SQL:2016 and behave identically to standard PostgreSQL.*

| View | Key columns | Use |
|---|---|---|
| `information_schema.tables` | `table_catalog`, `table_schema`, `table_name`, `table_type` | List all tables; ORM introspection |
| `information_schema.columns` | `table_name`, `column_name`, `data_type`, `is_nullable`, `column_default`, `ordinal_position` | Column metadata; DDL scaffolding |
| `information_schema.constraint_column_usage` | `table_name`, `column_name`, `constraint_name` | FK/PK column mapping |

**Coming in v0.7:** `referential_constraints`, `key_column_usage`.

---

## Layer 2 — PostgreSQL-Compatible Catalog

*Use this layer when your tooling (psql, pgAdmin, Flyway, ActiveRecord, SQLAlchemy) expects `pg_catalog`.*
*The goal is zero changes to existing PostgreSQL tooling.*

| View | Status | Notes |
|---|:---:|---|
| `pg_catalog.pg_tables` | ✅ | |
| `pg_catalog.pg_indexes` | ✅ | |
| `pg_catalog.pg_constraint` | ✅ | PK, FK, UNIQUE, CHECK |
| `pg_catalog.pg_namespace` | ✅ | |
| `pg_catalog.pg_database` | ✅ | |
| `pg_catalog.pg_sequences` / `pg_sequence` | ✅ | |
| `pg_catalog.pg_roles` / `pg_user` | ✅ | |
| `pg_catalog.pg_settings` | ✅ | |
| `pg_catalog.pg_index` | ✅ | Index OID metadata |
| `pg_catalog.pg_proc` | ⚠️ | Built-in functions only |
| `pg_catalog.pg_locks` | 🔜 v0.7 | Requires HA / multi-node |
| `pg_catalog.pg_stat_replication` | ⚠️ | Partial — replica lag via Prometheus |
| `pg_catalog.pg_stat_progress_*` | 🔜 v0.8 | Progress views |

---

## Layer 3 — Runtime Observability Views

*Use this layer to understand what is happening right now: sessions, waits, checkpoints, queue depth.*

### Session and connection state

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `angara_stat_activity` | `pid`, `datname`, `usename`, `state`, `query_start`, `query_fingerprint`, `consumer_id`, `wait_event`, `wait_event_type` | `pg_stat_activity` |
| `angara_stat_wait_events` | `event`, `event_type`, `total`, `active`, `total_duration_us` | `pg_stat_activity` wait columns |
| `sys.health` | `uptime_seconds`, `connections_active`, `connections_accepted_total`, `txn_commit_epoch_current`, `txlog_durable_lsn` | No single-view PG equivalent |
| `sys.identity` | `cluster_id`, `db_name`, `lease_holder_id`, `lease_expires_at`, `lease_acquired_at`, `lease_holder_hostname`, `recovery_mode` | No PG equivalent |

### Database and checkpoint statistics

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `angara_stat_database` | `datname`, `numbackends`, `xact_commit`, `xact_rollback`, `blks_read`, `blks_hit`, `conflicts` | `pg_stat_database` |
| `angara_stat_bgwriter` | `checkpoints_timed`, `checkpoints_req`, `checkpoint_write_time_ms`, `checkpoint_sync_time_ms`, `buffers_checkpoint`, `buffers_clean`, `checkpoint_errors` | `pg_stat_bgwriter` |

### Query execution statistics

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `angara_stat_statements` | `queryid`, `consumer_id`, `query`, `calls`, `total_exec_time_ms`, `min_exec_time_ms`, `max_exec_time_ms`, `mean_exec_time_ms`, `rows`, `shared_blks_hit`, `class` | `pg_stat_statements` — built-in, no extension needed |
| `angara_top_queries(N)` | `queryid`, `consumer_id`, `query`, `calls`, `total_exec_time_ms`, `rows` | `pg_stat_statements ORDER BY … LIMIT N` |

### Plan Store — plan history and regression detection

> No PostgreSQL equivalent. Analogous to SQL Server Query Store.

| View | Key columns | Purpose |
|---|---|---|
| `angara_query_store_entries` | `query_id`, `query_text`, `first_seen`, `last_seen`, `plan_count` | All tracked query shapes |
| `angara_query_store_plans` | `plan_id`, `query_id`, `plan_json`, `first_seen`, `last_seen`, `is_regressed` | Historical plans; flags regressions |
| `angara_query_store_intervals` | `query_id`, `plan_id`, `interval_start`, `interval_end`, `calls`, `mean_exec_time_ms`, `rows` | Per-plan performance over time windows |

### Security and compliance

| View | Key columns |
|---|---|
| `sys.roles` | `role_name` |
| `sys.user_roles` | `user_name`, `role_name`, `enabled` |
| `sys.role_privileges` | `role_name`, `privilege`, `enabled` |
| `sys.audit_log` | `ts_unix_seconds`, `user_name`, `action`, `result`, `reason` |

---

## Layer 4 — Engine Contract Views

*This layer is unique to AngaraBase. It exposes the engine-level contracts that make "Predictable by contract" a verifiable statement, not a marketing claim.*

*Every field here reflects an architectural decision — not just a statistic.*

### Table engine metadata

`sys.tables` exposes per-table engine contracts — fields that have no equivalent in PostgreSQL because PostgreSQL has a single storage engine:

| Column | What it expresses |
|---|---|
| `storage_engine` | `heap` / `memory` / `column` — which engine backs this table |
| `durability` | `full` / `logged` / `none` — the crash-survival guarantee declared at `CREATE TABLE` |
| `eviction_policy` | Memory eviction strategy for `AngaraMemory` tables |
| `mutation_policy` | Whether the table is append-only or allows UPDATE/DELETE |
| `append_only` | Append-only flag — enforced at engine level |
| `max_rows` | Declared row capacity contract for memory tables |
| `row_count_estimate` | Live planner input — updated after ANALYZE or DML |

```sql
-- Show the full engine contract for every table in the public schema
SELECT table_name, storage_engine, durability, max_rows,
       eviction_policy, append_only, mutation_policy, row_count_estimate
FROM sys.tables
WHERE schema_name = 'public'
ORDER BY table_name;
```

### Workload isolation state

| View | Key columns | What it proves |
|---|---|---|
| `angara_stat_qos_queues` | `level` (critical/interactive/background), `queued_total`, `rejected_total`, `blocking_inflight` | "Analytics cannot degrade OLTP" — real-time queue depth and rejection rate per service level |
| `sys.workload_stats` | `schema_name`, `table_name`, `query_class`, `access_count`, `seq_scan`, `idx_scan`, `tuples_read`, `tuples_written` | Per-table access breakdown by OLTP vs analytical workload class |

```sql
-- Is any service level being throttled right now?
SELECT level, queued_total, rejected_total, blocking_inflight
FROM angara_stat_qos_queues
WHERE rejected_total > 0 OR blocking_inflight > 0;

-- Which tables are accessed heavily by analytics vs OLTP?
SELECT table_name, query_class, access_count, seq_scan, idx_scan
FROM sys.workload_stats
ORDER BY access_count DESC
LIMIT 20;
```

### UNDO log GC state ("No VACUUM" control loop)

PostgreSQL runs VACUUM to reclaim dead tuple space. AngaraBase uses UNDO-log MVCC — there is no VACUUM.
The GC control loop is fully observable:

| View | Key columns | What it shows |
|---|---|---|
| `sys.gc_tuning_status` | `current_budget`, `bloat_ratio_percent`, `tuning_decision`, `min_active_epoch_lag`, `sleep_ms`, `decisions_increase_total`, `decisions_decrease_total` | The automatic GC cycle: current UNDO budget, last tuning decision, ratio of increase vs decrease cycles |

```sql
-- Is the UNDO GC keeping up? What is its current tuning decision?
SELECT current_budget, bloat_ratio_percent, tuning_decision,
       min_active_epoch_lag, decisions_increase_total, decisions_decrease_total
FROM sys.gc_tuning_status;
```

### Statistics and planner internals

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `sys.table_stats` | `seq_scan`, `idx_scan`, `tuples_read`, `tuples_written`, `row_count_estimate`, `row_count_live`, `dml_change_counter`, `last_committed_rowid` | `pg_stat_user_tables` |
| `sys.index_stats` | `seeks`, `scans`, `tuples_read`, `cache_hit`, `cache_miss` | `pg_stat_user_indexes` |
| `sys.column_stats` | `null_ppm`, `distinct_estimate`, `ndv_approx`, `col_min`, `col_max`, `histogram_bounds`, `mcv_values`, `mcv_frequencies`, `hll_enabled`, `reservoir_size`, `reservoir_epoch` | `pg_stats` — **full statistics state exposed**, including HLL sketch, histogram bounds, MCV lists |
| `sys.multicolumn_stats` | `col_a`, `col_b`, `correlation`, `ndv_joint`, `mcv_joint` | `pg_statistic_ext` (partial) |

### Adaptive Query Processing (AQP) state

| View | Purpose |
|---|---|
| `sys.aqp_stats` | AQP cardinality feedback loop statistics |
| `sys.aqp_feedback` | Per-query cardinality correction entries |
| `sys.aqp_blacklist` | Queries excluded from AQP re-optimization |

### Learned index models

| View | Key columns |
|---|---|
| `sys.learned_models` | Model registry |
| `sys.learned_active_models` | Currently active models by table |
| `sys.learned_model_stats` | Per-model accuracy and usage |

### Stream / event bus monitoring

| View | Key columns |
|---|---|
| `sys.stream_subscriptions` | Active LISTEN/NOTIFY subscribers per channel |
| `sys.stream_stats` | `table_name`, `log_size`, `oldest_offset` — event log depth per stream |

### Configuration and live metrics

| View | Key columns |
|---|---|
| `sys.settings` / `sys.knobs` | `name`, `value`, `source`, `dynamic`, `doc` — all configuration knobs with inline docs |
| `sys.settings_meta` | `name`, `scope`, `dynamic`, `restart_required`, `sensitive` |
| `sys.metrics` | `name`, `value` — **all Prometheus counters and gauges queryable via SQL** |

---

## Better than PostgreSQL 18

The following views expose capabilities that either don't exist in PostgreSQL or are significantly richer in AngaraBase.

| Capability | AngaraBase view | PostgreSQL 18 | Why it matters |
|---|---|---|---|
| **Storage engine per table** | `sys.tables.storage_engine` — `heap`/`memory`/`column` | Not applicable (single engine) | Verify which engine backs a table; understand durability guarantee |
| **Durability tier per table** | `sys.tables.durability` — `full`/`logged`/`none` | Not applicable | Contractual crash-survival guarantee per table, declared at creation |
| **Eviction and mutation policy** | `sys.tables.eviction_policy`, `mutation_policy`, `append_only` | Not applicable | Engine-level behavioral contracts |
| **QoS service-level queue depth** | `angara_stat_qos_queues.rejected_total` per level | ❌ No equivalent | Real-time evidence that OLTP is not being throttled by analytics |
| **Per-workload-class table access** | `sys.workload_stats.query_class` | ❌ No equivalent | See how OLTP and analytics split I/O on each table |
| **UNDO GC control loop state** | `sys.gc_tuning_status.tuning_decision` | ❌ VACUUM replaces this | Observe the "no VACUUM" loop; verify it is keeping up |
| **Plan history with regression flags** | `angara_query_store_plans.is_regressed` | ❌ `pg_stat_statements` has no plan history | Catch plan regressions before users report them |
| **Full column statistics state** | `sys.column_stats` — HLL sketch, histogram bounds, MCV lists, reservoir | `pg_stats` — read-only, partial | Understand exactly what the planner knows; audit statistics staleness |
| **All Prometheus metrics via SQL** | `sys.metrics` | ❌ No built-in SQL bridge | Query any counter or gauge from `psql` without a separate metrics stack |
| **Cluster identity + lease state** | `sys.identity.lease_holder_id`, `recovery_mode` | ❌ No equivalent in single-node PG | Diagnose HA failover state and recovery mode from SQL |
| **Structured audit trail** | `sys.audit_log` — stable fields, queryable | ❌ Requires extension (`pgaudit`) | Zero-config audit; no external log parsing |
| **Learned cardinality model registry** | `sys.learned_models`, `sys.learned_active_models` | ❌ No equivalent | Inspect and manage ML-backed cardinality estimation |
| **Event stream monitoring** | `sys.stream_subscriptions`, `sys.stream_stats` | ❌ No equivalent | Monitor LISTEN/NOTIFY and WAL-based CDC pipelines |
| **stat_statements built-in** | `angara_stat_statements` — no extension needed | `pg_stat_statements` (extension, must be installed) | Query performance stats available from day one |

---

## Feature detection for ORM and tooling authors

```sql
-- Detect AngaraBase
SELECT name, value
FROM sys.settings
WHERE name IN ('server_version', 'angara_version', 'angara_build_sha');

-- Detect which introspection layers are available
SELECT table_schema, count(*) AS view_count
FROM information_schema.tables
WHERE table_schema IN ('sys', 'information_schema', 'pg_catalog')
GROUP BY table_schema;

-- Check a specific view exists without error
SELECT * FROM sys.health LIMIT 0;
SELECT * FROM angara_stat_qos_queues LIMIT 0;
```

---

*See also:*
- *[`INTROSPECTION_QUICKSTART.md`](INTROSPECTION_QUICKSTART.md) — 15 queries in 5 minutes*
- *[`SQL_COMPATIBILITY.md`](SQL_COMPATIBILITY.md) — full SQL feature compatibility matrix*
- *[`DESIGN.md`](DESIGN.md) — architectural decisions behind the engine contract model*
