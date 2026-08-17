//! Dictator's decision-making: where a phrase ends, and what happens to it.
//!
//! Depends only on `object_model`, so its tests run without a microphone, a GPU,
//! or a whisper.cpp build. Sibling to `platform` — neither knows about the
//! other; the binary wires them together.

use std::sync::Arc;
use object_model::event::DictatorEvents;
use object_model::Recorder;

pub struct Engine {
    recorder: Arc<dyn Recorder>
}

impl Engine {
    pub fn new(recorder: Arc<dyn Recorder>) -> Engine {
        let engine = Engine {
            recorder,
        };
        engine
    }
}

impl DictatorEvents for Engine {
    fn on_start(&self) {
        println!("engine: start");
        self.recorder.record();
    }

    fn on_stop(&self) {
        println!("engine: stop");
        self.recorder.stop();
    }

    fn on_samples(&self, samples: &[f32]) {
        println!("on_samples: {}", samples.len());
    }

    fn on_capture_error(&self, message: &str) {
        println!("on_capture_erro: {}", message);
    }
}
