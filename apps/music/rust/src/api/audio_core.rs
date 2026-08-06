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

/// Whether `bytes` look like a loadable SoundFont (`.sf2`) — a RIFF container
/// tagged `sfbk`. This is the same cheap header check the app runs before
/// accepting a user-imported file, mirrored here so the engine can reject a
/// bogus swap **before** it drops the working synth: a SoundFont swap only
/// proceeds if the incoming bytes pass this test, otherwise the current
/// instrument is kept (graceful fallback, no hanging swap).
///
/// A `.sf2` file begins with the ASCII bytes `RIFF`, a 4-byte little-endian
/// chunk size, then the form type `sfbk`. This validates the tags without
/// parsing the (multi-MB) sample data.
pub(crate) fn is_valid_soundfont(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"sfbk"
}

impl VoiceTracker {
    /// Clears the tracker for a SoundFont swap, returning every pitch that was
    /// sounding so the audio thread can release them across the swap (an
    /// all-notes-off) — otherwise a held voice on the outgoing synth would hang.
    /// Semantically an [`AudioEvent::AllOff`]; named for the swap call site.
    pub(crate) fn clear_for_swap(&mut self) -> Vec<u8> {
        self.apply(AudioEvent::AllOff)
    }
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

/// Decode a canonical 16-bit PCM WAV (`RIFF`/`WAVE`) into its sample rate and mono
/// samples (change: add-soundfont-entitlement-previews). This is how the app plays a
/// server-rendered SoundFont **preview clip** through the same cross-platform audio
/// engine as the synth — no third-party audio plugin. Multi-channel input is
/// down-mixed to mono; non-16-bit or malformed input returns `None`.
///
/// Pure and host-testable (the real-time mixing lives in `audio.rs`).
pub(crate) fn decode_wav_pcm(bytes: &[u8]) -> Option<(u32, Vec<i16>)> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let mut channels = 1u16;
    let mut sample_rate = 44_100u32;
    let mut bits = 16u16;
    let mut data: Option<&[u8]> = None;
    // Walk the RIFF chunks after the 12-byte header (word-aligned).
    let mut off = 12usize;
    while off + 8 <= bytes.len() {
        let id = &bytes[off..off + 4];
        let size = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().ok()?) as usize;
        let body = off + 8;
        let end = body.saturating_add(size).min(bytes.len());
        if id == b"fmt " && end >= body + 16 {
            channels = u16::from_le_bytes(bytes[body + 2..body + 4].try_into().ok()?);
            sample_rate = u32::from_le_bytes(bytes[body + 4..body + 8].try_into().ok()?);
            bits = u16::from_le_bytes(bytes[body + 14..body + 16].try_into().ok()?);
        } else if id == b"data" {
            data = Some(&bytes[body..end]);
        }
        off = body + size + (size & 1); // pad byte for odd-sized chunks
    }
    if bits != 16 {
        return None;
    }
    let data = data?;
    let ch = channels.max(1) as usize;
    let frame = 2 * ch;
    let mut pcm = Vec::with_capacity(data.len() / frame.max(1));
    for f in data.chunks_exact(frame) {
        let mut acc = 0i32;
        for c in 0..ch {
            acc += i16::from_le_bytes([f[2 * c], f[2 * c + 1]]) as i32;
        }
        pcm.push((acc / ch as i32) as i16);
    }
    Some((sample_rate, pcm))
}

/// A looping preview-clip voice the audio thread mixes into its output — the clip's
/// PCM resampled (nearest-sample) from its own rate to the device rate, so a preview
/// plays at the right pitch/speed regardless of the output device. Pure DSP (no
/// device), so it is host-testable; `audio.rs` owns the cpal mixing.
#[frb(ignore)]
#[derive(Debug, Clone)]
pub(crate) struct ClipVoice {
    pcm: Vec<i16>,
    /// Fractional read position into `pcm`.
    pos: f64,
    /// Advance per output sample = clip_rate / device_rate.
    step: f64,
    gain: f32,
}

impl ClipVoice {
    /// Builds a voice for `pcm` recorded at `clip_rate`, played at `device_rate` Hz.
    /// Non-positive rates fall back to 44.1 kHz so the voice is always well-formed.
    pub(crate) fn new(pcm: Vec<i16>, clip_rate: u32, device_rate: f32) -> ClipVoice {
        let device_rate = if device_rate > 0.0 {
            device_rate
        } else {
            44_100.0
        };
        let step = if clip_rate > 0 {
            clip_rate as f64 / device_rate as f64
        } else {
            1.0
        };
        ClipVoice {
            pcm,
            pos: 0.0,
            step,
            gain: 0.9,
        }
    }

    /// Next mono sample (looping); `0.0` when the clip is empty.
    pub(crate) fn next_sample(&mut self) -> f32 {
        if self.pcm.is_empty() {
            return 0.0;
        }
        let len = self.pcm.len();
        let idx = (self.pos as usize).min(len - 1);
        let value = self.pcm[idx] as f32 / 32768.0 * self.gain;
        self.pos = (self.pos + self.step).rem_euclid(len as f64);
        value
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

    #[test]
    fn clear_for_swap_releases_every_held_voice() {
        // Swapping the SoundFont must release the outgoing synth's voices so none
        // hang; clear_for_swap returns them all and empties the tracker.
        let mut v = VoiceTracker::new();
        for p in [60, 64, 67] {
            v.apply(AudioEvent::note_on(p, 100));
        }
        let mut released = v.clear_for_swap();
        released.sort_unstable();
        assert_eq!(released, vec![60, 64, 67]);
        // Nothing left to release after the swap cleared it.
        assert!(v.clear_for_swap().is_empty());
    }

    #[test]
    fn valid_soundfont_header_is_accepted() {
        // RIFF <size> sfbk ... — the minimal well-formed SoundFont preamble.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(b"sfbk");
        assert!(is_valid_soundfont(&bytes));
    }

    #[test]
    fn non_soundfont_bytes_are_rejected() {
        // A WAV (RIFF/WAVE), a too-short buffer, and arbitrary junk must all be
        // rejected so a bad swap keeps the working synth.
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&0u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        assert!(!is_valid_soundfont(&wav));
        assert!(!is_valid_soundfont(b"RIFF"));
        assert!(!is_valid_soundfont(b""));
        assert!(!is_valid_soundfont(b"not a soundfont at all"));
    }

    /// A canonical mono 16-bit PCM WAV around `samples` at `rate`.
    fn wav(rate: u32, samples: &[i16]) -> Vec<u8> {
        let data_len = (samples.len() * 2) as u32;
        let mut w = b"RIFF".to_vec();
        w.extend_from_slice(&(36 + data_len).to_le_bytes());
        w.extend_from_slice(b"WAVE");
        w.extend_from_slice(b"fmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes()); // PCM
        w.extend_from_slice(&1u16.to_le_bytes()); // mono
        w.extend_from_slice(&rate.to_le_bytes());
        w.extend_from_slice(&(rate * 2).to_le_bytes()); // byte rate
        w.extend_from_slice(&2u16.to_le_bytes()); // block align
        w.extend_from_slice(&16u16.to_le_bytes()); // bits
        w.extend_from_slice(b"data");
        w.extend_from_slice(&data_len.to_le_bytes());
        for s in samples {
            w.extend_from_slice(&s.to_le_bytes());
        }
        w
    }

    #[test]
    fn decode_wav_pcm_reads_rate_and_mono_samples() {
        let (rate, pcm) = decode_wav_pcm(&wav(44_100, &[0, 100, -100, 32767])).unwrap();
        assert_eq!(rate, 44_100);
        assert_eq!(pcm, vec![0, 100, -100, 32767]);
    }

    #[test]
    fn decode_wav_pcm_rejects_non_wav() {
        assert!(decode_wav_pcm(b"").is_none());
        assert!(decode_wav_pcm(b"RIFF____NOTWAVE__").is_none());
    }

    #[test]
    fn clip_voice_loops_and_resamples() {
        // Same rate → nearest-sample cycles through the buffer and loops.
        let mut v = ClipVoice::new(vec![10, 20, 30], 44_100, 44_100.0);
        let n = |x: i16| x as f32 / 32768.0 * 0.9;
        assert!((v.next_sample() - n(10)).abs() < 1e-6);
        assert!((v.next_sample() - n(20)).abs() < 1e-6);
        assert!((v.next_sample() - n(30)).abs() < 1e-6);
        // Loops back to the start.
        assert!((v.next_sample() - n(10)).abs() < 1e-6);
    }

    #[test]
    fn clip_voice_empty_is_silent() {
        let mut v = ClipVoice::new(Vec::new(), 44_100, 48_000.0);
        assert_eq!(v.next_sample(), 0.0);
    }

    #[test]
    fn clip_voice_handles_degenerate_rates() {
        // A zero device rate falls back to 44.1 kHz; a zero clip rate → step 1.
        let mut v = ClipVoice::new(vec![5], 0, 0.0);
        let n = 5.0f32 / 32768.0 * 0.9;
        assert!((v.next_sample() - n).abs() < 1e-6);
    }
}
