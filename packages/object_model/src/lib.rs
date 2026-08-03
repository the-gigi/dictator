//! Shared vocabulary for dictator: the types and traits that more than one
//! package needs.
//!
//! Deliberately free of platform and I/O dependencies, and free of behaviour —
//! no segmentation, no orchestration, no device access. Logic lives in `engine`,
//! adapters live in `platform`. Anything used by only one package belongs there
//! instead of here.
//!
//! The four traits below are the ports: `platform` implements them, `engine`
//! consumes them, and `dictator` wires the two together.

pub mod config;
pub mod error;
pub mod event;
pub mod segment;

#[cfg(feature = "fakes")]
pub mod fakes;

// Traits go here rather than in a `traits.rs`: they are this crate's defining
// API, and naming a module after a kind of item rather than a domain concept
// tends to become a junk drawer.
//
// Recorder      - captures microphone audio
// Transcriber   - turns one completed utterance into text
// Injector      - delivers text to the focused window


