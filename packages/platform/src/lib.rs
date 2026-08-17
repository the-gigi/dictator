//! Concrete implementations of the `object_model` traits against the real
//! machine: microphone, GPU, and keyboard.
//!
//! Everything here can only be verified by actually talking at it, which is the
//! boundary this crate exists to draw. Logic worth unit-testing belongs in
//! `object_model` instead.

pub mod dictator_gate;
pub mod audio_capture;