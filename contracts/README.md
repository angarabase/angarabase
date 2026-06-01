---
title: "AngaraBase — Public Contracts"
language: en
doc_type: public_contract
status: active
last_updated: 2026-06-01
---

# AngaraBase — Public Contracts

This directory contains the **public-facing specification contracts** for AngaraBase.
They are the authoritative source for what the database *guarantees* to callers —
independent of how those guarantees are implemented.

## What "contract" means here

A contract in AngaraBase has five mandatory components:

1. **Boundary** — the named resource limit (config knob, e.g. `undo_max_size_mb`).
2. **Metric** — the Prometheus counter / gauge that surfaces utilisation in real time.
3. **Wait event** — the named wait-event emitted when a thread blocks on this boundary.
4. **SQLSTATE** — the PostgreSQL-compatible error code returned when the limit is breached.
5. **Reaction contract** — what the *caller* (query pipeline, client driver) must do next.

If any of these five components is missing, the contract is considered incomplete
and the related code cannot merge. This is enforced by CI.

## Files in this directory

| File | What it specifies |
|------|-------------------|
| `admission_control.rs` | The eight named resource boundaries with their SQLSTATE codes, Prometheus metric names, and fail-closed behavior guarantees. |
| `table_engine.rs` | The `TableEngine` trait — the interface every storage engine must satisfy to plug into the AngaraBase storage manager. |

## Canonical documentation

- Architecture overview: [`docs/01_ARCHITECTURE.md`](docs/01_ARCHITECTURE.md)
- Project principles (fail-closed philosophy): [`docs/00_PROJECT_PRINCIPLES.md`](docs/00_PROJECT_PRINCIPLES.md)
- Supported SQL subset: [`docs/technical_specs/v1_supported_sql_subset.md`](docs/technical_specs/v1_supported_sql_subset.md)
- Project website: [angarabase.com](https://angarabase.com)
- Full documentation: [angarabase.dev](https://angarabase.dev)

## Source code

Source code is not in this repository by design — it ships inside installation packages
from [GitHub Releases](https://github.com/angarabase/angarabase/releases) under the
terms of `LICENSE`. The files here are the *specification layer*: what every caller,
operator, and monitoring system can rely on regardless of internal implementation details.
