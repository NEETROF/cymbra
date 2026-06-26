// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pure, host-testable audio logic — no device, threads, or FFI.
//!
//! Split out of [`super::audio`] so it can be unit-tested (and counted by
//! `cargo llvm-cov`) on CI hosts that have no audio device. The real-time
//! `cpal`/`rustysynth` glue in `audio.rs` stays out of the coverage gate; the
//! event model, the MIDI pitch/velocity mapping, and the voice bookkeeping that
//! the audio thread relies on live here.
//!
//! These types are internal to the engine (the FFI surface in `audio.rs` only
//! exposes plain `u8`/`Vec<u8>`), so they are `#[frb(ignore)]`d to keep them out
//! of the generated bridge.

use flutter_rust_bridge::frb;

/// MIDI channel the piano plays on. A single-instrument synth only needs one.
pub(crate) const PIANO_CHANNEL: i32 = 0;

/// Velocity used when a source carries no pressure information (the on-screen
/// keyboard, the computer-keyboard fallback). A musical mezzo-forte.
pub(crate) const DEFAULT_VELOCITY: u8 = 100;

/// A control event handed from the UI/FFI thread to the audio thread.
///
/// The audio callback drains a queue of these each block and applies them to the
/// synthesizer. Keeping the variants tiny and `Copy` makes the hand-off
/// allocation-free.
#[frb(ignore)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AudioEvent {
    /// Begin sounding `pitch` at `velocity`.
    NoteOn { pitch: u8, velocity: u8 },
    /// Release `pitch` (enters the SoundFont's release stage).
    NoteOff { pitch: u8 },
    /// Release every sounding voice (stop/restart/seek/loop).
    AllOff,
}

impl AudioEvent {
    /// Builds a normalized note-on. MIDI pitch/velocity are 7-bit, so values are
    /// clamped to `0..=127`; a zero velocity is treated as the default rather
    /// than an inaudible note (some sources send 0 for "no pressure").
    pub(crate) fn note_on(pitch: u8, velocity: u8) -> AudioEvent {
        let velocity = if velocity == 0 {
            DEFAULT_VELOCITY
        } else {
            clamp7(velocity)
        };
        AudioEvent::NoteOn {
            pitch: clamp7(pitch),
            velocity,
        }
    }

    /// Builds a note-off for `pitch` (clamped to the 7-bit MIDI range).
    pub(crate) fn note_off(pitch: u8) -> AudioEvent {
        AudioEvent::NoteOff {
            pitch: clamp7(pitch),
        }
    }
}

/// Clamps a value to the 7-bit MIDI range (`0..=127`).
pub(crate) fn clamp7(v: u8) -> u8 {
    v.min(127)
}

/// Tracks which pitches are currently sounding so the audio thread can release
/// them precisely on an [`AudioEvent::AllOff`] and so the model is testable
/// without a synthesizer.
///
/// rustysynth manages its own voices internally; this mirror lets `audio.rs`
/// issue an exact note-off per held pitch (and lets tests assert the bookkeeping
/// without a device).
#[frb(ignore)]
#[derive(Debug, Default, Clone)]
pub(crate) struct VoiceTracker {
    active: Vec<u8>,
}

impl VoiceTracker {
    /// A tracker with no sounding voices.
    pub(crate) fn new() -> VoiceTracker {
        VoiceTracker { active: Vec::new() }
    }

    /// Applies an event to the bookkeeping. Returns the pitches that should be
    /// released as a result — one pitch for a note-off, every held pitch for an
    /// all-off, and none for a note-on (the caller starts that voice).
    pub(crate) fn apply(&mut self, event: AudioEvent) -> Vec<u8> {
        match event {
            AudioEvent::NoteOn { pitch, .. } => {
                if !self.active.contains(&pitch) {
                    self.active.push(pitch);
                }
                Vec::new()
            }
            AudioEvent::NoteOff { pitch } => {
                if let Some(i) = self.active.iter().position(|&p| p == pitch) {
                    self.active.remove(i);
                    vec![pitch]
                } else {
                    Vec::new()
                }
            }
            AudioEvent::AllOff => std::mem::take(&mut self.active),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_clamps_pitch_and_velocity() {
        assert_eq!(
            AudioEvent::note_on(200, 200),
            AudioEvent::NoteOn {
                pitch: 127,
                velocity: 127
            }
        );
    }

    #[test]
    fn zero_velocity_becomes_default_not_silent() {
        assert_eq!(
            AudioEvent::note_on(60, 0),
            AudioEvent::NoteOn {
                pitch: 60,
                velocity: DEFAULT_VELOCITY
            }
        );
    }

    #[test]
    fn note_off_clamps_pitch() {
        assert_eq!(
            AudioEvent::note_off(200),
            AudioEvent::NoteOff { pitch: 127 }
        );
    }

    /// Sorted snapshot of the voices a tracker still holds — taken by releasing
    /// them all. Lets tests assert the bookkeeping through the public `apply`.
    fn held(v: &mut VoiceTracker) -> Vec<u8> {
        let mut h = v.apply(AudioEvent::AllOff);
        h.sort_unstable();
        h
    }

    #[test]
    fn note_on_adds_a_voice_and_returns_nothing_to_release() {
        let mut v = VoiceTracker::new();
        let released = v.apply(AudioEvent::note_on(60, 100));
        assert!(released.is_empty());
        assert_eq!(held(&mut v), vec![60]);
    }

    #[test]
    fn duplicate_note_on_does_not_double_count() {
        let mut v = VoiceTracker::new();
        v.apply(AudioEvent::note_on(60, 100));
        v.apply(AudioEvent::note_on(60, 100));
        assert_eq!(held(&mut v), vec![60]);
    }

    #[test]
    fn note_off_releases_only_that_voice() {
        let mut v = VoiceTracker::new();
        v.apply(AudioEvent::note_on(60, 100));
        v.apply(AudioEvent::note_on(64, 100));
        let released = v.apply(AudioEvent::note_off(60));
        assert_eq!(released, vec![60]);
        // 64 is still sounding; 60 is gone.
        assert_eq!(held(&mut v), vec![64]);
    }

    #[test]
    fn note_off_for_idle_pitch_releases_nothing() {
        let mut v = VoiceTracker::new();
        assert!(v.apply(AudioEvent::note_off(60)).is_empty());
    }

    #[test]
    fn polyphony_holds_multiple_voices() {
        let mut v = VoiceTracker::new();
        for p in [60, 64, 67] {
            v.apply(AudioEvent::note_on(p, 100));
        }
        assert_eq!(held(&mut v), vec![60, 64, 67]);
    }

    #[test]
    fn all_off_releases_every_held_voice_and_clears() {
        let mut v = VoiceTracker::new();
        for p in [60, 64, 67] {
            v.apply(AudioEvent::note_on(p, 100));
        }
        let mut released = v.apply(AudioEvent::AllOff);
        released.sort_unstable();
        assert_eq!(released, vec![60, 64, 67]);
        // The tracker is empty afterwards — a second all-off releases nothing.
        assert!(v.apply(AudioEvent::AllOff).is_empty());
    }

    #[test]
    fn events_apply_in_fifo_order() {
        // The audio thread drains the queue in order; applying a recorded queue
        // FIFO leaves exactly the voices the sequence implies.
        let queue = [
            AudioEvent::note_on(60, 100),
            AudioEvent::note_on(64, 100),
            AudioEvent::note_off(60),
            AudioEvent::note_on(67, 100),
        ];
        let mut v = VoiceTracker::new();
        for &e in &queue {
            v.apply(e);
        }
        // 60 was switched off; 64 and 67 remain.
        assert_eq!(held(&mut v), vec![64, 67]);
    }
}
