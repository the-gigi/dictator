//! Error type shared by every stage of the pipeline.
//!
//! Variants will carry `String` rather than the underlying `cpal` /
//! `whisper-rs` / `enigo` error types: that is what keeps this package free of
//! platform dependencies. The adapters in `platform` stringify on the way out.
//!
//! `thiserror` here rather than `anyhow` — libraries hand callers matchable
//! variants, applications flatten. `anyhow` belongs to `dictator` alone.
//!
//! Holds: `Error`, `Result<T>`.
