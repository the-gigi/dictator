//! Dictator's decision-making: where a phrase ends, and what happens to it.
//!
//! Depends only on `object_model`, so its tests run without a microphone, a GPU,
//! or a whisper.cpp build. Sibling to `platform` — neither knows about the
//! other; the binary wires them together.

use object_model::event::DictatorEvents;

pub struct Engine {
        
}

impl Engine {
    pub fn new() -> Engine {
        
        Engine {
            
        }
    }
}

impl DictatorEvents for Engine {
    fn on_start(&self) {
        println!("engine_start");
    }

    fn on_stop(&self) {
        println!("engine_stop");
    }

    fn on_samples(&self, samples: &[f32]) {
        println!("on_samples: {}", samples.len());
    }

    fn on_capture_error(&self, message: &str) {
        println!("on_capture_erro: {}", message);
    }
}
