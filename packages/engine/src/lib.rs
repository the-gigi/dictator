//! Dictator's decision-making: where a phrase ends, and what happens to it.
//!
//! Depends only on `object_model`, so its tests run without a microphone, a GPU,
//! or a whisper.cpp build. Sibling to `platform` — neither knows about the
//! other; the binary wires them together.

mod segmenter;

use object_model::event::DictatorEvents;
use object_model::{Injector, Recorder, Transcriber};
use rtrb::Consumer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DRAIN_INTERVAL: Duration = Duration::from_millis(30);
const FRAME_DURATION: Duration = Duration::from_millis(30);
const PADDING_DURATION: Duration = Duration::from_millis(150);
const HANGOVER_DURATION: Duration = Duration::from_millis(700);
const THRESHOLD: f32 = 0.003; // update the audio threshold as needed

pub struct Engine {
    recorder: Arc<dyn Recorder>,
    transcriber: Arc<dyn Transcriber>,
    consumer: Mutex<Consumer<f32>>,
    injector: Arc<dyn Injector>,

    flush: AtomicBool,
}

impl Engine {
    pub fn new(
        recorder: Arc<dyn Recorder>,
        transcriber: Arc<dyn Transcriber>,
        consumer: Consumer<f32>,
        injector: Arc<dyn Injector>,
    ) -> Arc<Self> {
        let engine = Arc::new(Self {
            transcriber,
            recorder,
            consumer: Mutex::new(consumer),
            injector,
            flush: AtomicBool::new(false),
        });

        let worker = engine.clone();
        thread::spawn(move || worker.run());

        engine
    }

    fn run(&self) {
        let frame_len =
            (FRAME_DURATION.as_secs_f64() * self.recorder.sample_rate() as f64) as usize; // samples
        let padding = PADDING_DURATION
            .as_nanos()
            .div_ceil(FRAME_DURATION.as_nanos()) as usize; // frames
        let hangover = HANGOVER_DURATION
            .as_nanos()
            .div_ceil(FRAME_DURATION.as_nanos()) as usize; // frames
        let mut segmenter = segmenter::Segmenter::new(frame_len, padding, hangover, THRESHOLD);

        let mut audio: Vec<f32> = Vec::new();
        let mut consumer = self.consumer.lock().unwrap();
        loop {
            let chunk_size = consumer.slots();
            if let Ok(chunk) = consumer.read_chunk(chunk_size) {
                let (first, second) = chunk.as_slices();
                audio.extend_from_slice(first);
                audio.extend_from_slice(second);
                chunk.commit_all();

                // For reference: one-liner alternative - but goes item by item through iterator so slower
                //content.extend(chunk);
            }

            let audio_slice = audio.as_slice();
            let frames = audio_slice.len() / frame_len;

            for i in 0..frames {
                let frame = &audio_slice[i * frame_len..(i + 1) * frame_len];
                if let Some(segment) = segmenter.handle_frame(frame) {
                    self.handle_segment(segment.as_slice())
                }
            }

            audio.drain(..frames * frame_len);

            if self.flush.swap(false, Ordering::Acquire) {
                if let Some(segment) = segmenter.flush() {
                    self.handle_segment(segment.as_slice());
                }

                audio.clear();
                println!("engine: stop");
            }

            thread::sleep(DRAIN_INTERVAL)
        }
    }

    pub fn flush(&self) {
        self.flush.store(true, Ordering::Release)
    }

    fn handle_segment(&self, segment: &[f32]) {
        println!(
            "segment: {:.2}s",
            segment.len() as u32 / self.recorder.sample_rate()
        );

        let resampled: Vec<f32> = segment
            .chunks_exact(3)
            .map(|c| (c[0] + c[1] + c[2]) / 3.0)
            .collect();

        match self.transcriber.transcribe(resampled.as_slice()) {
            Ok(text) => self
                .injector
                .inject(text.as_str())
                .expect("failed to inject text!"),
            Err(e) => {
                println!("failed to transcribe audio! {:?}", e)
            }
        }
    }
}

impl DictatorEvents for Engine {
    fn on_start(&self) {
        println!("engine: start");
        self.recorder.record().expect("failed to record recorder");
    }

    fn on_stop(&self) {
        self.recorder.stop();
        self.flush();
    }

    fn on_capture_error(&self, message: &str) {
        println!("on_capture_erro: {}", message);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_on_start() {}

    #[test]
    fn test_on_stop() {}

    #[test]
    fn test_on_capture_error() {}

    #[test]
    fn test_run() {}
}
