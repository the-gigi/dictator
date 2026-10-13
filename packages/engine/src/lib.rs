//! Dictator's decision-making: where a phrase ends, and what happens to it.
//!
//! Depends only on `object_model`, so its tests run without a microphone, a GPU,
//! or a whisper.cpp build. Sibling to `platform` — neither knows about the
//! other; the binary wires them together.

use std::sync::{Arc, Mutex};
use std::thread;
use object_model::event::DictatorEvents;
use object_model::Recorder;
use rtrb::{Consumer};
use std::time::Duration;

const DRAIN_INTERVAL: Duration = Duration::from_millis(30);


pub struct Engine {
    recorder: Arc<dyn Recorder>,
    consumer: Mutex<Consumer<f32>>
}

impl Engine {
    pub fn new(recorder: Arc<dyn Recorder>, consumer: Consumer<f32>) -> Arc<Self> {
        let engine = Arc::new(Self {
            recorder,
            consumer: Mutex::new(consumer)
        });

        let worker = engine.clone();
        thread::spawn(move || worker.run());

        engine
    }

    fn run(&self) {
        let mut audio: Vec<f32> = Vec::new();
        let mut consumer = self.consumer.lock().unwrap();
        loop {
            let chunk_size= consumer.slots();
            if let Ok(chunk) = consumer.read_chunk(chunk_size) {
                let (first, second) = chunk.as_slices();
                audio.extend_from_slice(first);
                audio.extend_from_slice(second);
                chunk.commit_all();

                // one-liner alternative - but goes item by item through iterator so slower
                //content.extend(chunk);
            }

            if content.len() > 0 {
                println!("engine::run(), content size: {} ", content.len());
            }

            thread::sleep(DRAIN_INTERVAL)
        }
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

    fn on_capture_error(&self, message: &str) {
        println!("on_capture_erro: {}", message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_start() {}

    #[test]
    fn test_on_stop() {}

    #[test]
    fn test_on_capture_error() {}


}