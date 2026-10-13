use std::sync::Arc;
use engine::Engine;
use platform::audio_capture;
use platform::dictator_gate::DictationGate;
use rtrb::RingBuffer;
use std::time::Duration;

// Assumed to match the default mic (48 kHz mono) until we query the device.
const SAMPLE_RATE_HZ: u32 = 48_000;
const CHANNELS: u16 = 1;
const RING_SPAN: Duration = Duration::from_secs(30);

fn main() {
    // create the platform's TriggerSource
    let ring_slots = RING_SPAN.as_secs() as usize * SAMPLE_RATE_HZ as usize * CHANNELS as usize;
    let (producer, consumer) = RingBuffer::new(ring_slots);
    let recorder = Arc::new(audio_capture::new(producer));
    let engine = Engine::new(recorder.clone(), consumer);
    recorder.attach_sink(engine.clone());
    let dictation_gate = DictationGate::new(engine.clone());

    dictation_gate.run();
}
