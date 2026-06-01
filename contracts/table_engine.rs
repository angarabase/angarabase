//! # TableEngine Contract — AngaraBase v0
//!
//! This file is the **public specification** of the `TableEngine` trait —
//! the interface every storage engine must satisfy to plug into the
//! AngaraBase storage manager.
//!
//! **This file is a specification artifact, not a compilation unit.**
//! The implementation ships in the installation package (see `LICENSE`).
//!
//! ## Pluggable engines in AngaraBase v0
//!
//! | Engine | Durability | MVCC | Use case |
//! |--------|-----------|------|---------|
//! | `HeapStore` (row store) | Full (WAL + fsync) | UNDO-log | General OLTP |
//! | `AngaraMemory::None` | No persistence | In-memory snapshot | Ephemeral caches, temp tables |
//! | `AngaraMemory::Logged` | WAL only (no fsync) | In-memory snapshot | Durable at crash, fast at runtime |
//! | `AngaraMemory::Snapshotted` | WAL + periodic snapshot | In-memory snapshot | Low-latency + crash recovery |
//! | `AngaraColumn` *(v0.7+)* | Full | Shared MVCC contour | HTAP analytics inside same instance |
//!
//! Reference: RFC-2026-073 (Production Storage Architecture)

use std::ops::RangeBounds;

// ---------------------------------------------------------------------------
// §1 — Durability tiers (AngaraMemory)
// ---------------------------------------------------------------------------

/// Durability tier for AngaraMemory (in-memory) tables.
///
/// The tier is declared at table creation time and cannot be changed without
/// recreating the table. It is part of the *schema contract* visible in
/// `INFORMATION_SCHEMA` and in `SHOW TABLE STATUS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryDurabilityTier {
    /// No persistence. Data is lost on process restart.
    /// Config gate: `memory_engine_none_enabled = true` (default: true)
    None,

    /// WAL-logged but no fsync. Survives normal shutdown; data may be lost
    /// after a crash if the WAL buffer was not flushed.
    /// Prometheus metric: `angarabase_memory_engine_wal_bytes_written_total`
    Logged,

    /// WAL + periodic snapshot. Full crash recovery with bounded replay window.
    /// Config gate: `memory_engine_snapshot_interval_s` (0 = disabled)
    /// Prometheus metric: `angarabase_memory_engine_snapshots_total`
    Snapshotted,
}

// ---------------------------------------------------------------------------
// §2 — Engine type (routing key for the storage manager)
// ---------------------------------------------------------------------------

/// The type tag used by the storage manager to route DML to the correct engine.
///
/// Each `CREATE TABLE` statement specifies an engine (default: `HeapStore`).
/// The engine type is stored in the system catalog and is immutable after creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineType {
    /// Row-oriented on-disk store with UNDO-log MVCC.
    /// Default engine for all OLTP tables.
    HeapStore,

    /// In-memory store with three durability tiers.
    AngaraMemory(MemoryDurabilityTier),

    /// Columnar engine for analytics (v0.7+, experimental).
    /// Shares the MVCC contour with HeapStore tables.
    AngaraColumn,
}

// ---------------------------------------------------------------------------
// §3 — Core TableEngine trait
// ---------------------------------------------------------------------------

/// The `TableEngine` trait is the **contract every storage engine must satisfy**.
///
/// The storage manager calls this trait through a vtable; it does not know
/// (or care) about engine internals. All engines must implement all methods.
///
/// ### Fail-closed requirements
///
/// Every method that can fail MUST return an `EngineError` rather than panic.
/// Panics inside an engine implementation are treated as contract violations
/// and will terminate the server in debug builds.
///
/// ### Observability requirements
///
/// Engines must emit the following Prometheus metrics:
/// - `angarabase_engine_reads_total{engine, table}` (counter)
/// - `angarabase_engine_writes_total{engine, table}` (counter)
/// - `angarabase_engine_errors_total{engine, table, sqlstate}` (counter)
///
/// Failure to emit these metrics is a contract violation.
pub trait TableEngine: Send + Sync {
    // -----------------------------------------------------------------------
    // §3.1 — Identity
    // -----------------------------------------------------------------------

    /// Returns the engine type tag for catalog storage and routing.
    fn engine_type(&self) -> EngineType;

    /// Returns the declared durability guarantee for this engine instance.
    /// For `HeapStore` this is always `DurabilityGuarantee::Full`.
    fn durability(&self) -> DurabilityGuarantee;

    // -----------------------------------------------------------------------
    // §3.2 — Read path
    // -----------------------------------------------------------------------

    /// Point lookup by primary key.
    ///
    /// Returns `Ok(None)` if the key does not exist (not an error).
    /// Returns `Err(EngineError::Sqlstate("40001"))` if the transaction's
    /// snapshot has been force-closed.
    fn get(
        &self,
        txn: &dyn TransactionContext,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, EngineError>;

    /// Range scan over `[range]`.
    ///
    /// Returns an iterator that yields rows in key order.
    /// The iterator must respect the transaction's MVCC snapshot: rows
    /// modified after the snapshot's start timestamp MUST NOT be visible.
    fn scan<'a>(
        &'a self,
        txn: &'a dyn TransactionContext,
        range: impl RangeBounds<Vec<u8>> + 'a,
    ) -> Result<Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), EngineError>> + 'a>, EngineError>;

    // -----------------------------------------------------------------------
    // §3.3 — Write path
    // -----------------------------------------------------------------------

    /// Insert or replace a row.
    ///
    /// Must update the UNDO log (for HeapStore) or the in-memory snapshot
    /// (for AngaraMemory) atomically with respect to the transaction.
    ///
    /// Returns `Err(EngineError::Sqlstate(SQLSTATE_WRITE_SET_EXCEEDED))` if
    /// the per-transaction write set budget is exhausted.
    fn insert(
        &self,
        txn: &dyn TransactionContext,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), EngineError>;

    /// Delete a row by primary key.
    ///
    /// Returns `Ok(false)` if the key does not exist (not an error).
    fn delete(
        &self,
        txn: &dyn TransactionContext,
        key: &[u8],
    ) -> Result<bool, EngineError>;

    // -----------------------------------------------------------------------
    // §3.4 — Transaction lifecycle
    // -----------------------------------------------------------------------

    /// Called when the transaction commits.
    ///
    /// The engine must make all writes in `txn` durable according to its
    /// declared `durability()` tier before returning `Ok(())`.
    fn on_commit(&self, txn: &dyn TransactionContext) -> Result<(), EngineError>;

    /// Called when the transaction rolls back (explicit or error-path).
    ///
    /// The engine must undo all writes in `txn`. For HeapStore this is done
    /// by replaying UNDO log entries. For AngaraMemory this drops the in-memory
    /// write buffer.
    fn on_rollback(&self, txn: &dyn TransactionContext) -> Result<(), EngineError>;

    // -----------------------------------------------------------------------
    // §3.5 — Observability
    // -----------------------------------------------------------------------

    /// Returns a snapshot of engine-level statistics.
    ///
    /// Called by the metrics subsystem every `metrics_scrape_interval_ms`.
    /// The implementation MUST be lock-free or use a dedicated stats path.
    fn stats(&self) -> EngineStats;
}

// ---------------------------------------------------------------------------
// §4 — Supporting types (abbreviated for specification purposes)
// ---------------------------------------------------------------------------

/// Durability guarantee declared by an engine instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityGuarantee {
    /// All committed writes are fsync'd to stable storage before `COMMIT` returns.
    Full,
    /// Writes survive a clean shutdown; crash may lose the last WAL buffer.
    WalOnly,
    /// Writes survive only within the current process lifetime.
    Memory,
}

/// Engine-level error. Always includes a SQLSTATE for client propagation.
#[derive(Debug)]
pub struct EngineError {
    /// PostgreSQL-compatible SQLSTATE code (5 characters).
    pub sqlstate: &'static str,
    /// Human-readable message. Must not contain internal file paths.
    pub message: String,
}

/// Opaque transaction context passed by the storage manager to engine calls.
///
/// Engines must not hold references to `TransactionContext` beyond the
/// lifetime of a single method call.
pub trait TransactionContext: Send {
    /// Returns the transaction ID (monotonically increasing, unique per server).
    fn txn_id(&self) -> u64;

    /// Returns the MVCC snapshot timestamp for read visibility checks.
    fn snapshot_ts(&self) -> u64;

    /// Returns `true` if the transaction has been marked for abort.
    fn is_aborted(&self) -> bool;
}

/// Engine statistics snapshot (emitted to Prometheus).
#[derive(Debug, Default, Clone)]
pub struct EngineStats {
    pub reads_total: u64,
    pub writes_total: u64,
    pub errors_total: u64,
    /// Bytes of live data managed by this engine instance.
    pub live_bytes: u64,
    /// Number of rows currently visible to new transactions.
    pub live_rows: u64,
}
