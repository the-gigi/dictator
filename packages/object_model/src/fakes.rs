//! Test doubles for the four traits.
//!
//! Gated behind the `fakes` feature rather than `#[cfg(test)]` so that other
//! packages can use them. Integration tests compile as separate crates, which
//! never see `#[cfg(test)]` items — `engine`'s `tests/` directory reaches these
//! by enabling the feature in its dev-dependencies.
//!
//! Together these let the whole pipeline run with no microphone, no GPU, no
//! whisper.cpp build, and nothing typing into real windows.
//!
//! Holds: `FakeRecorder`, `FakeTranscriber`, `FakeInjector`, `FakeTrigger`.
