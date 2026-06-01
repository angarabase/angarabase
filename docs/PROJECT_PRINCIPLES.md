---
title: "AngaraBase — Project Principles"
language: en
doc_type: project_principles_public
public_mirror_of: docs/00_PROJECT_PRINCIPLES.md
sync_policy: "Update this file when the source RU file changes (new principle, revised principle, version bump)."
last_synced: 2026-06-01
source_version: v2.0 (2026-03-14)
---

# AngaraBase — Project Principles

**Version:** v2.0 · **Status:** current · **Owner:** Tech Lead

This document records the **design principles** of AngaraBase — no citations, no benchmarks.
It is the compass for decisions when a trade-off is disputed.

Each principle is a rule by which engineers and architects make decisions.
If a decision contradicts a principle, an RFC with justification is required.

---

## Contents

1. [Restrictive by Default — the architectural foundation](#1-restrictive-by-default)
2. [Every Guarantee Is Observable](#2-every-guarantee-is-observable)
3. [Degradation Modes Declared in Advance](#3-degradation-modes-declared-in-advance)
4. [Contract-First Development](#4-contract-first-development)
5. [Reliability and Correctness over "Magic"](#5-reliability-and-correctness-over-magic)
6. [Strategic Architecture Directions](#6-strategic-architecture-directions)
7. [Linux-only as a Strategic Choice](#7-linux-only-as-a-strategic-choice)
8. [Business Logic in the Application, DB as Data Engine](#8-business-logic-in-the-application)
9. [DBA and Developer Experience as a Product Feature](#9-dba-and-developer-experience)
10. [PostgreSQL Compatibility as a Bridge](#10-postgresql-compatibility-as-a-bridge)
11. [Performance-first (with caveats)](#11-performance-first)
12. [Modularity and Evolution](#12-modularity-and-evolution)
13. [Documentation and Process Discipline](#13-documentation-and-process-discipline)
14. [Frequent Small Changes (Controlled Risk)](#14-frequent-small-changes-controlled-risk)
15. [Minimal Dependencies and Supply Chain Security](#15-minimal-dependencies)
16. [What We Are Not Doing (yet)](#16-what-we-are-not-doing-yet)

---

## 1. Restrictive by Default

**AngaraBase only accepts work for which it has resources.**

Most databases operate on the model "accept everything until we crash." AngaraBase operates
on the inverse model: every component has explicit operational boundaries, and when those
boundaries are violated the system **rejects the request** (fail-closed) rather than silently
degrading (fail-open).

| Component | Boundary | On violation (fail-closed) | Caller reaction |
|---|---|---|---|
| BufferPool | `buffer_pool_size_mb` (RAM budget) | Eviction (CLOCK), WAL-first flush | StorageManager waits for I/O or hits OOM-guard |
| TxnWriteSet | `txn_max_write_set_mb` per txn | `Err(SQLSTATE=54023)` — reject DML | QueryPipeline aborts transaction, rollback |
| UndoStore | `undo_max_size_mb` disk budget | `Err(SQLSTATE=53100)` — reject writes | GC forces a sweep; DML waits or is rejected |
| AngaraMemory | `max_rows` per table | `Err` on INSERT beyond limit | Client receives error; retry will not help |
| AdmissionController | `max_concurrent_queries` | `AdmissionError::Overloaded` — fail-fast | Client receives 53300, applies exponential backoff |
| Connection pool | `max_connections` | Reject new connections | Load balancer redirects or client retries |
| Statement timeout | `statement_timeout_ms` | Cancel query | Client receives 57014; transaction rolls back |
| Snapshot age | `max_snapshot_age` | Force-close stale snapshots | Long-running query fails with 40001 (serialization failure) |

**Rule:** every new component MUST define its boundaries, its fail-closed behavior when those
boundaries are exceeded, and a **Reaction Propagation Contract** (how the calling code responds).
No boundaries = no merge.

---

## 2. Every Guarantee Is Observable

**Closing property: if a guarantee has no metrics or events, it exists only on paper.**

Every resource boundary from the table above MUST have:

1. **Prometheus metric** (counter/gauge) for monitoring utilization and rejections.
2. **Wait Event** if a boundary violation leads to a blocking wait.
3. **USDT probe point** for detailed tracing.

---

## 3. Degradation Modes Declared in Advance

**When designing a component, it must be specified: what is shed first? what is prioritized?**

The system must not degrade chaotically. Degradation modes must be documented in the
component's RFC. Example: under I/O pressure — first slow background GC, then limit new DML,
but DQL (reads) continue until the last possible moment.

---

## 4. Contract-First Development

In AngaraBase, a contract is a **Rust Trait** without which the code will not compile.

- Every subsystem has a public trait defining its semantics (`TableEngine`, `PageProvider`,
  `TransactionLogSink`, `StorageIo`).
- Invariants are captured in code (DbC style), in doc-comments, and in RFCs.
- Significant decisions go through RFC. An RFC is a contract against which implementation
  is built.
- Type system as enforcement: `Result<T, Error>` instead of panic; bounded generics
  instead of dynamic dispatch where possible.

**Rule:** "Engineer's promise" < "Compiler guarantee." If an invariant can be encoded in
the type system, it is encoded.

---

## 5. Reliability and Correctness over "Magic"

- Data corruption is not acceptable.
- Every optimization must have a safe rollback path (fallback).
- Optimizations must not compromise verifiability, observability, or recovery.
- **Fail-closed by default:** unsafe combinations or inputs → error/rejection, not
  "best-effort continuation."
- **No-panic policy:** the server does not crash due to user input. `unwrap()`/`expect()`
  in production code are forbidden.

---

## 6. Strategic Architecture Directions

These directions define where AngaraBase architecture is heading. All decisions must be
compatible with them or explicitly justify deviation via RFC.

### 6a. UNDO-log MVCC (Oracle/InnoDB style)

AngaraBase uses the **UNDO-log model** for MVCC: heap pages contain only current row
versions; history is moved to an append-only UNDO log. This is a strategic choice that
eliminates the need for VACUUM and ensures heap compactness.

- No VACUUM, no heap bloat, bounded UNDO chain (max 15 deltas per row).
- Recovery: ARIES-style (Analysis → Redo → Undo + CLR).

### 6b. HTAP-ready Architecture (Row + Column)

AngaraBase is built as an **HTAP-ready** system: row storage for OLTP, columnar storage
for OLAP, with transparent routing by query type.

- Row engine: `HeapStore` + `BufferPool`.
- Column engine: `AngaraColumn` segments + Column Cache.
- HTAP bridge: `StorageType::HtapRowColumn` — optimizer routes queries to the optimal engine.
- Pluggable engines: `TableEngine` trait allows new engines (LSM, external) without changing
  the core.

### 6c. Async/Sync Hybrid

New network, replication, and CDC components are async (tokio). The existing
storage/WAL/transaction layer is sync with a `spawn_blocking` bridge.

**Rule:** `std::thread::spawn` in new components is forbidden.

### 6d. Scale-Out Vision (v0.8+)

AngaraBase is designed with **transparent sharding** in view: coordinator cluster
(AngaraCoord) + data nodes (AngaraShard). Single-node path does not change;
sharding is opt-in.

**Rule:** decisions in v0.5–v0.7 must not block the distributed layer. Critical path:
async foundation → replication → auto-failover → sharding.

### 6e. Security as First-Class Concern

Security is a repeatable process, not an afterthought. Every security-impacting change
goes through planning, has fail-closed rules, pinned tests, and does not create doc drift.

- TDE: encryption at rest (heap pages + UNDO segments) via unified DEK scope.
- A security checklist (S1–S9) is required in every RFC touching data, I/O, network,
  or authentication.

---

## 7. Linux-only as a Strategic Choice

AngaraBase server is **Linux-only** (in the MVP / early-release phase).

**Why:**
- Maximum performance and predictability (tail latency).
- Faster product development by supporting a single platform.
- Linux-native mechanisms: io_uring, eBPF, advanced FS APIs.

**Principle:** when choosing between "cross-platform" and "maximum performance / control,"
we choose **performance**.

---

## 8. Business Logic in the Application

- AngaraBase is a "black box" for storing/retrieving data and executing SQL.
- Business logic must live in the application (client/service layer), not in triggers
  or stored procedures.
- Built-in SQL functions and views are permitted as part of SQL compatibility and
  read convenience.
- User-defined functions (UDF) are planned for future phases, but only with clear
  constraints (e.g., no side effects at launch).

---

## 9. DBA and Developer Experience

We build a system that is:
- Easy to operate (observability, backup/restore, clear error messages).
- Easy to develop against (modularity, well-defined APIs, testability).
- Easy to integrate (pgwire compatibility).

---

## 10. PostgreSQL Compatibility as a Bridge

- pgwire is the priority interface.
- SQL compatibility is achieved incrementally.
- We are not required to copy PostgreSQL 1-for-1: if architecturally safer or faster,
  we do it differently, but we document the differences.
- **Key distinction:** AngaraBase uses UNDO-log MVCC (§6a), unlike PostgreSQL multi-version
  heap. This means: no VACUUM, no bloat, a different GC model.

---

## 11. Performance-first

- Performance matters, but **not at the cost of correctness**.
- When choosing between "configuration flexibility" and "core speed/simplicity," we choose
  **speed/simplicity**.
- Every performance claim is backed by a pinned benchmark shipped with the release.

---

## 12. Modularity and Evolution

- Contracts and layers matter more than specific implementations.
- Pluggable storage engines (`TableEngine` trait) — the mechanism for evolution without
  rewriting the core.
- Kernel integration / SPDK / XDP — optional layers only, with user-space fallback.
- Each layer (Core → Adapters → Tooling) has a clear dependency direction: Core does not
  depend on Adapters or Tooling.

---

## 13. Documentation and Process Discipline

- Documentation-as-code.
- Repository discipline: commit conventions, coding standards, layering rules.
- Significant decisions go through RFC.
- Anti-drift: documentation is updated in the same commit that changes behavior.

---

## 14. Frequent Small Changes (Controlled Risk)

- We prefer **frequent small changes** over rare "big rewrites."
- This increases security (faster fixes and dependency updates), enables use of modern
  Linux capabilities, and makes risks **controlled** (easier review, easier rollback).
- This directly affects support policy: in early releases we target **modern stable Linux
  kernel versions**.

---

## 15. Minimal Dependencies

- **Principle:** Minimize the number of external dependencies (crates).
- **Selection:** Use only stable, proven, and maintained libraries. Avoid "magic" frameworks
  that hide complexity.
- **Discipline:** Adding any new dependency requires a separate analysis (trade-off:
  maintainability vs. utility) and justification in the PR or RFC.
- **Goal:** Reduce supply chain risk, simplify audit, reduce compile time and binary size.

---

## 16. What We Are Not Doing (yet)

- Server for Windows / macOS.
- Full PostgreSQL extension ecosystem.
- Complicating the core "for a feature" without clear user value.
- Globally distributed (multi-region active-active) — separate product class, v0.8+ horizon.

---

Changes to these principles require an RFC.
