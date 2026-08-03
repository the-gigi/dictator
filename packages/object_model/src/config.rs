//! Tunable knobs.
//!
//! Spans of time are `Duration`, never integer milliseconds, so units cannot be
//! misread at a call site.
//!
//! Holds: `SAMPLE_RATE`, `Config`, `VadConfig`, `InjectionMode`.
