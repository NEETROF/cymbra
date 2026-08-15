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

//! Score audio teaser (change: add-score-daily-access-rewards, design D7): the
//! pure helpers that turn a catalog piece into the bounded [`SampleSequence`] the
//! SoundFont render engine already knows how to synthesize.
//!
//! The note timing comes from the shared `musicxml-core` playback schedule — the
//! same one the app and the back-office console use — so the teaser sounds like
//! the piece plays. Everything here is host-testable and deterministic; the
//! `rustysynth` call ([`render_score_preview_wav`]) and object I/O are the
//! coverage-excluded glue, exactly as for the SoundFont previews.

use cymbra_musicxml_core::{DEFAULT_VELOCITY, PlaybackSchedule};

use crate::soundfont_preview::{Note, PREVIEW_SAMPLE_RATE, SampleSequence, encode_preview};
use crate::soundfont_synth::render_preview_pcm;

/// Object key of a catalog piece's audio teaser, stored in the SCORE store beside
/// the piece's bytes and distinct from them. Served by `GET /scores/{id}/preview`.
///
/// **Versioned by the render instant** (`rendered_at`, the row's
/// `preview_rendered_at`): the store's local warm cache treats an object as
/// immutable under its key, so a re-render must land under a NEW key or every
/// node that already read the old clip would keep serving it. The route derives
/// the key from the row's marker; the renderer deletes the previous key.
pub fn score_preview_object_key(
    catalog_id: &str,
    rendered_at: chrono::DateTime<chrono::Utc>,
) -> String {
    format!(
        "catalog-preview/{catalog_id}/{}.wav",
        rendered_at.timestamp_millis()
    )
}

/// Short release tail after the last sounding note so the clip does not cut off
/// abruptly. Included in the bounded length.
pub const RELEASE_MS: u32 = 300;

/// The teaser configuration in force (read per render from the feature flags
/// through [`ScorePreviewConfigSource`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScorePreviewConfig {
    /// Maximum clip length in milliseconds (release tail included).
    pub max_ms: u32,
    /// The ACCEPTED catalog SoundFont the clip is rendered with. Empty = the
    /// teasers are dormant: nothing is rendered, nothing is broken.
    pub soundfont_id: String,
}

impl Default for ScorePreviewConfig {
    fn default() -> Self {
        Self {
            max_ms: 30_000,
            soundfont_id: String::new(),
        }
    }
}

/// Where the teaser configuration comes from at render time (the server and the
/// worker implement it over the flag service).
pub trait ScorePreviewConfigSource: Send + Sync {
    fn score_preview_config(&self) -> ScorePreviewConfig;
}

/// A fixed configuration (tests, or a deployment with no flag store).
pub struct FixedScorePreviewConfig(pub ScorePreviewConfig);

impl ScorePreviewConfigSource for FixedScorePreviewConfig {
    fn score_preview_config(&self) -> ScorePreviewConfig {
        self.0.clone()
    }
}

/// Convert a playback schedule into the bounded teaser sequence.
///
/// - The clip **starts at the first sounding note**: every onset is shifted by the
///   earliest one, so leading rests / an empty pickup bar are skipped and the
///   bounded window is spent on music, not silence.
/// - Notes are taken in onset order and every note whose (shifted) onset falls
///   before the sounding bound (`max_ms - RELEASE_MS`) is kept; a held note is
///   truncated at that bound, so no note sounds past it.
/// - A repeated pitch is cut at the next onset of the same pitch (the synth loop
///   pairs note-on/note-off by pitch), so overlapping repeats do not swallow each
///   other's note-off.
/// - `total_ms` = last sounding end + [`RELEASE_MS`], never above `max_ms`.
/// - An empty schedule (or `max_ms` too short to sound anything) yields an empty
///   sequence with `total_ms = 0` — the caller renders nothing.
///
/// Pure and deterministic: the same schedule + bound always yields the same
/// sequence.
pub fn preview_sequence(schedule: &PlaybackSchedule, max_ms: u32) -> SampleSequence {
    let bound = max_ms.saturating_sub(RELEASE_MS);
    if bound == 0 {
        return SampleSequence {
            notes: Vec::new(),
            total_ms: 0,
        };
    }
    // Skip the leading silence: the teaser starts on the first pitched note.
    let first_onset = schedule
        .notes
        .iter()
        .filter(|n| n.duration_ms > 0 && (0..=127).contains(&n.midi))
        .map(|n| n.onset_ms)
        .min()
        .unwrap_or(0);
    let mut timed: Vec<(u32, u32, u8)> = schedule
        .notes
        .iter()
        .filter(|n| n.duration_ms > 0)
        .filter_map(|n| {
            u8::try_from(n.midi)
                .ok()
                .filter(|p| *p <= 127)
                .map(|p| (n.onset_ms - first_onset, n.duration_ms, p))
        })
        .filter(|(onset, _, _)| *onset < bound)
        .collect();
    timed.sort_by_key(|(onset, _, pitch)| (*onset, *pitch));
    let mut notes: Vec<Note> = Vec::with_capacity(timed.len());
    for (i, (onset, duration, pitch)) in timed.iter().enumerate() {
        // Cut a repeated pitch at its next onset so the note-offs pair cleanly.
        let next_same = timed[i + 1..]
            .iter()
            .find(|(_, _, p)| p == pitch)
            .map(|(o, _, _)| *o);
        let mut end = onset.saturating_add(*duration).min(bound);
        if let Some(next) = next_same {
            end = end.min(next);
        }
        if end <= *onset {
            continue;
        }
        notes.push(Note {
            pitch: *pitch,
            velocity: DEFAULT_VELOCITY,
            start_ms: *onset,
            duration_ms: end - onset,
        });
    }
    let last_end = notes
        .iter()
        .map(|n| n.start_ms + n.duration_ms)
        .max()
        .unwrap_or(0);
    let total_ms = if notes.is_empty() {
        0
    } else {
        (last_end + RELEASE_MS).min(max_ms)
    };
    SampleSequence { notes, total_ms }
}

/// Render the teaser of `score_bytes` (MusicXML or `.mxl`) with `font_bytes` to a
/// WAV clip bounded to `max_ms`. `Ok(None)` when the piece sounds nothing within
/// the bound (no clip is stored). Coverage-excluded glue: parse → schedule →
/// [`preview_sequence`] → `rustysynth` → [`encode_preview`].
pub fn render_score_preview_wav(
    font_bytes: &[u8],
    score_bytes: &[u8],
    max_ms: u32,
) -> anyhow::Result<Option<Vec<u8>>> {
    let doc = cymbra_musicxml_core::decode_and_parse(score_bytes)
        .map_err(|r| anyhow::anyhow!("score not renderable: {r:?}"))?;
    let schedule = cymbra_musicxml_core::schedule(&doc);
    let seq = preview_sequence(&schedule, max_ms);
    if seq.notes.is_empty() {
        return Ok(None);
    }
    let pcm = render_preview_pcm(font_bytes, &seq)?;
    Ok(Some(encode_preview(&pcm, PREVIEW_SAMPLE_RATE)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cymbra_musicxml_core::TimedNote;

    fn tn(midi: i32, onset_ms: u32, duration_ms: u32) -> TimedNote {
        TimedNote {
            midi,
            onset_ms,
            duration_ms,
            staff: 1,
            measure_index: 0,
            note_index: 0,
        }
    }

    fn sched(notes: Vec<TimedNote>) -> PlaybackSchedule {
        let song_end_ms = notes
            .iter()
            .map(|n| n.onset_ms + n.duration_ms)
            .max()
            .unwrap_or(0);
        PlaybackSchedule {
            notes,
            measure_start_ms: vec![0],
            song_end_ms,
            bpm: 90,
        }
    }

    #[test]
    fn object_key_is_versioned_by_the_render_instant() {
        let at = chrono::DateTime::from_timestamp_millis(1_700_000_000_123).unwrap();
        assert_eq!(
            score_preview_object_key("abc", at),
            "catalog-preview/abc/1700000000123.wav"
        );
        let later = chrono::DateTime::from_timestamp_millis(1_700_000_000_124).unwrap();
        assert_ne!(
            score_preview_object_key("abc", at),
            score_preview_object_key("abc", later)
        );
    }

    #[test]
    fn keeps_notes_before_the_bound_and_truncates_held_ones() {
        let s = sched(vec![tn(60, 0, 1000), tn(64, 500, 5000), tn(67, 4000, 100)]);
        let seq = preview_sequence(&s, 2_300); // bound = 2000
        assert_eq!(seq.notes.len(), 2, "the note at 4000 ms is past the bound");
        assert_eq!(seq.notes[0].duration_ms, 1000);
        assert_eq!(seq.notes[1].start_ms + seq.notes[1].duration_ms, 2000);
        assert!(seq.total_ms <= 2_300);
        assert_eq!(seq.total_ms, 2000 + RELEASE_MS);
        assert!(seq.notes.iter().all(|n| n.start_ms < 2000));
    }

    #[test]
    fn total_never_exceeds_max_and_release_is_included() {
        let s = sched(vec![tn(60, 0, 400)]);
        let seq = preview_sequence(&s, 30_000);
        assert_eq!(seq.total_ms, 400 + RELEASE_MS);
        let seq2 = preview_sequence(&s, 500);
        assert_eq!(seq2.total_ms, 500);
        assert_eq!(seq2.notes[0].duration_ms, 200);
    }

    #[test]
    fn leading_silence_is_skipped_so_the_clip_starts_on_the_first_note() {
        // Three bars of rest, then music at 3000 ms: the teaser starts at 0.
        let s = sched(vec![
            tn(60, 3000, 500),
            tn(64, 3500, 500),
            tn(67, 4000, 500),
        ]);
        let seq = preview_sequence(&s, 30_000);
        assert_eq!(seq.notes[0].start_ms, 0);
        assert_eq!(seq.notes[1].start_ms, 500);
        assert_eq!(seq.notes[2].start_ms, 1000);
        assert_eq!(seq.total_ms, 1500 + RELEASE_MS);
        // The bound is spent on music: with a 1.3 s window only the first two
        // (shifted) notes fit, not zero notes.
        let short = preview_sequence(&s, 1_300);
        assert_eq!(short.notes.len(), 2);
    }

    #[test]
    fn deterministic() {
        let s = sched(vec![tn(60, 0, 1000), tn(62, 100, 200), tn(64, 300, 700)]);
        assert_eq!(preview_sequence(&s, 10_000), preview_sequence(&s, 10_000));
    }

    #[test]
    fn empty_schedule_or_tiny_bound_yields_nothing() {
        assert_eq!(preview_sequence(&sched(vec![]), 30_000).notes.len(), 0);
        assert_eq!(preview_sequence(&sched(vec![]), 30_000).total_ms, 0);
        let s = sched(vec![tn(60, 0, 1000)]);
        assert_eq!(preview_sequence(&s, RELEASE_MS).total_ms, 0);
    }

    #[test]
    fn repeated_pitch_is_cut_at_its_next_onset() {
        let s = sched(vec![tn(60, 0, 1000), tn(60, 500, 500)]);
        let seq = preview_sequence(&s, 10_000);
        assert_eq!(seq.notes[0].duration_ms, 500);
        assert_eq!(seq.notes[1].start_ms, 500);
    }

    #[test]
    fn out_of_range_midi_is_dropped() {
        let s = sched(vec![tn(-1, 0, 100), tn(200, 0, 100), tn(72, 0, 100)]);
        let seq = preview_sequence(&s, 10_000);
        assert_eq!(seq.notes.len(), 1);
        assert_eq!(seq.notes[0].pitch, 72);
    }

    #[test]
    fn config_defaults_are_dormant_thirty_seconds() {
        let c = ScorePreviewConfig::default();
        assert_eq!(c.max_ms, 30_000);
        assert!(c.soundfont_id.is_empty());
        assert_eq!(FixedScorePreviewConfig(c.clone()).score_preview_config(), c);
    }

    #[test]
    fn unparseable_score_is_an_error_not_a_panic() {
        assert!(render_score_preview_wav(b"not a font", b"<nope>", 1000).is_err());
    }
}
