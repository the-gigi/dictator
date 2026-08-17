use std::sync::Arc;
use engine::Engine;
use platform::audio_capture;
use platform::dictator_gate::DictationGate;

fn main() {
    // create the platform's TriggerSource
    let recorder = Arc::new(audio_capture::new());
    let engine = Arc::new(Engine::new(recorder.clone()));
    recorder.attach_sink(engine.clone());
    let dictation_gate = DictationGate::new(engine.clone());

    dictation_gate.run();
}
