use std::sync::{Mutex, Arc};
use cpal::{Device, Error, StreamConfig, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use object_model::error::DictatorError;
use object_model::event::DictatorEvents;
use object_model::Recorder;
use rtrb::{Producer};

pub struct AudioCapture {
    sink: Mutex<Option<Arc<dyn DictatorEvents>>>,
    stream: Mutex<Option<cpal::Stream>>,
    producer: Arc<Mutex<Producer<f32>>>,
    device: Device,
    config: SupportedStreamConfig
}

pub fn new(producer: Producer<f32>) -> AudioCapture {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device.default_input_config().unwrap();

    AudioCapture {
        sink: Mutex::new(None),
        stream: Mutex::new(None),
        producer: Arc::from(Mutex::new(producer)),
        device,
        config
    }
}


impl AudioCapture {
    pub fn attach_sink(&self, sink: Arc<dyn DictatorEvents>) {
        self.sink.lock().unwrap().replace(sink);
    }
}

impl Recorder for AudioCapture {
    fn record(&self) -> Option<DictatorError> {

        let config: StreamConfig = self.config.into();
        let producer = Arc::clone(&self.producer);
        let sink = self.sink.lock().unwrap().clone().unwrap();
        match self.device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut p) = producer.try_lock() {
                    let _ = p.push_entire_slice(data);
                }
            },
            move |err: Error| sink.on_capture_error(err.message().unwrap()),
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

    fn sample_rate(&self) -> u32 {
        self.config.sample_rate()
    }

    fn channel_count(&self) -> u16 {
        self.config.channels()
    }
}
