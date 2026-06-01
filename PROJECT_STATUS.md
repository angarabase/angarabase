
# AngaraBase — Project Status

AngaraBase is under **active development**. Pre-release software — APIs and on-disk formats
may change between minor versions.

---

## Current state

| Field | Value |
|---|---|
| **Version** | `0.6.7` |
| **Status** | Active development |
| **Platform** | Linux · x86_64 / aarch64 |
| **Wire protocol** | PostgreSQL pgwire v3 |
| **License** | Business Source License 1.1 (see `LICENSE`) |

---

## What we are working on

Current focus: **Advanced Security, Observability Polish & Multi-Version Phase 0** —
SCRAM-SHA-256 hardening, query-level wait-event attribution, and the first phase of
multi-version page format for the columnar storage layer.

Detailed technical roadmap is maintained internally; major milestones are announced
via [GitHub Releases](../../releases) and the [angarabase.dev](https://angarabase.dev)
documentation portal.

---

## Recent milestones

| Version | Highlight |
|---|---|
| **0.6.7** | IndexStore physical move to `engine-storage`; GcWorker & CheckpointWorker migration; WAL crash-safety |
| **0.6.6** | Columnar SELECT native execution path end-to-end: zone maps, CRC32C, ManifestRegistry, batch accumulation |
| **0.6.5** | SQL semantic correctness (ORDER BY alias/ordinal, type safety); monolith decomposition — `engine-sql` split |
| **0.6.4** | Write-path performance recovery: B-tree insert fast-path, index GC, memory cap, MVCC quick-wins |
| **0.6.3** | `REINDEX CONCURRENTLY`; HTAP perf-pack harness |
| **0.6.2** | Version-Aware IndexStore v2 (MV-PBT style): on-disk leaf format, visibility check, IOS integration |
| **0.6.1** | UNDO write-amplification reduction: HOT++ skip-index path, UPDATE-batch planner hint, per-table UNDO retention |
| **0.6.0** | UNDO MVCC full transition; per-DB MVCC; legacy heap elimination; ARIES undo pass |

Each release ships a signed tarball, SHA-256 checksums, and a pinned benchmark evidence
pack. See [RELEASES.md](RELEASES.md) for verification instructions.

---

## Known limitations (pre-v1)

- **Linux only.** No Windows or macOS server binary.
- **No global distributed mode.** Single-node and streaming replication only in current series;
  transparent sharding is a v0.8+ horizon item.
- **No user-defined functions (UDF).** Planned for a future minor.
- **No full PostgreSQL extension ecosystem.** pgwire-compatible but not a PostgreSQL fork.
- **Pre-release on-disk format.** Minor-version format changes are possible; migration tooling
  is provided in the release notes when formats change.

---

## Installation

Binary packages and verification instructions: [PACKAGES.md](PACKAGES.md)  
All signed releases: [GitHub Releases](../../releases)

---

## Documentation

Full documentation and runbooks: [**angarabase.dev**](https://angarabase.dev)  
Project website: [**angarabase.com**](https://angarabase.com)
