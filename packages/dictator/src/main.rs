use std::env::args;
use std::sync::Arc;
use engine::Engine;
use platform::audio_capture::AudioCapture;
use platform::dictator_gate::DictationGate;
use rtrb::RingBuffer;
use std::time::Duration;
use platform::audio_transcriber::WhisperTranscriber;
use platform::text_injector::TextInjector;

// Assumed to match the default mic (48 kHz mono) until we query the device.
const SAMPLE_RATE_HZ: u32 = 48_000;
const CHANNELS: u16 = 1;
const RING_SPAN: Duration = Duration::from_secs(30);
const DEFAULT_WHISPER_MODEL_PATH: &str = "models/ggml-large-v3-turbo.bin";

fn main() {
    let whisper_model_path = args().nth(1).unwrap_or_else(|| DEFAULT_WHISPER_MODEL_PATH.to_string());

    // create the platform's TriggerSource
    let ring_slots = RING_SPAN.as_secs() as usize * SAMPLE_RATE_HZ as usize * CHANNELS as usize;
    let (producer, consumer) = RingBuffer::new(ring_slots);
    let recorder = Arc::new(AudioCapture::new(producer));
    let transcriber = Arc::new(WhisperTranscriber::new(whisper_model_path));
    let injector = Arc::new(TextInjector::new());
    let engine = Engine::new(recorder.clone(), transcriber, consumer, injector);
    recorder.attach_sink(engine.clone());
    let dictation_gate = DictationGate::new(engine.clone());

    dictation_gate.run();
}
