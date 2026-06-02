
# Understand AngaraBase in 5 minutes — 15 queries

Connect with `psql` (or any PostgreSQL client) and run these queries in order.
No extensions. No special mode. Standard `pgwire`.

```bash
psql -h 127.0.0.1 -p 5432 -U angara mydb
```

---

## 1. What's running

```sql
-- Server version and build
SELECT name, value FROM sys.settings
WHERE name IN ('server_version', 'angara_version');

-- Uptime, active connections, commit epoch, WAL LSN
SELECT * FROM sys.health;
```

---

## 2. What tables exist — and what engine backs them

```sql
-- All tables with their storage engine and durability guarantee
SELECT schema_name, table_name, storage_engine, durability,
       row_count_estimate, tablespace_name
FROM sys.tables
WHERE schema_name NOT IN ('sys', 'pg_catalog', 'information_schema')
ORDER BY schema_name, table_name;
```

> `storage_engine` = `heap` (row store), `memory`, or `column`.
> `durability` = `full` (WAL + fsync), `logged` (WAL only), or `none` (in-memory only).
> These are contractual declarations made at `CREATE TABLE` time.

---

## 3. Indexes, constraints, and tablespaces

```sql
-- All indexes on your tables
SELECT schema_name, table_name, index_name, columns, created_by_constraint
FROM sys.indexes
WHERE schema_name = 'public'
ORDER BY table_name;

-- All constraints with enforcement mode
SELECT table_name, constraint_name, kind, enforcement_mode, child_columns, parent_table
FROM sys.constraints
WHERE schema_name = 'public';

-- Tablespaces and their filesystem paths
SELECT tablespace_name, location_path, is_default FROM sys.tablespaces;
```

---

## 4. Active sessions and what they are waiting on

```sql
-- Active sessions right now
SELECT pid, usename, state, wait_event_type, wait_event, query_fingerprint
FROM angara_stat_activity
WHERE state = 'active';

-- Top wait events (across all sessions)
SELECT event, event_type, active, total_duration_us
FROM angara_stat_wait_events
ORDER BY active DESC, total_duration_us DESC
LIMIT 10;
```

---

## 5. Is OLTP being impacted by analytics? (QoS contract check)

```sql
-- Are any service levels being throttled or rejected?
SELECT level, queued_total, rejected_total, blocking_inflight
FROM angara_stat_qos_queues;
```

> `rejected_total > 0` for a level means that workload hit its resource contract
> and was turned away with a `SQLSTATE` error — the other levels were unaffected.
> This is the observable evidence of the fail-closed isolation guarantee.

---

## 6. Top queries by execution time

```sql
-- Slowest queries by total time
SELECT * FROM angara_top_queries(10);

-- With per-call averages
SELECT queryid, calls, mean_exec_time_ms, class, LEFT(query, 80) AS query_short
FROM angara_stat_statements
ORDER BY mean_exec_time_ms DESC
LIMIT 20;
```

---

## 7. Table access patterns — seq scans vs index scans

```sql
-- Tables with high sequential scan ratio (potential missing indexes)
SELECT schema_name, table_name,
       seq_scan, idx_scan,
       row_count_estimate
FROM sys.table_stats
WHERE seq_scan > 100
ORDER BY seq_scan DESC
LIMIT 20;
```

---

## 8. OLTP vs analytics workload split per table

```sql
-- How is each table accessed by workload class?
SELECT table_name, query_class, access_count, tuples_read, tuples_written
FROM sys.workload_stats
WHERE schema_name = 'public'
ORDER BY table_name, query_class;
```

> `query_class` distinguishes OLTP and analytical access patterns.
> No equivalent exists in PostgreSQL.

---

## 9. The "No VACUUM" control loop — is GC keeping up?

```sql
-- UNDO log GC state
SELECT current_budget, bloat_ratio_percent, tuning_decision,
       min_active_epoch_lag, sleep_ms,
       decisions_increase_total, decisions_decrease_total
FROM sys.gc_tuning_status;
```

> `tuning_decision` = `increase` / `decrease` / `hold` — what the GC loop decided last cycle.
> `bloat_ratio_percent` near 100 means the UNDO log is nearing its budget — worth monitoring.

---

## 10. Query plan history — catch regressions before users do

```sql
-- Queries with detected plan regressions
SELECT e.query_text, p.plan_id, p.first_seen, p.last_seen
FROM angara_query_store_entries e
JOIN angara_query_store_plans p ON p.query_id = e.query_id
WHERE p.is_regressed = true
ORDER BY p.last_seen DESC;

-- All plans for a specific query
SELECT plan_id, first_seen, last_seen, is_regressed
FROM angara_query_store_plans
WHERE query_id = (
    SELECT query_id FROM angara_query_store_entries
    WHERE query_text LIKE '%orders%'
    LIMIT 1
);
```

---

## 11. Column statistics — what does the planner know?

```sql
-- Statistics quality for a table's columns
SELECT column_name, distinct_estimate, null_ppm,
       hll_enabled, reservoir_size, stats_epoch,
       col_min, col_max
FROM sys.column_stats
WHERE schema_name = 'public' AND table_name = 'orders'
ORDER BY column_name;
```

> `hll_enabled = true` means HyperLogLog sketch is active for NDV estimation.
> `reservoir_size` shows how many sampled rows back the statistics.

---

## 12. Cluster identity (HA diagnostics)

```sql
-- Who holds the lease? Is the instance in recovery mode?
SELECT cluster_id, db_name, lease_holder_id,
       lease_expires_at, recovery_mode
FROM sys.identity;
```

---

## 13. All metrics via SQL — no Prometheus scrape needed

```sql
-- Live resource contract counters
SELECT name, value FROM sys.metrics
WHERE name LIKE '%undo%'
   OR name LIKE '%admission%'
   OR name LIKE '%qos%'
ORDER BY name;

-- Any counter that has fired (non-zero)
SELECT name, value FROM sys.metrics
WHERE value > 0 AND name LIKE '%error%'
ORDER BY value DESC
LIMIT 20;
```

---

## 14. Configuration knobs — what can change without restart?

```sql
-- Dynamic settings (no restart required)
SELECT s.name, s.value, m.scope, m.doc
FROM sys.settings s
JOIN sys.settings_meta m USING (name)
WHERE m.dynamic = true
ORDER BY s.name;
```

---

## 15. Portable schema inspection (for ORMs and migration tools)

```sql
-- Standard SQL:2016 — works identically on PostgreSQL
SELECT table_schema, table_name, column_name,
       data_type, is_nullable, column_default
FROM information_schema.columns
WHERE table_schema = 'public'
ORDER BY table_name, ordinal_position;
```

---

## Terminology reference

| Term | What it means |
|---|---|
| **Portable SQL Metadata** | `information_schema.*` — SQL:2016 standard, works on any RDBMS |
| **PostgreSQL-Compatible Catalog** | `pg_catalog.*` — drop-in for pg tooling; psql, pgAdmin, Flyway |
| **Runtime Observability Views** | `angara_stat_*` — live session, wait, QoS, checkpoint state |
| **Engine Contract Views** | `sys.*` views exposing per-table engine guarantees — storage engine, durability, GC state, workload isolation |

---

*Full reference with all columns and cookbook:* [`INTROSPECTION.md`](INTROSPECTION.md)
