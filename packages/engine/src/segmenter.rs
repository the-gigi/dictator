//! Split a stream of audio frames into segments of speech separated by long silence

use std::collections::VecDeque;

/// The Segmenter is responsible to break a stream of frames to segments of speech separated by silence
///
/// This is not trivial because within segments there are
/// also short periods of silence and at the beginning and end
/// of segments there may be some quieter frames that still contain
/// audio that should be part of the segment.
///
/// The way it works is that the Segmenter gets the `threshold` that determines if a frame
/// is considered quiet or not and also the `hangover` that decides how many quiet frames
/// constitute a segment ending silence (as opposed to in-segment silence).
/// The `padding` is the number of quiet periods before and after the non-quiet sequence
/// of frames and include in the Segment (can be less than the hangover).
///
///  With padding = 2 and hangover = 4 (Q = quiet, L = loud):
///
///  input:   Q Q Q Q Q L L Q L L L Q Q Q Q
///
///  segment:       Q Q L L Q L L L Q Q
///                 ^^^             ^^^
///              pre-padding       post-padding
pub struct Segmenter {
    frame_len: usize, // number of samples in each frame
    quiet_prefix_count: usize, // how many quiet frames before first loud
    speech: Vec<f32>,
    quiet_suffix_count: usize , // how many contiguous quiet frames are currently at the end
    padding: usize, // number of padding quiet frames (same for pre and post)
    hangover: usize, // number of quiet frames after a phrase that determine complete segment
    threshold: f32, // the cutoff between quiet and speech where
                    // the noise level of a frame is the RMS - sqrt(mean(x²)) of its samples
}

impl Segmenter {
    pub fn new(frame_len: usize, padding: usize, hangover: usize, threshold: f32) -> Self {
        Self {
            quiet_prefix_count: 0,
            speech: Vec::<f32>::new(),
            quiet_suffix_count: 0,
            frame_len,
            padding,
            hangover,
            threshold,
        }
    }

    fn has_speech(&self) -> bool {
        self.speech.len() > self.quiet_prefix_count * self.frame_len
    }

    fn is_quiet(&self, samples: &[f32]) -> bool {
        // calculate RMS
        let sum_of_squares = samples.iter().map(|s| s * s).sum::<f32>();
        (sum_of_squares / samples.len() as f32).sqrt() < self.threshold
    }

    fn handle_frame(&mut self, samples: &[f32]) -> Option<Vec<f32>> {
        assert_eq!(samples.len(), self.frame_len);
        self.speech.extend_from_slice(samples);

        // If loud frame, reset the quiet suffix count and return `None`
        // because we never end a segment on a loud frame
        if !self.is_quiet(samples) {
            self.quiet_suffix_count = 0;
            return None
        }

        self.quiet_prefix_count += 1;
        if !self.has_speech() {
            if self.quiet_prefix_count <= self.padding {
                return None;
            }
            // drop first frame
            self.speech.drain(..self.frame_len);
            self.quiet_prefix_count -= 1;
        }

        // We have speech so the new Q frame is added to the suffic count
        self.quiet_suffix_count += 1;

        if self.quiet_suffix_count < self.hangover {
            return None
        }

        let cut = (self.hangover - self.padding) * self.frame_len;
        let mut next_speech = self.speech.split_off(self.speech.len() - cut);
        let result = Some(self.speech.to_vec());
        if next_speech.len() > self.frame_len * self.padding {
            next_speech.drain(..next_speech.len() - self.padding);
        }
        self.speech = next_speech;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_LEN: usize = 10;
    fn make_frame(amplitude: f32) -> Vec<f32> {
        return vec![amplitude; FRAME_LEN];
    }

    #[test]
    fn test_is_quiet() {
        let s = Segmenter::new(FRAME_LEN, 2, 4, 0.5);
        assert_eq!(s.is_quiet(&make_frame(0.3)), true);
        assert_eq!(s.is_quiet(&make_frame(0.8)), false);
    }

    #[test]
    fn test_has_speech() {
        let mut s = Segmenter::new(FRAME_LEN, 2, 4, 0.5);

        let quiet_frame = make_frame(0.3);
        let loud_frame = make_frame(0.8);

        assert!(s.handle_frame(quiet_frame.as_slice()).is_none());
        assert!(!s.has_speech());

        assert!(s.handle_frame(loud_frame.as_slice()).is_none());
        assert!(s.has_speech());
    }



    #[test]
    fn test_handle_frame() {
        // create Segmenter with padding of 2 and threshold of 0.5
        let mut s = Segmenter::new(FRAME_LEN, 2, 4, 0.5);

        let quiet_frame = make_frame(0.3);
        let loud_frame = make_frame(0.8);

        // add 3 quiet frames
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());

        // Verify only two frames in the pre-frames
        assert_eq!(s.speech.len(), 2 * FRAME_LEN);

        // add two loud frames
        assert!(s.handle_frame(&loud_frame).is_none());
        assert!(s.handle_frame(&loud_frame).is_none());
        // add quiet frame
        assert!(s.handle_frame(&quiet_frame).is_none());
        // add another loud frame
        assert!(s.handle_frame(&loud_frame).is_none());
        // add 3 more quiet frames
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());

        // add the 4th quiet frame that will signal the end of the segment
        // Expect 2 quiet frames + 2 loud frames + quiet frame + loud frame + 2 quiet frames
        let expected = [
            quiet_frame.repeat(2),
            loud_frame.repeat(2),
            quiet_frame.clone(),
            loud_frame.clone(),
            quiet_frame.repeat(2),
        ].concat();

        assert_eq!(expected, s.handle_frame(&quiet_frame).unwrap());
    }
}