use std::sync::{Mutex, Arc};
use cpal::{Error, StreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use object_model::error::DictatorError;
use object_model::event::DictatorEvents;

pub struct AudioCapture {
    sink: Arc<dyn DictatorEvents>,
    stream: Mutex<Option<cpal::Stream>>
}

impl AudioCapture {
    pub fn new(sink: Arc<dyn DictatorEvents>) -> Self {
        Self {
            sink,
            stream: Mutex::new(None)
        }
    }

    pub fn capture(self) -> Option<DictatorError> {
        let host = cpal::default_host();
        let device_wrapper = host.default_input_device()
            .ok_or_else(|| DictatorError::AudioCapture("no input device".into()));
        let device = device_wrapper.unwrap();
        let supported = device.default_input_config()
            .map_err(|_| DictatorError::AudioCapture("no default input config available".into()));
        let config: StreamConfig = supported.unwrap().config();
        let sink1 = self.sink.clone();
        let sink2 = self.sink.clone();
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
}
