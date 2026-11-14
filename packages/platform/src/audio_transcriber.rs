use object_model::Transcriber;
use object_model::error::DictatorError;
use std::sync::Mutex;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperError,
    WhisperState, install_logging_hooks,
};

pub struct WhisperTranscriber {
    params: FullParams<'static, 'static>,
    state: Mutex<WhisperState>,
}

impl WhisperTranscriber {
    pub fn new(model_path: String) -> WhisperTranscriber {
        install_logging_hooks();
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .expect("failed to open whisper model");
        let state = ctx.create_state().expect("failed to create Whisper state");
        let state = Mutex::new(state);
        let params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });

        WhisperTranscriber { params, state }
    }
}

fn whisper_err(e: WhisperError) -> DictatorError {
    DictatorError::Transcription(e.to_string())
}

impl Transcriber for WhisperTranscriber {
    fn transcribe(&self, segment: &[f32]) -> Result<String, DictatorError> {
        let mut state = self.state.lock().unwrap();
        state
            .full(self.params.clone(), segment)
            .map_err(whisper_err)?;
        let mut result = String::new();
        for s in state.as_iter() {
            let start_timestamp = s.start_timestamp();
            let end_timestamp = s.end_timestamp();
            println!("[{} - {}]: {}", start_timestamp, end_timestamp, s);

            let text = s.to_str_lossy().map_err(whisper_err)?;
            result.push_str(&text);
        }
        Ok(result)
    }
}
