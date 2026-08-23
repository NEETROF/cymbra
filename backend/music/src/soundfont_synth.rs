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

//! Device-free `rustysynth` render of a preview clip (change:
//! add-soundfont-entitlement-previews).
//!
//! This is the **coverage-excluded glue**: it loads a SoundFont from bytes, drives the
//! synthesizer through the fixed [`SampleSequence`](crate::soundfont_preview::SampleSequence)
//! (scheduled by the covered [`scheduled_events`](crate::soundfont_preview::scheduled_events))
//! with no audio device, and collects mono 16-bit PCM. The pure scheduling / length /
//! WAV-encoding pieces it depends on live in [`crate::soundfont_preview`] and are
//! host-tested there. `rustysynth` is already vendored client-side
//! (`apps/music/rust`), so the version/licensing is known-good.

use std::io::Cursor;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::soundfont_preview::{
    Event, PREVIEW_SAMPLE_RATE, SampleSequence, encode_preview, sample_sequence_for_family,
    scheduled_events, total_samples,
};

/// Convert a clamped `f32` sample in `[-1.0, 1.0]` to `i16` PCM.
fn f32_to_i16(s: f32) -> i16 {
    (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

/// Render `seq` with the SoundFont in `font_bytes` to mono 16-bit PCM at
/// [`PREVIEW_SAMPLE_RATE`], headlessly (no audio device), on `channel` — the
/// shared `cymbra_musicxml_core::MELODIC_CHANNEL` for keyboard material or
/// `DRUM_CHANNEL` for percussion, where rustysynth resolves presets in bank 128
/// (change: add-drum-audio-channel; formerly a hardcoded `PREVIEW_CHANNEL = 0`).
/// Deterministic: the same font + sequence yields an equivalent-length clip
/// (length is `total_samples(seq, rate)`).
pub fn render_preview_pcm(
    font_bytes: &[u8],
    seq: &SampleSequence,
    channel: i32,
) -> Result<Vec<i16>> {
    let mut reader = Cursor::new(font_bytes);
    let sound_font =
        Arc::new(SoundFont::new(&mut reader).map_err(|e| anyhow!("invalid SoundFont: {e}"))?);
    let settings = SynthesizerSettings::new(PREVIEW_SAMPLE_RATE as i32);
    let mut synth =
        Synthesizer::new(&sound_font, &settings).map_err(|e| anyhow!("synth init: {e}"))?;

    let total = total_samples(seq, PREVIEW_SAMPLE_RATE);
    let events = scheduled_events(seq, PREVIEW_SAMPLE_RATE);
    let mut pcm: Vec<i16> = Vec::with_capacity(total);
    let mut cursor = 0usize;
    let mut ei = 0usize;
    // Reused scratch buffers so we don't reallocate per block.
    let mut left: Vec<f32> = Vec::new();
    let mut right: Vec<f32> = Vec::new();

    while cursor < total {
        // Apply every event due at or before the current sample offset.
        while ei < events.len() && events[ei].0 <= cursor {
            match events[ei].1 {
                Event::NoteOn { pitch, velocity } => {
                    synth.note_on(channel, pitch as i32, velocity as i32)
                }
                Event::NoteOff { pitch } => synth.note_off(channel, pitch as i32),
            }
            ei += 1;
        }
        // Render up to the next event boundary (or the end of the clip).
        let next = events
            .get(ei)
            .map(|(off, _)| (*off).min(total))
            .unwrap_or(total);
        let block = next.saturating_sub(cursor).max(1);
        if left.len() < block {
            left.resize(block, 0.0);
            right.resize(block, 0.0);
        }
        let l = &mut left[..block];
        let r = &mut right[..block];
        synth.render(l, r);
        for i in 0..block {
            // Down-mix stereo to mono.
            pcm.push(f32_to_i16((l[i] + r[i]) * 0.5));
        }
        cursor += block;
    }
    Ok(pcm)
}

/// Render the fixed preview sequence of the font's instrument `family` (change:
/// add-drum-audio-channel: a `percussion` font plays the drum groove on the drum
/// channel; anything else keeps the melodic phrase on the melodic channel) and
/// encode it as a WAV clip — the one call the delivery/upload path uses to
/// produce a font's preview object.
pub fn render_preview_wav(font_bytes: &[u8], family: &str) -> Result<Vec<u8>> {
    let (seq, channel) = sample_sequence_for_family(family);
    let pcm = render_preview_pcm(font_bytes, &seq, channel)?;
    Ok(encode_preview(&pcm, PREVIEW_SAMPLE_RATE))
}
