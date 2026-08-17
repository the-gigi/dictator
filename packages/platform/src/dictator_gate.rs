use std::sync::Arc;
use object_model::event::DictatorEvents;
use object_model::error::DictatorError;
use rdev::{listen, Event, EventType, Key};

pub struct DictationGate  {
    sink: Arc<dyn DictatorEvents>
}

impl DictationGate {
    pub fn new(sink: Arc<dyn DictatorEvents>) -> Self {
        Self { sink }
    }

    pub fn run(self) -> DictatorError {
        match listen(move |event| self.handle(event)) {
            Err(e) => DictatorError::KeyListener(format!("{:?}", e)),
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