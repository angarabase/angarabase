# AngaraBase Roadmap

This document describes the high-level direction for AngaraBase.
Detailed technical train plans are maintained internally and announced via
[Releases](../../releases) and [angarabase.dev](https://angarabase.dev).

---

## Now — v0.6.x · Dev Preview

AngaraBase is under **active development**. The core engine is built and working;
we are hardening it for production use.

**Already shipped:**

| Area | Status |
|---|---|
| UNDO-log MVCC (no VACUUM, no heap bloat) | ✅ delivered |
| ARIES crash recovery (Analysis → Redo → Undo) | ✅ delivered |
| B-tree IndexStore (MVCC-aware, online build) | ✅ delivered |
| AngaraColumn — columnar storage engine (HTAP) | ✅ delivered |
| PostgreSQL pgwire v3 protocol | ✅ delivered |
| SCRAM-SHA-256 authentication | ✅ delivered |
| Streaming replication (sync / async) | ✅ delivered |
| AngaraStream Phase 0 — built-in event bus | ✅ delivered |
| Prometheus metrics + structured observability | ✅ delivered |
| Fail-closed resource contracts (8 named limits, each with a `SQLSTATE`) | ✅ delivered |

**Current focus (v0.6.x trains):**
Advanced security, columnar multi-version page format, query-level wait-event attribution,
WAL crash-safety hardening, monolith decomposition into engine subcrates.

---

## Next — v0.7.x · Production-Ready Single Node + Open Beta

Planned major release. Entry criteria: 60–70% closure of v0.6.x trains.

**Key themes:**

### SQL Completeness & Optimizer
- Extended Query Protocol — named portals, server-side cursors, COPY binary, prepared-statement cache
- Native JSONB storage (columnar-aware, GIN index in v0.8)
- CBO Phase 2 — bushy join trees, expression-level statistics, decorrelated subqueries
- Foreign key enforcement (`DEFERRABLE INITIALLY DEFERRED`)
- Vector type — basic `float32[]` with distance functions (HNSW/IVFFlat in v0.8)
- Index Advisor — hypothetical index analysis integrated with the cost model

### AngaraFunc — Compiled UDF & WASM Sandbox
- SQL UDFs compiled to native code (not interpreted), integrated with the query planner
- WASM-isolated external functions for polyglot UDFs (Rust, C, C++, Go)
- Stored generated columns, trigger foundation

### Async I/O & Adaptive Tuning
- Full `io_uring`-based async I/O for the storage layer (RFC-461)
- Memory Arbitrator — runtime memory budget arbitration across query operators
- AngaraTuner Phase 0 — MemoryAdvisor (automatic `shared_buffers` / `work_mem` guidance)

### Cloud-Native (AngaraCloud)
- Tiered storage: hot data on local NVMe, warm/cold data on S3-compatible object store
- Kubernetes-native deployment (operator, liveness/readiness probes, PVC lifecycle)

### High Availability (AngaraHA)
- Auto-failover with Raft-based leader election (no external coordinator required)
- Chaos engineering test harness (network partition, node crash drills)

### CDC & Event Bus
- Event Bus Phase 2 — WAL logical decoding, per-row CDC with delivery guarantees
- External connectors: Kafka, NATS

### Open Beta gate
- SQL conformance test suite (pgregress subset)
- Public Open Beta launch (AngaraBase Community Edition preview)

---

## v0.8.x · Single-Node Hardening & GA Preparation

**Key themes:**
- Performance benchmarking vs. PostgreSQL on OLTP + HTAP workloads; published results
- GIN Index (for JSONB / full-text)
- HNSW / IVFFlat vector indexes (AI/ML workloads)
- Full temporal table support, column masking
- Async runtime completion (full `io_uring` migration)
- Tech debt closure; final production hardening

---

## v0.9.x · Public GA & Scale-Out

**Key themes:**
- **Public General Availability (GA)** — stable on-disk format, migration tooling, SLA commitments
- **Community Edition launch** — binary distribution with license key, full feature set, size limits
- **Transparent horizontal sharding** — automatic data partitioning across nodes without application changes
- **AngaraKey** — automated license key issuance (offline-verifiable, air-gap friendly)

---

## Licensing roadmap

| Stage | License | Distribution |
|---|---|---|
| Dev Preview (now) | BUSL-1.1 → Apache 2.0 by 2030 | Source in release package |
| Open Beta (v0.7) | BUSL-1.1 | Binary + Community preview key |
| GA (v0.9+) | AngaraBase Community License | Binary, license key auto-issued |
| Commercial / Enterprise | Commercial license | Source access + SLA |

The **Community Edition** (v0.9+) will be free, require a license key (auto-issued,
annually renewable, fully offline-verifiable), and include the full feature set
with instance-size limits. Government and educational institutions can request
extended limits. See [LICENSE](LICENSE) for current terms.

---

## What is not on this roadmap

- **Windows / macOS server** — Linux-only by design; no plans to change this.
- **PostgreSQL fork or extension ecosystem** — pgwire-compatible, not a PostgreSQL fork;
  third-party PG extensions will not work.
- **Managed cloud service** — self-hosted Linux only. A hosted offering is not currently planned.

---

## How to follow progress

- **[GitHub Releases](../../releases)** — signed binaries + evidence packs after each train
- **[Discussions](../../discussions)** — roadmap Q&A, use cases, design partner program
- **[angarabase.dev](https://angarabase.dev)** — full documentation
- **[angarabase.com](https://angarabase.com)** — project website and announcements
