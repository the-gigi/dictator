//! One completed utterance, bounded by silence. 16 kHz mono PCM.
//!
//! The type only. The `Segmenter` that decides where utterances begin and end
//! is behaviour, and lives in `engine`.
//!
//! Holds: `Segment`.
