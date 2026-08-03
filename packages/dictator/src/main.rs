use std::sync::Arc;
use engine::Engine;
use platform::DictationGate;

fn main() {
    // create the platform's TriggerSource
    let engine = Engine::new();
    let dictation_gate = DictationGate::new(Arc::new(engine));

    dictation_gate.run();
}
