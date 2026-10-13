use std::collections::VecDeque;

struct Frame {
    audio: Vec<f32>,
    quiet: bool,
}

pub struct Segmenter {
    pre_frames: VecDeque<Vec<f32>>, // keep up to `padding` quiet frames. Drop early frames
    speech_frames: Vec<Vec<f32>>,
    post_padding_count: usize, // how many contiguous quiet frames are at the end of speech_frames
    padding: usize, // number of padding frames (same for pre and post)
    hangover: usize, // number of quiet frames after a phrase that determine complete segment
    threshold: f32, //
}

impl Segmenter {
    pub fn new(padding: usize, hangover: hangover) -> Self {
        pre_frames: VecDeque::<Vec<f32>>::new(),
        speech_frames:
        post_padding_count: 0,
        padding
        hangover
    }

    pub fn handle_frame(&mut self, samples: &[f32]) -> Option<Segment>{
        let frame = self.samples2frame(samples);
        if frame.quiet {
            if self.speech_frames.is_empty() {

            }

        }


        self.frames.push_back(frame);
    }

    fn samples2frame(&self, samples: &[f32]) -> Frame {

    }
}