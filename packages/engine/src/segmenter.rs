//! Split a stream of audio frames into segments of speech separated by long silence

use std::cmp::min;

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

        let quiet = self.is_quiet(samples);
        // If loud frame, reset the quiet suffix count and return `None`
        // because we never end a segment on a loud frame
        if !quiet {
            self.quiet_suffix_count = 0;
            return None
        }

        self.quiet_prefix_count += 1;
        if !self.has_speech() {
            if self.quiet_prefix_count > self.padding {
                // drop first frame
                self.speech.drain(..self.frame_len);
                self.quiet_prefix_count -= 1;
            }
            return None
        }

        // We have speech so the new Q frame is added to the suffix count and removed from the prefix
        self.quiet_prefix_count -= 1;
        self.quiet_suffix_count += 1;

        if self.quiet_suffix_count < self.hangover {
            return None
        }

        let tail = (self.hangover - self.padding) * self.frame_len;
        let mut next_speech = self.speech.split_off(self.speech.len() - tail);
        let result = Some(self.speech.to_vec());
        let keep = min(self.padding, self.hangover - self.padding) * self.frame_len;
        next_speech.drain(..next_speech.len() - keep);
        self.speech = next_speech;
        self.quiet_prefix_count = self.speech.len() / self.frame_len;
        self.quiet_suffix_count = 0;

        result
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::min;
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
        let mut s = Segmenter::new(FRAME_LEN, 2, 6, 0.5);

        let quiet_frame = make_frame(0.3);
        let loud_frame = make_frame(0.8);

        // baseline
        assert_eq!(s.quiet_prefix_count, 0);
        assert_eq!(s.quiet_suffix_count, 0);

        // add 3 quiet frames
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());

        // Verify only two frames in the pre-frames and quiet prefix is two as well
        assert_eq!(s.speech.len(), 2 * FRAME_LEN);
        assert_eq!(s.quiet_prefix_count, 2);
        assert_eq!(s.quiet_suffix_count, 0);

        // add two loud frames
        assert!(s.handle_frame(&loud_frame).is_none());
        assert!(s.handle_frame(&loud_frame).is_none());
        // add quiet frame
        assert!(s.handle_frame(&quiet_frame).is_none());

        // Verify quiet prefix is still 2
        assert_eq!(s.quiet_prefix_count, 2);

        // add another loud frame
        assert!(s.handle_frame(&loud_frame).is_none());
        // add 3 more quiet frames
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());

        // Verify quiet prefix is still 2 and quiet suffix is 3 now
        assert_eq!(s.quiet_prefix_count, 2);
        assert_eq!(s.quiet_suffix_count, 3);

        // add 2 more quiet frames
        assert!(s.handle_frame(&quiet_frame).is_none());
        assert!(s.handle_frame(&quiet_frame).is_none());

        // add the final quiet frame that will signal the end of the segment
        // Expect 2 quiet frames + 2 loud frames + quiet frame + loud frame + 2 quiet frames
        let expected = [
            quiet_frame.repeat(2),
            loud_frame.repeat(2),
            quiet_frame.clone(),
            loud_frame.clone(),
            quiet_frame.repeat(2),
        ].concat();

        let expected_len = expected.len();
        let actual = s.handle_frame(&quiet_frame).unwrap();
        let actual_len = actual.len();
        assert_eq!(expected_len, actual_len);
        assert_eq!(expected, actual);
        assert_eq!(s.quiet_prefix_count, min(s.padding, s.hangover - s.padding));
        assert_eq!(s.quiet_suffix_count, 0);
    }
}