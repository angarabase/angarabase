
# AngaraBase — Schema & Runtime Introspection

AngaraBase provides a **three-namespace introspection model**:

| Namespace | Purpose | Compatibility |
|---|---|---|
| `information_schema.*` | Portable SQL-standard schema metadata | SQL:2016 / JDBC / ODBC / ORMs |
| `pg_catalog.*` | PostgreSQL-compatible catalog | psql, pgAdmin, pg drivers, ORMs |
| `sys.*` / `angara_stat_*` | AngaraBase-native runtime and analytics | AngaraBase-specific, no PG equivalent |

Connect via any PostgreSQL driver (`psql`, libpq, JDBC, asyncpg, `psycopg`, `node-postgres`).
All views are queryable over `pgwire v3` without any special extensions.

---

## Layer A — Schema Discovery

These views answer: *"What objects exist and how are they structured?"*

### `information_schema` (SQL-standard, portable)

| View | Columns | Use |
|---|---|---|
| `information_schema.tables` | `table_catalog`, `table_schema`, `table_name`, `table_type` | List all tables; ORM introspection |
| `information_schema.columns` | `table_catalog`, `table_schema`, `table_name`, `column_name`, `data_type`, `is_nullable`, `column_default`, `ordinal_position` | Column metadata; DDL generation |
| `information_schema.constraint_column_usage` | `table_name`, `column_name`, `constraint_name` | FK/PK column mapping |

### `sys.*` (native AngaraBase schema views)

| View | Key columns | Notes |
|---|---|---|
| `sys.databases` | `db_id`, `name` | All databases in the instance |
| `sys.schemas` | `db_id`, `schema_name` | All schemas in current database |
| `sys.tables` | `db_id`, `schema_name`, `table_name`, `tablespace_name`, `storage_engine`, `durability`, `max_rows`, `eviction_policy`, `append_only`, `mutation_policy`, `row_count_estimate` | **Includes engine type** (heap/memory/column); no PG equivalent |
| `sys.columns` | `db_id`, `schema_name`, `table_name`, `column_name`, `type`, `nullable` | Column metadata |
| `sys.indexes` | `db_id`, `schema_name`, `table_name`, `index_name`, `columns`, `auto_created`, `created_by_constraint` | All indexes including system-created |
| `sys.constraints` | `db_id`, `schema_name`, `table_name`, `constraint_name`, `kind`, `enforcement_mode`, `trust_status`, `child_columns`, `parent_table`, `parent_columns`, `auto_index_name` | Constraint enforcement mode; deferral state |
| `sys.tablespaces` | `db_id`, `tablespace_name`, `location_path`, `is_default` | Filesystem placement per tablespace |

### `pg_catalog.*` (PostgreSQL-compatible schema views)

| View | PostgreSQL source | Coverage |
|---|---|---|
| `pg_catalog.pg_namespace` | `pg_namespace` | All schemas |
| `pg_catalog.pg_tables` | `pg_tables` | Tables + schema + owner |
| `pg_catalog.pg_indexes` | `pg_indexes` | Indexes + definition |
| `pg_catalog.pg_constraint` | `pg_constraint` | PK, FK, UNIQUE, CHECK constraints |
| `pg_catalog.pg_sequences` / `pg_sequence` | `pg_sequences` | All sequences |
| `pg_catalog.pg_database` | `pg_database` | Database list |
| `pg_catalog.pg_namespace` | `pg_namespace` | Namespace/schema list |
| `pg_catalog.pg_proc` | `pg_proc` | Function catalog (partial) |
| `pg_catalog.pg_roles` | `pg_roles` | Role list |
| `pg_catalog.pg_user` | `pg_user` | User list |
| `pg_catalog.pg_settings` | `pg_settings` | Configuration parameters |
| `pg_catalog.pg_index` | `pg_index` | Index metadata with OIDs |

---

## Layer B — Runtime State

These views answer: *"What is the system doing right now?"*

### Session and connection monitoring

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `angara_stat_activity` | `pid`, `datname`, `usename`, `state`, `query_start`, `query_fingerprint`, `consumer_id`, `wait_event`, `wait_event_type` | `pg_stat_activity` |
| `angara_stat_wait_events` | `event`, `event_type`, `total`, `active`, `total_duration_us` | `pg_stat_activity` (wait fields) |
| `sys.health` | `uptime_seconds`, `connections_active`, `connections_accepted_total`, `txn_commit_epoch_current`, `txlog_durable_lsn` | No single-view PG equivalent |
| `sys.identity` | `cluster_id`, `db_id`, `db_name`, `lease_holder_id`, `lease_expires_at`, `lease_acquired_at`, `lease_holder_hostname`, `recovery_mode` | No PG equivalent |

### Database-level statistics

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `angara_stat_database` | `datname`, `numbackends`, `xact_commit`, `xact_rollback`, `blks_read`, `blks_hit`, `conflicts` | `pg_stat_database` |
| `angara_stat_bgwriter` | `checkpoints_timed`, `checkpoints_req`, `checkpoint_write_time_ms`, `checkpoint_sync_time_ms`, `buffers_checkpoint`, `buffers_clean`, `buffers_backend`, `checkpoint_errors` | `pg_stat_bgwriter` |

### QoS and workload isolation state

**Unique to AngaraBase — no PostgreSQL equivalent.**

| View | Key columns | What it shows |
|---|---|---|
| `angara_stat_qos_queues` | `level` (critical/interactive/background), `queued_total`, `rejected_total`, `blocking_inflight` | Real-time queue depth and rejection rate per service level |
| `sys.gc_tuning_status` | `current_budget`, `last_cycle_duration_ms`, `tuning_decision`, `bloat_ratio_percent`, `min_active_epoch_lag`, `sleep_ms`, `decisions_increase_total` | UNDO log GC state — the "no VACUUM" control loop |

### Security and compliance

| View | Key columns | Use |
|---|---|---|
| `sys.roles` | `role_name` | All roles |
| `sys.user_roles` | `user_name`, `role_name`, `enabled` | User-role assignments |
| `sys.role_privileges` | `role_name`, `privilege`, `enabled` | Privilege matrix per role |
| `sys.audit_log` | `ts_unix_seconds`, `user_name`, `action`, `result`, `reason` | Structured audit trail |

---

## Layer C — Performance & Query Analytics

These views answer: *"How are queries performing and what does the optimizer know?"*

### Query execution statistics

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `angara_stat_statements` | `queryid`, `consumer_id`, `query`, `calls`, `total_exec_time_ms`, `min_exec_time_ms`, `max_exec_time_ms`, `mean_exec_time_ms`, `rows`, `shared_blks_hit`, `shared_blks_read`, `class` | `pg_stat_statements` (extension in PG, built-in here) |
| `angara_top_queries(N)` | `queryid`, `consumer_id`, `query`, `calls`, `total_exec_time_ms`, `rows` | `pg_stat_statements ORDER BY total_exec_time DESC LIMIT N` |

### Query Store — plan history and regression detection

**Unique to AngaraBase — analogous to SQL Server Query Store, not available in PostgreSQL without extensions.**

| View | Key columns | Purpose |
|---|---|---|
| `angara_query_store_entries` | `query_id`, `query_text`, `first_seen`, `last_seen`, `plan_count` | All tracked queries |
| `angara_query_store_plans` | `plan_id`, `query_id`, `plan_json`, `first_seen`, `last_seen`, `is_regressed` | Historical execution plans; flags regressed plans |
| `angara_query_store_intervals` | `query_id`, `plan_id`, `interval_start`, `interval_end`, `calls`, `mean_exec_time_ms`, `rows` | Per-plan performance over time |

### Table and index statistics

| View | Key columns | PostgreSQL equivalent |
|---|---|---|
| `sys.table_stats` | `seq_scan`, `idx_scan`, `tuples_read`, `tuples_written`, `row_count_estimate`, `row_count_live`, `dml_change_counter`, `last_committed_rowid`, `last_insert_epoch` | `pg_stat_user_tables` |
| `sys.index_stats` | `seeks`, `scans`, `tuples_read`, `cache_hit`, `cache_miss` | `pg_stat_user_indexes` |
| `sys.workload_stats` | `query_class`, `access_count`, `seq_scan`, `idx_scan`, `tuples_read`, `tuples_written` per table per workload class | **No PG equivalent** — breakdown by OLTP vs HTAP workload |
| `sys.column_stats` | `null_ppm`, `distinct_estimate`, `ndv_approx`, `col_min`, `col_max`, `histogram_bounds`, `mcv_values`, `mcv_frequencies`, `hll_enabled`, `reservoir_size` | `pg_stats` (partial) — **exposes full statistics state** |
| `sys.multicolumn_stats` | `col_a`, `col_b`, `correlation`, `ndv_joint`, `mcv_joint` | `pg_statistic_ext` (partial) |

### Adaptive Query Processing (AQP)

| View | Purpose |
|---|---|
| `sys.aqp_stats` | AQP feedback loop statistics |
| `sys.aqp_feedback` | Per-query cardinality feedback entries |
| `sys.aqp_blacklist` | Queries excluded from AQP re-optimization |

### Learned index models

**Unique — allows inspection and management of learned cardinality models.**

| View | Key columns |
|---|---|
| `sys.learned_models` | Model registry |
| `sys.learned_active_models` | Currently active models by table |
| `sys.learned_model_stats` | Per-model accuracy and usage |

### Stream / event bus monitoring

| View | Key columns | Use |
|---|---|---|
| `sys.stream_subscriptions` | Subscription list per channel | Monitor active LISTEN/NOTIFY subscribers |
| `sys.stream_stats` | `table_name`, `log_size`, `oldest_offset` | Event log depth per stream |

---

## Layer D — Configuration and Metrics

| View | Key columns | Use |
|---|---|---|
| `sys.settings` / `sys.knobs` | `name`, `value`, `source`, `dynamic`, `doc` | All configuration parameters with docs |
| `sys.settings_meta` | `name`, `scope`, `dynamic`, `restart_required`, `sensitive` | Parameter metadata (restart required? sensitive?) |
| `sys.metrics` | `name`, `value` | All Prometheus metrics accessible via SQL |
| `pg_catalog.pg_settings` | `name`, `setting`, `category` | PostgreSQL-compatible settings view |

---

## Cookbook — 20 essential queries

### Schema discovery

```sql
-- 1. List all tables with their storage engine and estimated row count
SELECT schema_name, table_name, storage_engine, durability, row_count_estimate
FROM sys.tables
ORDER BY schema_name, table_name;

-- 2. Find all indexes on a specific table
SELECT index_name, columns, auto_created, created_by_constraint
FROM sys.indexes
WHERE schema_name = 'public' AND table_name = 'orders';

-- 3. List all foreign key constraints with their targets
SELECT table_name, constraint_name, child_columns, parent_table, parent_columns, enforcement_mode
FROM sys.constraints
WHERE kind = 'FK'
ORDER BY table_name;

-- 4. Show all tablespaces and their filesystem paths
SELECT tablespace_name, location_path, is_default
FROM sys.tablespaces;

-- 5. Portable schema inspection (works with any PostgreSQL tool)
SELECT table_schema, table_name, column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_schema = 'public'
ORDER BY table_name, ordinal_position;
```

### Runtime state

```sql
-- 6. Active sessions with wait state
SELECT pid, datname, usename, state, wait_event_type, wait_event, query_fingerprint
FROM angara_stat_activity
WHERE state = 'active';

-- 7. Top wait events right now
SELECT event, event_type, active, total_duration_us
FROM angara_stat_wait_events
WHERE active > 0
ORDER BY active DESC;

-- 8. Instance health snapshot (single row)
SELECT uptime_seconds, connections_active, txn_commit_epoch_current, txlog_durable_lsn
FROM sys.health;

-- 9. Cluster identity and lease state (HA diagnostics)
SELECT cluster_id, db_name, lease_holder_id, lease_expires_at, recovery_mode
FROM sys.identity;

-- 10. QoS queue state: are any service levels being throttled?
SELECT level, queued_total, rejected_total, blocking_inflight
FROM angara_stat_qos_queues
WHERE rejected_total > 0 OR blocking_inflight > 0;
```

### Performance analysis

```sql
-- 11. Top 10 queries by total execution time
SELECT * FROM angara_top_queries(10);

-- 12. Queries slower than 100ms on average
SELECT queryid, query, calls, mean_exec_time_ms, class
FROM angara_stat_statements
WHERE mean_exec_time_ms > 100
ORDER BY mean_exec_time_ms DESC
LIMIT 20;

-- 13. Tables with high seq_scan / idx_scan ratio (missing indexes?)
SELECT schema_name, table_name, seq_scan, idx_scan,
       CASE WHEN idx_scan = 0 THEN 'no index scans'
            ELSE CAST(seq_scan * 100 / (seq_scan + idx_scan) AS text) || '% seq'
       END AS scan_profile
FROM sys.table_stats
WHERE seq_scan > 100
ORDER BY seq_scan DESC;

-- 14. Index hit rate per index (cache efficiency)
SELECT schema_name, table_name, index_name,
       cache_hit, cache_miss,
       CASE WHEN cache_hit + cache_miss = 0 THEN NULL
            ELSE CAST(cache_hit * 100 / (cache_hit + cache_miss) AS text) || '%'
       END AS hit_rate
FROM sys.index_stats
ORDER BY cache_miss DESC;

-- 15. OLTP vs analytics workload split per table
SELECT schema_name, table_name, query_class,
       access_count, tuples_read, tuples_written
FROM sys.workload_stats
ORDER BY schema_name, table_name, query_class;
```

### Query plan history

```sql
-- 16. Queries with regressed plans (needs intervention)
SELECT e.query_text, p.plan_id, p.first_seen, p.last_seen, p.is_regressed
FROM angara_query_store_entries e
JOIN angara_query_store_plans p ON p.query_id = e.query_id
WHERE p.is_regressed = true;

-- 17. Performance of a specific plan over time
SELECT interval_start, interval_end, calls, mean_exec_time_ms, rows
FROM angara_query_store_intervals
WHERE query_id = 12345  -- from angara_query_store_entries
ORDER BY interval_start DESC
LIMIT 48;
```

### UNDO log and GC state

```sql
-- 18. UNDO log health (the "no VACUUM" control loop)
SELECT current_budget, bloat_ratio_percent, tuning_decision,
       min_active_epoch_lag, decisions_increase_total, decisions_decrease_total
FROM sys.gc_tuning_status;
```

### Configuration and settings

```sql
-- 19. Show all dynamic settings that can be changed without restart
SELECT name, value, scope, doc
FROM sys.settings_meta
JOIN sys.settings USING (name)
WHERE dynamic = true
ORDER BY name;

-- 20. Show current service_level and related knobs
SELECT name, value
FROM sys.settings
WHERE name LIKE '%service_level%' OR name LIKE '%workload%' OR name LIKE '%qos%';
```

---

## What AngaraBase exposes that PostgreSQL doesn't

| Capability | AngaraBase | PostgreSQL 18 |
|---|---|---|
| Storage engine per table | `sys.tables.storage_engine` (heap/memory/column) | Not applicable — single engine |
| Durability tier per table | `sys.tables.durability` | Not applicable |
| QoS queue depth by service level | `angara_stat_qos_queues` | ❌ No equivalent |
| UNDO log GC control loop state | `sys.gc_tuning_status` | ❌ VACUUM replaces this |
| Workload-class breakdown per table | `sys.workload_stats` | ❌ No per-class split |
| Full column statistics state | `sys.column_stats` — HLL, histograms, MCV, reservoir | `pg_stats` — partial read-only |
| Plan history + regression flags | `angara_query_store_*` | `pg_stat_statements` — no plan history |
| Learned index model registry | `sys.learned_models`, `sys.learned_active_models` | ❌ No equivalent |
| Event stream monitoring | `sys.stream_subscriptions`, `sys.stream_stats` | ❌ No equivalent |
| Cluster identity + lease state | `sys.identity` | ❌ No equivalent in single-node PG |
| Constraint enforcement mode | `sys.constraints.enforcement_mode`, `trust_status` | `pg_constraint` — mode only |
| Tablespace to filesystem path | `sys.tablespaces.location_path` | `pg_tablespace` — similar |
| All metrics via SQL | `sys.metrics` — any Prometheus counter/gauge | ❌ No built-in SQL bridge |

---

## What is not yet implemented

| PostgreSQL feature | Status |
|---|---|
| `pg_locks` — lock table with `granted`, `locktype` | 🔜 v0.7 (HA / multi-node prerequisite) |
| `pg_stat_replication` — replica lag per standby | ⚠️ Partial — replica lag via Prometheus metric |
| `pg_blocking_pids()` — blocking session graph | 🔜 v0.7 |
| `information_schema.referential_constraints` | 🔜 v0.7 |
| `information_schema.key_column_usage` | 🔜 v0.7 |
| `pg_stat_progress_*` — operation progress views | 🔜 v0.8 |

---

## Feature detection

ORM and tooling authors: use this query to detect AngaraBase and check capabilities.

```sql
-- Detect AngaraBase and query its introspection capabilities
SELECT name, value
FROM sys.settings
WHERE name IN ('server_version', 'angara_version', 'angara_build_sha');

-- Check if a specific view exists
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'sys' AND table_name = 'workload_stats';

-- Or query directly — returns empty result set (not an error) if view exists
SELECT * FROM sys.health LIMIT 0;
```

---

*See also: [`SQL_COMPATIBILITY.md`](SQL_COMPATIBILITY.md) — full SQL feature compatibility matrix.*
