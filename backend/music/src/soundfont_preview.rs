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

//! Preview-clip building blocks (change: add-soundfont-entitlement-previews).
//!
//! The **pure**, host-testable pieces of the server-side preview render: the fixed
//! sample sequence (the same short musical phrase for every font, so previews are
//! comparable and deterministic), the sample-offset scheduling derived from it, and
//! the WAV/PCM encoder. The actual device-free `rustysynth` synthesis call lives in
//! [`crate::soundfont_synth`] (coverage-excluded glue) and consumes what this module
//! produces.

/// Render sample rate for previews (Hz). A fixed 44.1 kHz keeps the render
/// deterministic and the clip universally playable.
pub const PREVIEW_SAMPLE_RATE: u32 = 44_100;

/// Public object key for a font's preview clip, distinct from the private `{id}.sf2`
/// font bytes. Served openly by `GET /soundfonts/{id}/preview`.
pub fn preview_object_key(id: &str) -> String {
    format!("{id}.preview.wav")
}

/// One note in the fixed [`SampleSequence`]: a MIDI pitch played from `start_ms` for
/// `duration_ms` at `velocity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pub pitch: u8,
    pub velocity: u8,
    pub start_ms: u32,
    pub duration_ms: u32,
}

/// A synth control event at a sample offset from the clip start — the scheduled form
/// the synth loop consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    NoteOn { pitch: u8, velocity: u8 },
    NoteOff { pitch: u8 },
}

/// The fixed preview phrase: a handful of notes and the total clip length. The same
/// phrase is rendered with every font, so clips are comparable and each render is
/// deterministic (its length is a pure function of this sequence + sample rate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleSequence {
    pub notes: Vec<Note>,
    /// Total clip length in milliseconds (covers the last note plus a short release
    /// tail so it doesn't cut off abruptly).
    pub total_ms: u32,
}

/// The canonical preview phrase for a **keyboard**-family font: a rising C-major
/// arpeggio (C4·E4·G4·C5) followed by a sustained C-major triad — ~2.4 s, the
/// same for every font.
pub fn sample_sequence() -> SampleSequence {
    const V: u8 = 100;
    // Arpeggio: four notes, 300 ms apart, each sounding 400 ms.
    let arpeggio = [60u8, 64, 67, 72]
        .into_iter()
        .enumerate()
        .map(|(i, pitch)| Note {
            pitch,
            velocity: V,
            start_ms: 300 * i as u32,
            duration_ms: 400,
        });
    // Final triad at 1300 ms, held 900 ms.
    let chord = [60u8, 64, 67].into_iter().map(|pitch| Note {
        pitch,
        velocity: V,
        start_ms: 1300,
        duration_ms: 900,
    });
    SampleSequence {
        notes: arpeggio.chain(chord).collect(),
        // Last note ends at 2200 ms; add a 200 ms release tail.
        total_ms: 2400,
    }
}

/// The canonical preview groove for a **percussion**-family font (change:
/// add-drum-audio-channel): one 4/4 bar of a basic rock pattern in General MIDI
/// numbers — kick 36, snare 38, closed hi-hat 42 — at 120 bpm, synthesized on
/// the drum channel. Fixed like the melodic phrase, so kit clips stay
/// comparable and deterministic; the melodic phrase through a kit font would be
/// silence or nonsense (a kit's presets live in bank 128).
pub fn drum_sample_sequence() -> SampleSequence {
    /// General MIDI percussion numbers of the groove.
    const KICK: u8 = 36;
    const SNARE: u8 = 38;
    const CLOSED_HAT: u8 = 42;
    let mut notes = Vec::new();
    // Closed hi-hat on every eighth (120 bpm → 250 ms per eighth).
    for i in 0..8u32 {
        notes.push(Note {
            pitch: CLOSED_HAT,
            velocity: 80,
            start_ms: 250 * i,
            duration_ms: 60,
        });
    }
    // Kick on beat 1, beat 3 and the "and" of 3 (0 / 1000 / 1250 ms).
    for start_ms in [0u32, 1000, 1250] {
        notes.push(Note {
            pitch: KICK,
            velocity: 110,
            start_ms,
            duration_ms: 150,
        });
    }
    // Snare backbeat on 2 and 4 (500 / 1500 ms).
    for start_ms in [500u32, 1500] {
        notes.push(Note {
            pitch: SNARE,
            velocity: 100,
            start_ms,
            duration_ms: 150,
        });
    }
    SampleSequence {
        notes,
        // The bar ends at 2000 ms; keep the melodic phrase's total so the two
        // families' clips are the same length.
        total_ms: 2400,
    }
}

/// The fixed sample sequence and synthesis channel for a font's instrument
/// family (change: add-drum-audio-channel): a `percussion`-family font plays
/// [`drum_sample_sequence`] on the shared drum channel; **any** other value —
/// `keyboard`, or a legacy row not yet migrated — keeps the existing melodic
/// phrase on the melodic channel, byte-identically.
pub fn sample_sequence_for_family(family: &str) -> (SampleSequence, i32) {
    if family == crate::soundfont::PERCUSSION_FAMILY {
        (drum_sample_sequence(), cymbra_musicxml_core::DRUM_CHANNEL)
    } else {
        (sample_sequence(), cymbra_musicxml_core::MELODIC_CHANNEL)
    }
}

/// Total number of mono sample frames a clip of `seq` occupies at `sample_rate`.
pub fn total_samples(seq: &SampleSequence, sample_rate: u32) -> usize {
    (seq.total_ms as u64 * sample_rate as u64 / 1000) as usize
}

/// Convert a millisecond offset to a sample offset at `sample_rate`.
fn ms_to_samples(ms: u32, sample_rate: u32) -> usize {
    (ms as u64 * sample_rate as u64 / 1000) as usize
}

/// Schedule `seq` into `(sample_offset, event)` pairs sorted by offset (note-offs at a
/// given offset ordered before note-ons is unnecessary here — the phrase never re-uses
/// a pitch while it is still sounding). Pure, so the synth loop is a thin driver.
pub fn scheduled_events(seq: &SampleSequence, sample_rate: u32) -> Vec<(usize, Event)> {
    let mut events: Vec<(usize, Event)> = Vec::with_capacity(seq.notes.len() * 2);
    for n in &seq.notes {
        let on = ms_to_samples(n.start_ms, sample_rate);
        let off = ms_to_samples(n.start_ms + n.duration_ms, sample_rate);
        events.push((
            on,
            Event::NoteOn {
                pitch: n.pitch,
                velocity: n.velocity,
            },
        ));
        events.push((off, Event::NoteOff { pitch: n.pitch }));
    }
    events.sort_by_key(|(offset, _)| *offset);
    events
}

/// Encode mono 16-bit PCM `pcm` at `sample_rate` as a canonical little-endian WAV
/// (RIFF/WAVE, PCM format). No extra codec dependency; the clip is short so size is
/// acceptable.
pub fn encode_preview(pcm: &[i16], sample_rate: u32) -> Vec<u8> {
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;
    let byte_rate = sample_rate * CHANNELS as u32 * (BITS_PER_SAMPLE as u32 / 8);
    let block_align = CHANNELS * (BITS_PER_SAMPLE / 8);
    let data_len = (pcm.len() * 2) as u32;
    // 44-byte canonical header + PCM payload.
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes()); // chunk size
    out.extend_from_slice(b"WAVE");
    // fmt subchunk
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // subchunk1 size (PCM)
    out.extend_from_slice(&1u16.to_le_bytes()); // audio format = PCM
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    // data subchunk
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in pcm {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_object_key_is_public_wav_namespace() {
        assert_eq!(preview_object_key("ydp-grand"), "ydp-grand.preview.wav");
    }

    #[test]
    fn sample_sequence_is_a_non_empty_fixed_phrase() {
        let seq = sample_sequence();
        assert!(!seq.notes.is_empty());
        assert_eq!(seq.total_ms, 2400);
        // The phrase is fixed — two calls yield the identical sequence.
        assert_eq!(seq, sample_sequence());
    }

    #[test]
    fn drum_sequence_is_a_fixed_groove_of_kick_snare_hat() {
        let seq = drum_sample_sequence();
        assert!(!seq.notes.is_empty());
        // The groove speaks General MIDI percussion numbers only.
        for n in &seq.notes {
            assert!(
                [36, 38, 42].contains(&n.pitch),
                "unexpected number {}",
                n.pitch
            );
        }
        // All three voices are present (kick, snare, hi-hat).
        for pitch in [36u8, 38, 42] {
            assert!(seq.notes.iter().any(|n| n.pitch == pitch));
        }
        // Same length as the melodic phrase, and fixed across calls.
        assert_eq!(seq.total_ms, sample_sequence().total_ms);
        assert_eq!(seq, drum_sample_sequence());
        // Every note ends inside the clip.
        assert!(
            seq.notes
                .iter()
                .all(|n| n.start_ms + n.duration_ms <= seq.total_ms)
        );
    }

    #[test]
    fn family_selects_the_sequence_and_channel() {
        // Percussion → the groove on the drum channel.
        let (seq, channel) = sample_sequence_for_family("percussion");
        assert_eq!(seq, drum_sample_sequence());
        assert_eq!(channel, cymbra_musicxml_core::DRUM_CHANNEL);
        // Keyboard keeps the existing phrase on the melodic channel,
        // byte-identically (the pre-change hardcoded channel 0).
        let (seq, channel) = sample_sequence_for_family("keyboard");
        assert_eq!(seq, sample_sequence());
        assert_eq!(channel, cymbra_musicxml_core::MELODIC_CHANNEL);
        assert_eq!(channel, 0);
        // A legacy not-yet-migrated spelling degrades to the melodic phrase —
        // never a guess at percussion.
        let (seq, channel) = sample_sequence_for_family("piano");
        assert_eq!(seq, sample_sequence());
        assert_eq!(channel, 0);
    }

    #[test]
    fn total_samples_is_deterministic_from_the_sequence() {
        let seq = sample_sequence();
        // 2400 ms at 44_100 Hz = 105_840 frames — a pure function of the sequence, so
        // the same font + sequence always renders an equivalent-length clip.
        assert_eq!(total_samples(&seq, PREVIEW_SAMPLE_RATE), 105_840);
        assert_eq!(
            total_samples(&seq, PREVIEW_SAMPLE_RATE),
            total_samples(&sample_sequence(), PREVIEW_SAMPLE_RATE)
        );
    }

    #[test]
    fn scheduled_events_pairs_each_note_on_off_sorted() {
        let seq = sample_sequence();
        let events = scheduled_events(&seq, PREVIEW_SAMPLE_RATE);
        // One on + one off per note.
        assert_eq!(events.len(), seq.notes.len() * 2);
        // Offsets are non-decreasing.
        assert!(events.windows(2).all(|w| w[0].0 <= w[1].0));
        // First event is the first note's onset at sample 0.
        assert_eq!(
            events[0],
            (
                0,
                Event::NoteOn {
                    pitch: 60,
                    velocity: 100
                }
            )
        );
        // Equal counts of ons and offs.
        let ons = events
            .iter()
            .filter(|(_, e)| matches!(e, Event::NoteOn { .. }))
            .count();
        let offs = events.len() - ons;
        assert_eq!(ons, offs);
    }

    #[test]
    fn encode_preview_writes_a_valid_wav_header_and_payload() {
        let pcm: Vec<i16> = vec![0, 1, -1, 100, -100];
        let wav = encode_preview(&pcm, PREVIEW_SAMPLE_RATE);
        // 44-byte header + 2 bytes per sample.
        assert_eq!(wav.len(), 44 + pcm.len() * 2);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        // RIFF chunk size = 36 + data_len.
        let riff_size = u32::from_le_bytes(wav[4..8].try_into().unwrap());
        assert_eq!(riff_size, 36 + (pcm.len() * 2) as u32);
        // Audio format = PCM (1), mono (1 channel), 16-bit.
        assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
        // Sample rate round-trips.
        assert_eq!(
            u32::from_le_bytes(wav[24..28].try_into().unwrap()),
            PREVIEW_SAMPLE_RATE
        );
        // data length header matches the payload.
        let data_len = u32::from_le_bytes(wav[40..44].try_into().unwrap());
        assert_eq!(data_len, (pcm.len() * 2) as u32);
        // First sample (0) encodes little-endian.
        assert_eq!(&wav[44..46], &0i16.to_le_bytes());
    }

    #[test]
    fn encode_preview_of_empty_pcm_is_a_header_only_wav() {
        let wav = encode_preview(&[], PREVIEW_SAMPLE_RATE);
        assert_eq!(wav.len(), 44);
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 0);
    }
}
