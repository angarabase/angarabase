//! # Admission Control Contract — AngaraBase v0
//!
//! This file is the **public specification** of AngaraBase's fail-closed resource
//! boundaries. It describes:
//!
//! - the eight named resource limits;
//! - the SQLSTATE code returned when each limit is breached;
//! - the Prometheus metric family that surfaces utilisation;
//! - the mandatory caller reaction (what client code must do on each error).
//!
//! **This file is a specification artifact, not a compilation unit.**
//! The implementation ships in the installation package (see `LICENSE`).
//!
//! Reference: `docs/00_PROJECT_PRINCIPLES.md §1 — Restrictive by Default`

// ---------------------------------------------------------------------------
// §1 — SQLSTATE codes (PostgreSQL-compatible class 53 / 54 / 57 / 40)
// ---------------------------------------------------------------------------

/// `53100` — `insufficient_resources`: UndoStore has reached its disk budget.
///
/// **Config knob:** `undo_max_size_mb` (default: 8 192 MiB)
/// **Metric:** `angarabase_undo_store_bytes_used` (gauge)
/// **Wait event:** `UndoGCWait`
/// **Caller reaction:** Retry with exponential back-off after GC cycle completes
///   (observe `angarabase_undo_gc_cycles_total`). Do NOT retry immediately.
pub const SQLSTATE_UNDO_OVERLOAD: &str = "53100";

/// `53200` — `insufficient_resources` (memory): buffer pool eviction backpressure.
///
/// **Config knob:** `buffer_pool_size_mb`
/// **Metric:** `angarabase_buffer_pool_evictions_total` (counter)
/// **Wait event:** `BufferPoolEviction`
/// **Caller reaction:** Reduce write batch size; surface to operator as capacity signal.
pub const SQLSTATE_BUFFER_OVERLOAD: &str = "53200";

/// `53300` — `too_many_connections` / `too_many_concurrent_queries`: admission gate.
///
/// **Config knob:** `max_concurrent_queries`
/// **Metric:** `angarabase_admission_rejected_total` (counter, label: reason)
/// **Wait event:** `AdmissionWait`
/// **Caller reaction:** Exponential back-off with jitter; surface to load-balancer.
pub const SQLSTATE_ADMISSION_OVERLOAD: &str = "53300";

/// `54023` — `too_many_arguments`: per-transaction write set exceeded.
///
/// **Config knob:** `txn_max_write_set_mb` (default: 512 MiB per transaction)
/// **Metric:** `angarabase_txn_write_set_bytes` (histogram, per-transaction)
/// **Wait event:** *(none — immediate reject)*
/// **Caller reaction:** Roll back and retry with a smaller write batch.
pub const SQLSTATE_WRITE_SET_EXCEEDED: &str = "54023";

/// `57014` — `query_canceled`: statement timeout fired.
///
/// **Config knob:** `statement_timeout_ms` (0 = disabled)
/// **Metric:** `angarabase_statement_timeout_total` (counter)
/// **Wait event:** *(none — active cancel)*
/// **Caller reaction:** Roll back the transaction; do not retry the same statement
///   without reviewing the query plan.
pub const SQLSTATE_STATEMENT_TIMEOUT: &str = "57014";

/// `40001` — `serialization_failure`: stale snapshot force-closed.
///
/// **Config knob:** `max_snapshot_age` (seconds)
/// **Metric:** `angarabase_snapshot_force_close_total` (counter)
/// **Wait event:** *(none — active cancel)*
/// **Caller reaction:** Retry the entire transaction from the beginning.
pub const SQLSTATE_SNAPSHOT_AGE: &str = "40001";

/// `0A000` — `feature_not_supported`: SQL construct outside the supported subset.
///
/// Not a resource limit — a capability boundary. Returned for any SQL syntax or
/// feature that AngaraBase does not yet implement. The error message includes a
/// reference to the supported-SQL contract document.
///
/// Reference: `docs/technical_specs/v1_supported_sql_subset.md`
pub const SQLSTATE_NOT_SUPPORTED: &str = "0A000";

// ---------------------------------------------------------------------------
// §2 — Resource boundary descriptor (informational)
// ---------------------------------------------------------------------------

/// A descriptor that each resource boundary must publish.
///
/// Every component in AngaraBase that has a resource limit exposes one of these
/// so that operators and monitoring systems can discover the contract without
/// reading source code.
pub struct ResourceBoundaryDescriptor {
    /// Human-readable name of the boundary, e.g. `"UndoStore disk budget"`.
    pub name: &'static str,

    /// Config knob that controls the limit, e.g. `"undo_max_size_mb"`.
    pub config_key: &'static str,

    /// Prometheus metric name that tracks utilisation (gauge or counter).
    pub prometheus_metric: &'static str,

    /// Named wait event emitted when a thread blocks on this boundary.
    /// `None` if the component rejects immediately without blocking.
    pub wait_event: Option<&'static str>,

    /// SQLSTATE code returned when the limit is breached.
    pub sqlstate: &'static str,

    /// Short description of the fail-closed behavior.
    pub fail_closed_behavior: &'static str,
}

/// The eight named resource boundaries in AngaraBase v0.
///
/// This table is the authoritative operator reference. Monitoring runbooks,
/// alerting rules, and client retry logic MUST be derived from this table,
/// not from empirical observation.
pub const RESOURCE_BOUNDARIES: &[ResourceBoundaryDescriptor] = &[
    ResourceBoundaryDescriptor {
        name: "UndoStore disk budget",
        config_key: "undo_max_size_mb",
        prometheus_metric: "angarabase_undo_store_bytes_used",
        wait_event: Some("UndoGCWait"),
        sqlstate: SQLSTATE_UNDO_OVERLOAD,
        fail_closed_behavior: "Reject DML; force GC cycle; writer waits or returns 53100.",
    },
    ResourceBoundaryDescriptor {
        name: "Buffer pool memory",
        config_key: "buffer_pool_size_mb",
        prometheus_metric: "angarabase_buffer_pool_evictions_total",
        wait_event: Some("BufferPoolEviction"),
        sqlstate: SQLSTATE_BUFFER_OVERLOAD,
        fail_closed_behavior: "Evict pages (CLOCK); WAL-first flush; OOM guard rejects.",
    },
    ResourceBoundaryDescriptor {
        name: "Admission gate (concurrent queries)",
        config_key: "max_concurrent_queries",
        prometheus_metric: "angarabase_admission_rejected_total",
        wait_event: Some("AdmissionWait"),
        sqlstate: SQLSTATE_ADMISSION_OVERLOAD,
        fail_closed_behavior: "Reject immediately with 53300; no queuing by default.",
    },
    ResourceBoundaryDescriptor {
        name: "Per-transaction write set",
        config_key: "txn_max_write_set_mb",
        prometheus_metric: "angarabase_txn_write_set_bytes",
        wait_event: None,
        sqlstate: SQLSTATE_WRITE_SET_EXCEEDED,
        fail_closed_behavior: "Reject the DML statement; transaction must roll back.",
    },
    ResourceBoundaryDescriptor {
        name: "AngaraMemory max rows",
        config_key: "memory_engine_max_rows",
        prometheus_metric: "angarabase_memory_engine_rows_total",
        wait_event: None,
        sqlstate: "53000",
        fail_closed_behavior: "Reject INSERT when max_rows reached; no silent drop.",
    },
    ResourceBoundaryDescriptor {
        name: "Connection limit",
        config_key: "max_connections",
        prometheus_metric: "angarabase_connections_total",
        wait_event: None,
        sqlstate: "53300",
        fail_closed_behavior: "Reject new TCP connection; existing connections unaffected.",
    },
    ResourceBoundaryDescriptor {
        name: "Statement timeout",
        config_key: "statement_timeout_ms",
        prometheus_metric: "angarabase_statement_timeout_total",
        wait_event: None,
        sqlstate: SQLSTATE_STATEMENT_TIMEOUT,
        fail_closed_behavior: "Cancel the running statement; transaction must roll back.",
    },
    ResourceBoundaryDescriptor {
        name: "Snapshot age",
        config_key: "max_snapshot_age",
        prometheus_metric: "angarabase_snapshot_force_close_total",
        wait_event: None,
        sqlstate: SQLSTATE_SNAPSHOT_AGE,
        fail_closed_behavior: "Force-close stale snapshot; transaction receives 40001.",
    },
];
