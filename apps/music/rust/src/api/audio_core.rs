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
    /// Sound a one-shot metronome click. `accent` marks the downbeat (higher and
    /// louder). Self-terminating — no matching release event.
    Click { accent: bool },
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
            // The metronome click is not a tracked piano voice (it is mixed in
            // separately and decays on its own), so it releases nothing here.
            AudioEvent::Click { .. } => Vec::new(),
        }
    }
}

/// Metronome click tone frequencies (Hz). The accented downbeat is pitched higher
/// than a normal beat so the start of the measure is audible.
pub(crate) const CLICK_FREQ_NORMAL: f32 = 1000.0;
pub(crate) const CLICK_FREQ_ACCENT: f32 = 1500.0;

/// Peak amplitudes for the click envelope; the accent is a touch louder as well
/// as higher in pitch.
pub(crate) const CLICK_AMP_NORMAL: f32 = 0.25;
pub(crate) const CLICK_AMP_ACCENT: f32 = 0.40;

/// Click length in seconds — short and percussive, so it self-terminates well
/// within a beat at any musical tempo.
pub(crate) const CLICK_SECS: f32 = 0.035;

/// A one-shot, self-terminating metronome click — an enveloped sine burst that
/// the audio thread mixes into its output **independently of the SoundFont**, so
/// a beat sounds without using a piano voice and is unmistakably distinct from the
/// music. Accent (downbeat) clicks are higher in pitch and amplitude than normal
/// beats.
///
/// Pure DSP with no device/synth dependency, so it is host-testable (and counted
/// by `cargo llvm-cov`); `audio.rs` only owns the mixing into the cpal buffer.
#[frb(ignore)]
#[derive(Debug, Clone)]
pub(crate) struct ClickVoice {
    /// Samples still to emit (counts down to 0, then the voice is inactive).
    remaining: u32,
    /// Total samples in the click, for the decay envelope.
    total: u32,
    /// Sine phase in radians.
    phase: f32,
    /// Phase increment per sample (`2π·f / sample_rate`).
    phase_inc: f32,
    /// Peak amplitude.
    amplitude: f32,
}

impl ClickVoice {
    /// Builds a click for the given `accent` at `sample_rate` Hz. A non-positive
    /// sample rate falls back to 44.1 kHz so the voice is always well-formed.
    pub(crate) fn new(accent: bool, sample_rate: f32) -> ClickVoice {
        let sample_rate = if sample_rate > 0.0 {
            sample_rate
        } else {
            44_100.0
        };
        let freq = if accent {
            CLICK_FREQ_ACCENT
        } else {
            CLICK_FREQ_NORMAL
        };
        let amplitude = if accent {
            CLICK_AMP_ACCENT
        } else {
            CLICK_AMP_NORMAL
        };
        let total = ((CLICK_SECS * sample_rate).round() as u32).max(1);
        ClickVoice {
            remaining: total,
            total,
            phase: 0.0,
            phase_inc: std::f32::consts::TAU * freq / sample_rate,
            amplitude,
        }
    }

    /// Next mono sample of the click, advancing its envelope. Returns `0.0` once
    /// the click has finished (and on every later call), so it leaves no hanging
    /// voice.
    pub(crate) fn next_sample(&mut self) -> f32 {
        if self.remaining == 0 {
            return 0.0;
        }
        // Linear decay (fast, percussive): 1.0 at onset → ~0.0 at the end.
        let envelope = self.remaining as f32 / self.total as f32;
        let value = self.amplitude * envelope * self.phase.sin();
        self.phase += self.phase_inc;
        if self.phase >= std::f32::consts::TAU {
            self.phase -= std::f32::consts::TAU;
        }
        self.remaining -= 1;
        value
    }

    /// Whether the click still has samples to emit.
    pub(crate) fn is_active(&self) -> bool {
        self.remaining > 0
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

    #[test]
    fn click_event_releases_no_voice() {
        // A click is mixed in separately, not tracked as a piano voice, so it must
        // never release one (and must not disturb the held set).
        let mut v = VoiceTracker::new();
        v.apply(AudioEvent::note_on(60, 100));
        assert!(v.apply(AudioEvent::Click { accent: true }).is_empty());
        assert_eq!(held(&mut v), vec![60]);
    }

    /// Total energy of a freshly built click, rendered to completion.
    fn click_energy(accent: bool, sample_rate: f32) -> f32 {
        let mut c = ClickVoice::new(accent, sample_rate);
        let mut energy = 0.0;
        while c.is_active() {
            let s = c.next_sample();
            energy += s * s;
        }
        energy
    }

    #[test]
    fn click_is_audible() {
        // A normal-beat click renders non-silent audio.
        assert!(click_energy(false, 44_100.0) > 0.0);
    }

    #[test]
    fn accent_click_is_louder_than_normal() {
        // The downbeat must be audibly distinct: more energy than a normal beat.
        let accent = click_energy(true, 44_100.0);
        let normal = click_energy(false, 44_100.0);
        assert!(
            accent > normal,
            "accent energy {accent} should exceed normal {normal}"
        );
    }

    #[test]
    fn click_self_terminates_and_stays_silent() {
        let mut c = ClickVoice::new(false, 44_100.0);
        // Drain exactly the click's length.
        for _ in 0..(CLICK_SECS * 44_100.0).round() as u32 {
            c.next_sample();
        }
        assert!(!c.is_active());
        // Every later call is a silent no-op — no panic, no hanging voice.
        assert_eq!(c.next_sample(), 0.0);
        assert_eq!(c.next_sample(), 0.0);
    }

    #[test]
    fn click_handles_nonpositive_sample_rate() {
        // A degenerate sample rate falls back rather than producing an empty or
        // NaN-laden voice.
        let mut c = ClickVoice::new(true, 0.0);
        assert!(c.is_active());
        assert!(c.next_sample().is_finite());
    }
}
