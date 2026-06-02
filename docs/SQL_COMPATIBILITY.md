
# AngaraBase — PostgreSQL Compatibility Matrix

AngaraBase is **PostgreSQL-wire-compatible**, not a PostgreSQL fork. It speaks `pgwire v3`
and understands a well-defined SQL subset. This document is the canonical compatibility
reference: what works today, what is partial, and what is on the roadmap.

**Legend**

| Symbol | Meaning |
|---|---|
| ✅ | Implemented and tested |
| ⚠️ | Partial — works for common cases, see notes |
| 🔜 | Planned (version indicated) |
| ❌ | Not planned / excluded by design |

For the full SQL reference, runbooks and known edge cases:
[angarabase.dev → SQL Reference](https://angarabase.dev/sql-reference/)

---

## Wire Protocol & Drivers

| Feature | Status | Notes |
|---|:---:|---|
| PostgreSQL pgwire v3 | ✅ | Simple Query + basic Extended Query |
| `psql` (any version) | ✅ | Tested with psql 13–16 |
| JDBC / `pgjdbc` | ✅ | Standard `jdbc:postgresql://` URL |
| `libpq` (C) | ✅ | |
| `pgx` (Go) | ✅ | |
| `asyncpg` (Python) | ✅ | |
| `psycopg2` / `psycopg3` | ✅ | |
| `node-postgres` (`pg`) | ✅ | |
| Named portals & server-side cursors | 🔜 v0.7 | Extended Query Protocol Phase 2 |
| COPY text format | ✅ | `COPY … FROM STDIN` / `TO STDOUT` |
| COPY binary format | 🔜 v0.7 | |
| Pipelining (multiple queries per message) | ⚠️ | Basic pipelining; not battle-tested |
| SSL/TLS connections | ✅ | |
| SCRAM-SHA-256 authentication | ✅ | Default and recommended |
| MD5 authentication | ⚠️ | Accepted for compatibility; SCRAM preferred |
| Trust (no password) | ✅ | Local / loopback only |

---

## Transactions & Isolation

| Feature | Status | Notes |
|---|:---:|---|
| `BEGIN` / `COMMIT` / `ROLLBACK` | ✅ | |
| `SAVEPOINT` / `ROLLBACK TO SAVEPOINT` / `RELEASE SAVEPOINT` | ✅ | |
| `SET TRANSACTION ISOLATION LEVEL` | ✅ | |
| **READ COMMITTED** (default) | ✅ | |
| **REPEATABLE READ** | ✅ | Snapshot isolation via UNDO-log MVCC |
| **SERIALIZABLE** | ✅ | SSI with write-conflict detection |
| READ UNCOMMITTED | ⚠️ | Accepted; mapped to READ COMMITTED |
| MVCC — no blocking readers | ✅ | UNDO-log model: writers never block readers |
| Long-running transactions | ✅ | UNDO space is bounded by contract (`SQLSTATE 56U01`) |
| Deadlock detection | ✅ | |
| Implicit transaction (autocommit) | ✅ | |
| Two-phase commit (`PREPARE TRANSACTION`) | 🔜 v0.8 | |

---

## DDL — Tables & Schemas

| Feature | Status | Notes |
|---|:---:|---|
| `CREATE TABLE` | ✅ | |
| `CREATE TABLE … AS SELECT` | ✅ | |
| `ALTER TABLE … ADD / DROP / ALTER COLUMN` | ✅ | |
| `ALTER TABLE … RENAME` | ✅ | |
| `DROP TABLE` | ✅ | `IF EXISTS`, `CASCADE` |
| `TRUNCATE` | ✅ | |
| Temporary tables (`CREATE TEMP TABLE`) | ✅ | Session-scoped |
| Unlogged tables | ⚠️ | Accepted; treated as logged in current release |
| Partitioned tables (`PARTITION BY`) | 🔜 v0.8 | |
| Table inheritance | ❌ | Not planned |
| `CREATE SCHEMA` | ✅ | |
| `DROP SCHEMA` | ✅ | |
| `SET search_path` | ✅ | |
| `CREATE DATABASE` / `DROP DATABASE` | ✅ | |

---

## DDL — Indexes

| Feature | Status | Notes |
|---|:---:|---|
| B-tree index | ✅ | MVCC-aware; online build |
| `CREATE INDEX` | ✅ | |
| `CREATE INDEX CONCURRENTLY` | ✅ | Online, non-blocking |
| `DROP INDEX` / `REINDEX CONCURRENTLY` | ✅ | |
| Composite indexes | ✅ | |
| Partial indexes (`WHERE` clause) | ✅ | |
| Expression indexes | ⚠️ | Simple expressions; complex cases limited |
| GIN index (JSONB / full-text) | 🔜 v0.8 | |
| GiST index | 🔜 v0.9+ | |
| BRIN index | ❌ | Not planned short-term |
| Hash index | ❌ | B-tree covers all use cases; hash not prioritized |
| Vector index (HNSW / IVFFlat) | 🔜 v0.8 | |

---

## DDL — Sequences & Identity

| Feature | Status | Notes |
|---|:---:|---|
| `CREATE SEQUENCE` / `DROP SEQUENCE` | ✅ | |
| `SERIAL` / `BIGSERIAL` / `SMALLSERIAL` | ✅ | |
| `IDENTITY` columns (`GENERATED … AS IDENTITY`) | ⚠️ | Syntax accepted; behavior matches SERIAL |
| `nextval()` / `currval()` / `setval()` | ✅ | |
| Sequence in DDL expressions | ✅ | |

---

## DDL — Constraints

| Feature | Status | Notes |
|---|:---:|---|
| `PRIMARY KEY` | ✅ | |
| `UNIQUE` | ✅ | |
| `NOT NULL` | ✅ | |
| `DEFAULT` | ✅ | |
| `CHECK` | ✅ | |
| `FOREIGN KEY` (`REFERENCES`) | ✅ | Enforcement; `ON DELETE CASCADE` / `SET NULL` |
| `DEFERRABLE INITIALLY DEFERRED` | 🔜 v0.7 | |
| `EXCLUDE` constraints | ❌ | Not planned |

---

## DDL — Views & Procedures

| Feature | Status | Notes |
|---|:---:|---|
| `CREATE VIEW` | ✅ | Non-materialized |
| `DROP VIEW` | ✅ | |
| Materialized views | 🔜 v0.8 | |
| `CREATE FUNCTION` (SQL-bodied) | 🔜 v0.7 | AngaraFunc: native compiled UDFs |
| `CREATE FUNCTION` (PL/pgSQL) | ❌ | Not planned; use SQL UDFs or WASM UDFs |
| Stored procedures (`CREATE PROCEDURE`) | 🔜 v0.8 | |
| Triggers | 🔜 v0.7 | Foundation in v0.7 |
| Rules | ❌ | Not planned |

---

## DML — Core

| Feature | Status | Notes |
|---|:---:|---|
| `SELECT` | ✅ | |
| `INSERT` | ✅ | |
| `UPDATE` | ✅ | |
| `DELETE` | ✅ | |
| `INSERT … ON CONFLICT DO NOTHING` | ✅ | Upsert (ignore) |
| `INSERT … ON CONFLICT DO UPDATE` | ✅ | Upsert (merge) |
| `RETURNING` clause | ✅ | On INSERT / UPDATE / DELETE |
| `COPY … FROM / TO` | ✅ | Text format |
| `MERGE` | 🔜 v0.8 | |

---

## Query Features

| Feature | Status | Notes |
|---|:---:|---|
| `JOIN`: INNER, LEFT, RIGHT, FULL, CROSS | ✅ | Hash join, merge join, nested loop |
| Self-join | ✅ | |
| Correlated subqueries | ✅ | |
| Non-correlated subqueries | ✅ | |
| `EXISTS` / `NOT EXISTS` | ✅ | |
| `IN` / `NOT IN` subquery | ✅ | |
| `ANY` / `ALL` | ✅ | |
| `LATERAL` subquery | ⚠️ | Basic lateral; complex cases limited |
| `WITH` (CTEs, non-recursive) | ✅ | |
| `WITH RECURSIVE` | ⚠️ | Supported; depth limit applies |
| Window functions (`OVER`, `PARTITION BY`, `RANK`, `ROW_NUMBER`, `LAG`, `LEAD`, …) | ✅ | |
| `GROUP BY` / `HAVING` | ✅ | |
| `GROUP BY ROLLUP` / `CUBE` / `GROUPING SETS` | 🔜 v0.8 | |
| `DISTINCT` / `DISTINCT ON` | ✅ | |
| `ORDER BY` | ✅ | ASC / DESC / NULLS FIRST / NULLS LAST |
| `LIMIT` / `OFFSET` | ✅ | |
| `FETCH FIRST … ROWS ONLY` | ✅ | SQL standard form of LIMIT |
| `UNION` / `UNION ALL` / `INTERSECT` / `EXCEPT` | ✅ | Set operations |
| `EXPLAIN` | ✅ | |
| `EXPLAIN ANALYZE` | ✅ | With per-phase timing and operator stats |
| `EXPLAIN (FORMAT JSON)` | ✅ | |

---

## Data Types

| Type | Status | Notes |
|---|:---:|---|
| `BOOLEAN` | ✅ | |
| `SMALLINT`, `INTEGER`, `BIGINT` | ✅ | |
| `REAL`, `DOUBLE PRECISION` | ✅ | |
| `NUMERIC(p, s)` / `DECIMAL` | ✅ | Arbitrary precision |
| `TEXT`, `VARCHAR(n)`, `CHAR(n)` | ✅ | |
| `BYTEA` | ✅ | |
| `DATE`, `TIME`, `TIMESTAMP`, `TIMESTAMPTZ` | ✅ | |
| `INTERVAL` | ⚠️ | Basic intervals; arithmetic partially limited |
| `UUID` | ✅ | |
| `JSONB` | 🔜 v0.7 | Columnar-aware storage + GIN in v0.8 |
| `JSON` | 🔜 v0.7 | |
| `ARRAY` types | ⚠️ | 1-D arrays; multi-dimensional limited |
| `ENUM` | ⚠️ | Basic; `ALTER TYPE ADD VALUE` limited |
| `OID` | ⚠️ | Exposed in system catalog; limited user-space use |
| `HSTORE` | ❌ | Extension-based; not planned |
| Composite types | 🔜 v0.8 | |
| Domain types (`CREATE DOMAIN`) | 🔜 v0.8 | |
| Range types (`TSRANGE`, etc.) | 🔜 v0.9+ | |
| `float32[]` (vector) | 🔜 v0.7 | Basic HNSW/IVFFlat in v0.8 |

---

## Built-in Functions

| Category | Status | Notes |
|---|:---:|---|
| String functions (`length`, `substring`, `trim`, `upper`, `lower`, `concat`, …) | ✅ | |
| Math functions (`abs`, `ceil`, `floor`, `round`, `sqrt`, `mod`, …) | ✅ | |
| Date/time functions (`now()`, `current_date`, `date_trunc`, `extract`, `age`, …) | ✅ | |
| Aggregate functions (`count`, `sum`, `avg`, `min`, `max`, `stddev`, `variance`, …) | ✅ | |
| Conditional expressions (`CASE`, `COALESCE`, `NULLIF`, `GREATEST`, `LEAST`) | ✅ | |
| Type casting (`::type`, `CAST(… AS …)`) | ✅ | |
| `generate_series()` | ✅ | |
| `pg_typeof()` | ✅ | |
| Regex operators (`~`, `~*`, `!~`, `!~*`, `SIMILAR TO`) | ✅ | |
| `LIKE` / `ILIKE` | ✅ | |
| `json_*` / `jsonb_*` functions | 🔜 v0.7 | |
| `to_tsvector` / `plainto_tsquery` (full-text) | 🔜 v0.8 | |
| Custom functions (user-defined) | 🔜 v0.7 | AngaraFunc |

---

## Event Bus & Pub/Sub

| Feature | Status | Notes |
|---|:---:|---|
| `LISTEN channel` | ✅ | Per-channel sharded dispatcher |
| `NOTIFY channel [, payload]` | ✅ | Transactional: deferred to COMMIT |
| `UNLISTEN channel` / `UNLISTEN *` | ✅ | |
| Delivery guarantee | ✅ | At-most-once (ephemeral, per RFC-2026-239) |
| AngaraStream (WAL-based CDC) | 🔜 v0.7 | Durable, at-least-once; Kafka/NATS connectors |

---

## Replication & HA

| Feature | Status | Notes |
|---|:---:|---|
| Streaming replication (sync / async) | ✅ | Physical WAL streaming |
| Replica lag monitoring | ✅ | Prometheus metric |
| Manual failover | ✅ | |
| Auto-failover (Raft leader election) | 🔜 v0.7 | No external coordinator required |
| Logical replication / CDC | 🔜 v0.7 | WAL decoding; Kafka, NATS connectors |
| Point-in-time recovery (PITR) | ✅ | Via ARIES recovery + WAL |
| `pg_basebackup`-compatible | ⚠️ | Partial; use native backup tooling |

---

## System Catalog & Observability

> **Full introspection reference:** [`docs/INTROSPECTION.md`](INTROSPECTION.md) — view map, column lists, 20 SQL cookbook queries, and comparison with PostgreSQL.

AngaraBase provides **three introspection namespaces** queryable via standard `pgwire`:

### `pg_catalog.*` — PostgreSQL-compatible

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
| `pg_catalog.pg_index` | ✅ | |
| `pg_catalog.pg_proc` | ⚠️ | Partial — built-in functions only |
| `pg_catalog.pg_stat_activity` | ✅ | Via `angara_stat_activity` alias |
| `pg_catalog.pg_stat_user_tables` | ⚠️ | Partial — via `sys.table_stats` |
| `pg_catalog.pg_locks` | 🔜 v0.7 | Requires HA / multi-node |
| `pg_catalog.pg_stat_replication` | ⚠️ | Partial — via Prometheus metric |
| `pg_catalog.pg_stat_progress_*` | 🔜 v0.8 | Progress views |

### `information_schema.*` — SQL-standard portable

| View | Status | Notes |
|---|:---:|---|
| `information_schema.tables` | ✅ | |
| `information_schema.columns` | ✅ | |
| `information_schema.constraint_column_usage` | ✅ | |
| `information_schema.referential_constraints` | 🔜 v0.7 | |
| `information_schema.key_column_usage` | 🔜 v0.7 | |

### `sys.*` and `angara_stat_*` — AngaraBase-native

| View | Status | PostgreSQL equivalent |
|---|:---:|---|
| `sys.databases`, `sys.schemas`, `sys.tables` | ✅ | `pg_database`, `pg_namespace`, `pg_tables` |
| `sys.columns`, `sys.indexes`, `sys.constraints` | ✅ | `information_schema` |
| `sys.tablespaces` | ✅ | `pg_tablespace` |
| `sys.health` | ✅ | No single-view equivalent |
| `sys.identity` | ✅ | No equivalent (lease/cluster info) |
| `sys.settings` / `sys.settings_meta` | ✅ | `pg_settings` |
| `sys.table_stats`, `sys.index_stats` | ✅ | `pg_stat_user_tables/indexes` |
| `sys.column_stats`, `sys.multicolumn_stats` | ✅ | `pg_stats` (partial) |
| `sys.workload_stats` | ✅ | **No equivalent** — per-workload-class breakdown |
| `sys.gc_tuning_status` | ✅ | **No equivalent** — UNDO log GC state |
| `sys.roles`, `sys.user_roles`, `sys.role_privileges` | ✅ | `pg_roles` (partial) |
| `sys.audit_log` | ✅ | **No equivalent** — structured audit trail |
| `angara_stat_activity` | ✅ | `pg_stat_activity` |
| `angara_stat_statements` | ✅ | `pg_stat_statements` (built-in, no extension needed) |
| `angara_stat_wait_events` | ✅ | `pg_stat_activity` wait columns |
| `angara_stat_database` | ✅ | `pg_stat_database` |
| `angara_stat_bgwriter` | ✅ | `pg_stat_bgwriter` |
| `angara_stat_qos_queues` | ✅ | **No equivalent** — QoS service-level queue state |
| `angara_top_queries(N)` | ✅ | `pg_stat_statements ORDER BY … LIMIT N` |
| `angara_query_store_*` (entries/plans/intervals) | ✅ | **No equivalent** — plan history + regression flags |
| `sys.aqp_stats`, `sys.aqp_feedback`, `sys.aqp_blacklist` | ✅ | **No equivalent** — AQP loop state |
| `sys.learned_models`, `sys.learned_active_models` | ✅ | **No equivalent** — learned cardinality models |
| `sys.stream_subscriptions`, `sys.stream_stats` | ✅ | **No equivalent** — event stream monitoring |
| `sys.metrics` | ✅ | **No equivalent** — all Prometheus counters via SQL |

### Observability infrastructure

| Feature | Status | Notes |
|---|:---:|---|
| Prometheus metrics endpoint | ✅ | Every resource boundary has a named metric |
| USDT probes (`bpftrace` / `perf`) | ✅ | `probe_query_*`, `probe_phase_*`, `probe_operator_*` |
| Structured logs (stable field names) | ✅ | JSON-compatible |
| `EXPLAIN ANALYZE` with per-phase timing | ✅ | Plan + execution breakdown |

---

## Storage Engines

| Engine | Status | Notes |
|---|:---:|---|
| **Row store** (default) | ✅ | UNDO-log MVCC, B-tree indexes |
| **AngaraMemory** — `none` tier (no durability) | ✅ | Fastest; data lost on restart |
| **AngaraMemory** — `logged` tier (WAL-backed) | ✅ | Survives restart; good for hot working sets |
| **AngaraMemory** — `snapshotted` tier | ✅ | Periodic snapshot + WAL |
| **AngaraColumn** (columnar HTAP engine) | ✅ | SIMD-accelerated; zone maps; CRC32C integrity |
| Tiered storage (hot NVMe + S3 cold tier) | 🔜 v0.7 | AngaraCloud |

---

## Security

| Feature | Status | Notes |
|---|:---:|---|
| SCRAM-SHA-256 | ✅ | Default |
| Role-based access control (RBAC) | ✅ | `GRANT` / `REVOKE` |
| Row-level security (RLS) | 🔜 v0.7 | |
| Column-level masking | 🔜 v0.8 | |
| Audit log | 🔜 v0.7 | |
| Break-glass access | 🔜 v0.7 | |
| TDE (transparent data encryption) | 🔜 v0.8 | |
| SSL/TLS | ✅ | |

---

## Not supported — by design

These are explicitly **out of scope** and are not planned:

| Feature | Reason |
|---|---|
| PL/pgSQL | Use AngaraFunc (SQL UDFs + WASM sandbox, v0.7) |
| Third-party extensions (`pgvector`, `PostGIS`, …) | AngaraBase is not a PostgreSQL fork; extensions assume PG internals |
| Windows / macOS server binary | Linux-native by design; no plans to change |
| Managed / hosted cloud offering | Self-hosted only; cloud-native deployment via K8s operator (v0.7) |
| Table inheritance | Superseded by partitioning (v0.8 roadmap) |
| Rules (`CREATE RULE`) | Triggers (v0.7) cover the use cases |

---

## PostgreSQL version compatibility

AngaraBase speaks `pgwire v3` and reports server version **`0.6.x`**.
Client libraries that check the server version string should treat AngaraBase as
"PostgreSQL 14-compatible" for protocol negotiation purposes.

Known incompatibilities with PostgreSQL 14/15 behavior are tracked in
[angarabase.dev → Known Issues](https://angarabase.dev/reference/known-issues.html).

---

*Last updated: 2026-06. Maintained by the AngaraBase team.*
*Found a gap or error? Open an [issue](../../issues) tagged `docs`.*
