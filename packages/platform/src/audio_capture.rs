use std::sync::{Mutex, Arc};
use cpal::{Error, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use object_model::error::DictatorError;
use object_model::event::DictatorEvents;
use object_model::Recorder;

pub struct AudioCapture {
    sink: Mutex<Option<Arc<dyn DictatorEvents>>>,
    stream: Mutex<Option<cpal::Stream>>
}

pub fn new() -> AudioCapture {
    AudioCapture {
        sink: Mutex::new(None),
        stream: Mutex::new(None)
    }
}


impl AudioCapture {
    pub fn attach_sink(&self, sink: Arc<dyn DictatorEvents>) {
        self.sink.lock().unwrap().replace(sink);
    }
}

impl Recorder for AudioCapture {
    fn record(&self) -> Option<DictatorError> {
        let host = cpal::default_host();
        let device_wrapper = host.default_input_device()
            .ok_or_else(|| DictatorError::AudioCapture("no input device".into()));
        let device = device_wrapper.unwrap();
        let supported = device.default_input_config()
            .map_err(|_| DictatorError::AudioCapture("no default input config available".into()));
        let config: StreamConfig = supported.unwrap().config();
        let sink1 = self.sink.lock().unwrap().clone().unwrap();
        let sink2 = self.sink.lock().unwrap().clone().unwrap();
        match device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| sink1.on_samples(data),
            move |err: Error| sink2.on_capture_error(err.message().unwrap()),
            None) {
                Ok(stream) => {
                    stream.play().unwrap();
                    self.stream.lock().unwrap().replace(stream);
                }
                Err(e) => {
                    DictatorError::AudioCapture(format!("{:?}", e));
                }
            }
        None
    }

    fn stop(&self) {
        drop(self.stream.lock().unwrap().take());
    }
}
