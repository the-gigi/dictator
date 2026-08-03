//! Concrete implementations of the `object_model` traits against the real
//! machine: microphone, GPU, and keyboard.
//!
//! Everything here can only be verified by actually talking at it, which is the
//! boundary this crate exists to draw. Logic worth unit-testing belongs in
//! `object_model` instead.

use std::sync::Arc;
use object_model::event::DictatorEvents;
use rdev::{listen, Event, EventType, Key};

pub struct DictationGate  {
    sink: Arc<dyn DictatorEvents>
}

impl DictationGate {
    pub fn new(sink: Arc<dyn DictatorEvents>) -> Self {
        Self { sink }
    }

    pub fn run(self) -> rdev::ListenError {
        match listen(move |event| self.handle(event)) {
            Err(e) => e,
            Ok(()) => unreachable!("rdev::listen only returns on failure"),
        }
    }

    fn handle(&self, event: Event) {
        match event.event_type {
            EventType::KeyPress(Key::AltGr) => self.sink.on_start(),
            EventType::KeyRelease(Key::AltGr) => self.sink.on_stop(),
            _ => (),
        }
    }
}