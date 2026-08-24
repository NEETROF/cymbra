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

//! Offline audio render for the back-office preview.
//!
//! [`render_pcm`] takes score bytes + a SoundFont and renders the whole piece to one
//! interleaved-stereo PCM buffer, using the same `rustysynth` engine and the same
//! playback schedule (`cymbra-musicxml-core`) as the app. The browser plays the buffer
//! through a Web Audio `AudioBufferSourceNode` — Web Audio is the output sink here,
//! replacing the app's native `cpal`. No live seek/tempo; a preview is short and
//! rendered once on demand.
//!
//! The pure [`render_pcm`] is host-testable; the `#[wasm_bindgen]` exports (wasm target
//! only) are thin shells returning a `Float32Array`.
//!
//! **Parsed-SoundFont cache.** Parsing a `.sf2` is the expensive part of a render (tens
//! of MB of samples, seconds of CPU for the big grands) and it does not depend on the
//! score. The wasm shell therefore keeps the last parsed font in a one-slot cache keyed
//! by an opaque string the caller owns (`load_soundfont` / `has_soundfont` /
//! `render_cached`), so replaying, re-rendering after a score change, or auditioning a
//! second piece with the same instrument pays the parse **once** — and the caller only
//! has to post the bytes across the worker boundary on a miss.

use std::io::Cursor;
use std::sync::Arc;

use cymbra_musicxml_core::{
    DEFAULT_VELOCITY, DRUM_CHANNEL, InstrumentKind, MELODIC_CHANNEL, decode_and_parse,
    instrument_of, schedule,
};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

/// Render quantum (frames). Note on/off are applied on block boundaries — 64 frames
/// is ~1.5ms at 44.1kHz, inaudible.
const BLOCK: usize = 64;
/// Extra tail rendered after the last note ends, so releases ring out.
const RELEASE_TAIL_MS: u32 = 2000;

/// A frame-stamped synth event.
struct Ev {
    frame: usize,
    on: bool,
    key: i32,
}

/// Parse a `.sf2` into a shareable [`SoundFont`], or the stable `bad_soundfont` code.
/// Split out of [`render_pcm`] so the parse can be cached across renders.
pub fn parse_soundfont(sf2_bytes: &[u8]) -> Result<Arc<SoundFont>, String> {
    let mut cursor = Cursor::new(sf2_bytes);
    Ok(Arc::new(
        SoundFont::new(&mut cursor).map_err(|_| "bad_soundfont".to_string())?,
    ))
}

/// Render `score_bytes` to interleaved-stereo (`L,R,L,R,…`) PCM at `sample_rate`,
/// synthesised from `sf2_bytes`. Returns a stable error **code** string
/// (`RejectReason::code()`, or `bad_soundfont`/`synth_init`) on failure — never
/// panics. The buffer length is `frames * 2`.
pub fn render_pcm(
    score_bytes: &[u8],
    sf2_bytes: &[u8],
    sample_rate: u32,
) -> Result<Vec<f32>, String> {
    // Score first, so an unparseable score is reported as such even when the SoundFont
    // is *also* rubbish — the caller shows a per-cause message.
    let document = decode_and_parse(score_bytes).map_err(|e| e.code().to_string())?;
    render_document(&document, &parse_soundfont(sf2_bytes)?, sample_rate)
}

/// [`render_pcm`] against an **already-parsed** SoundFont — the hot path when the same
/// instrument renders several scores (or the same score again).
pub fn render_pcm_with(
    score_bytes: &[u8],
    sound_font: &Arc<SoundFont>,
    sample_rate: u32,
) -> Result<Vec<f32>, String> {
    let document = decode_and_parse(score_bytes).map_err(|e| e.code().to_string())?;
    render_document(&document, sound_font, sample_rate)
}

/// The synth channel the whole render plays on, resolved from the document's own
/// instrument classification (change: add-drum-audio-channel) — never from the
/// caller's context: a percussion-classified score sounds on the shared drum
/// channel (where rustysynth resolves presets in bank 128), everything else on
/// the melodic channel exactly as before. The schedule emits unpitched notes only
/// for percussion-classified scores, so a mixed (`Unknown`) score cannot reach
/// the drum channel by construction.
fn render_channel(document: &cymbra_musicxml_core::ScoreDocument) -> i32 {
    match instrument_of(document) {
        InstrumentKind::Percussion => DRUM_CHANNEL,
        InstrumentKind::Keyboard | InstrumentKind::Unknown => MELODIC_CHANNEL,
    }
}

/// Synthesise an already-parsed score with an already-parsed SoundFont.
fn render_document(
    document: &cymbra_musicxml_core::ScoreDocument,
    sound_font: &Arc<SoundFont>,
    sample_rate: u32,
) -> Result<Vec<f32>, String> {
    let sched = schedule(document);
    let channel = render_channel(document);

    let settings = SynthesizerSettings::new(sample_rate as i32);
    let mut synth =
        Synthesizer::new(sound_font, &settings).map_err(|_| "synth_init".to_string())?;

    let sr = f64::from(sample_rate);
    let total_ms = sched.song_end_ms + RELEASE_TAIL_MS;
    let total_frames = ((f64::from(total_ms) / 1000.0) * sr).ceil() as usize;
    if total_frames == 0 {
        return Ok(Vec::new());
    }

    // Note-on at onset, note-off at end (at least one frame later).
    let frame_of = |ms: u32| ((f64::from(ms) / 1000.0) * sr) as usize;
    let mut events: Vec<Ev> = Vec::with_capacity(sched.notes.len() * 2);
    for n in &sched.notes {
        let on = frame_of(n.onset_ms);
        let off = frame_of(n.onset_ms + n.duration_ms).max(on + 1);
        events.push(Ev {
            frame: on,
            on: true,
            key: n.midi,
        });
        events.push(Ev {
            frame: off,
            on: false,
            key: n.midi,
        });
    }
    events.sort_by_key(|e| e.frame);

    let mut left = vec![0f32; BLOCK];
    let mut right = vec![0f32; BLOCK];
    let mut out = vec![0f32; total_frames * 2];
    let mut ei = 0usize;
    let mut frame = 0usize;
    while frame < total_frames {
        let block = BLOCK.min(total_frames - frame);
        while ei < events.len() && events[ei].frame < frame + block {
            let e = &events[ei];
            if e.on {
                synth.note_on(channel, e.key, i32::from(DEFAULT_VELOCITY));
            } else {
                synth.note_off(channel, e.key);
            }
            ei += 1;
        }
        synth.render(&mut left[..block], &mut right[..block]);
        for i in 0..block {
            out[(frame + i) * 2] = left[i];
            out[(frame + i) * 2 + 1] = right[i];
        }
        frame += block;
    }
    Ok(out)
}

// The last parsed SoundFont, keyed by the caller's opaque font key. One slot: the
// console plays one instrument at a time, and holding two decoded grands would double
// an already large wasm heap. A miss is not an error the user ever sees — the caller
// re-posts the bytes and reloads (see `render_cached`'s `font_not_loaded`).
#[cfg(target_arch = "wasm32")]
thread_local! {
    static FONT_CACHE: std::cell::RefCell<Option<(String, Arc<SoundFont>)>> =
        const { std::cell::RefCell::new(None) };
}

/// Whether the font cached under `key` is loaded — lets the caller skip re-posting tens
/// of MB across the worker boundary. wasm target only.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn has_soundfont(key: &str) -> bool {
    FONT_CACHE.with_borrow(|slot| slot.as_ref().is_some_and(|(k, _)| k == key))
}

/// Parse `sf2_bytes` and cache them under `key`, replacing whatever was cached. A
/// re-load of the key already held is a no-op (the parse is what we're avoiding).
/// wasm target only; a bad font surfaces the stable `bad_soundfont` code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn load_soundfont(key: &str, sf2_bytes: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
    if has_soundfont(key) {
        return Ok(());
    }
    let font = parse_soundfont(sf2_bytes).map_err(|e| wasm_bindgen::JsValue::from_str(&e))?;
    FONT_CACHE.with_borrow_mut(|slot| *slot = Some((key.to_string(), font)));
    Ok(())
}

/// wasm-bindgen entry point: interleaved-stereo PCM as a `Float32Array`, rendered with
/// the font cached under `key`. wasm target only. Errors surface as the stable code
/// string — `font_not_loaded` when the key was evicted, which the caller answers by
/// re-posting the bytes through [`load_soundfont`].
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render_cached(
    score_bytes: &[u8],
    key: &str,
    sample_rate: u32,
) -> Result<Vec<f32>, wasm_bindgen::JsValue> {
    let font = FONT_CACHE.with_borrow(|slot| match slot {
        Some((k, font)) if k == key => Some(Arc::clone(font)),
        _ => None,
    });
    let Some(font) = font else {
        return Err(wasm_bindgen::JsValue::from_str("font_not_loaded"));
    };
    render_pcm_with(score_bytes, &font, sample_rate)
        .map_err(|e| wasm_bindgen::JsValue::from_str(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"<?xml version="1.0"?>
<score-partwise version="3.1">
  <part-list><score-part id="P1"/></part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>4</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note><pitch><step>C</step><octave>5</octave></pitch><duration>4</duration><type>quarter</type><staff>1</staff></note>
    </measure>
  </part>
</score-partwise>"#;

    /// A one-note percussion score: a GM-38 snare (`<midi-unpitched>` is 1-based,
    /// as conforming exporters write it) under a percussion clef.
    const PERCUSSION: &str = r#"<?xml version="1.0"?>
<score-partwise version="4.0">
  <part-list>
    <score-part id="P1">
      <part-name>Drums</part-name>
      <score-instrument id="P1-I39"><instrument-name>Snare</instrument-name></score-instrument>
      <midi-instrument id="P1-I39"><midi-channel>10</midi-channel><midi-unpitched>39</midi-unpitched></midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes><divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>percussion</sign><line>2</line></clef>
      </attributes>
      <note><unpitched><display-step>C</display-step><display-octave>5</display-octave></unpitched><duration>4</duration><instrument id="P1-I39"/><voice>1</voice><type>whole</type></note>
    </measure>
  </part>
</score-partwise>"#;

    // --- Minimal renderable SoundFont (change: add-drum-audio-channel) --------
    //
    // rustysynth degrades silently when a channel's bank holds no preset (design
    // context), so the channel-routing tests need a real parseable font whose
    // presets sit in chosen banks — the app's 57 MB piano can't provide a
    // bank-128 kit and stays behind `--ignored`. This builder emits the smallest
    // `.sf2` rustysynth accepts: one square-wave sample, one instrument, and one
    // preset per requested `(bank, patch)`, all sharing the instrument.

    /// One RIFF sub-chunk: fourcc + little-endian size + body.
    fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(body);
        out
    }

    /// A LIST chunk of `kind` wrapping already-encoded sub-chunks.
    fn list(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut inner = kind.to_vec();
        inner.extend_from_slice(body);
        chunk(b"LIST", &inner)
    }

    /// A fixed-length zero-padded name field.
    fn name(n: &str, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        out[..n.len()].copy_from_slice(n.as_bytes());
        out
    }

    /// Instrument 0 of [`test_sf2`]: an audible square wave.
    const AUDIBLE: u16 = 0;
    /// Instrument 1 of [`test_sf2`]: all-zero samples — renders exact silence,
    /// so a preset wired to it proves which preset a channel actually picked.
    const SILENT: u16 = 1;

    /// Builds a minimal, fully parseable `.sf2` holding one preset per
    /// `(bank, patch, instrument)`. Bank 128 is where the drum channel resolves
    /// presets; wiring the two banks to [`AUDIBLE`] vs [`SILENT`] makes the
    /// channel routing observable in the PCM (rustysynth falls back to *some*
    /// preset when a bank is empty, so bare non-silence alone would not
    /// discriminate).
    fn test_sf2(presets: &[(u16, u16, u16)]) -> Vec<u8> {
        // 2000 frames of a period-100 square wave, then 2000 frames of silence.
        let mut wave: Vec<u8> = (0..2000i32)
            .flat_map(|i| {
                let s: i16 = if (i / 50) % 2 == 0 { 12_000 } else { -12_000 };
                s.to_le_bytes()
            })
            .collect();
        wave.extend_from_slice(&[0u8; 4000]);

        // phdr: one 38-byte record per preset, then the EOP terminal whose
        // zone start closes the last preset's zone span.
        let mut phdr = Vec::new();
        for (i, (bank, patch, _)) in presets.iter().enumerate() {
            phdr.extend_from_slice(&name("P", 20));
            phdr.extend_from_slice(&patch.to_le_bytes());
            phdr.extend_from_slice(&bank.to_le_bytes());
            phdr.extend_from_slice(&(i as u16).to_le_bytes()); // zone start
            phdr.extend_from_slice(&[0u8; 12]); // library/genre/morphology
        }
        phdr.extend_from_slice(&name("EOP", 20));
        phdr.extend_from_slice(&[0u8; 2 + 2]);
        phdr.extend_from_slice(&(presets.len() as u16).to_le_bytes());
        phdr.extend_from_slice(&[0u8; 12]);

        // pbag/pgen: each preset zone carries the single generator
        // "instrument N" (41 = INSTRUMENT, which must come last in a zone).
        let mut pbag = Vec::new();
        for i in 0..=presets.len() as u16 {
            pbag.extend_from_slice(&i.to_le_bytes()); // generator index
            pbag.extend_from_slice(&0u16.to_le_bytes()); // modulator index
        }
        let mut pgen = Vec::new();
        for (_, _, instrument) in presets {
            pgen.extend_from_slice(&41u16.to_le_bytes());
            pgen.extend_from_slice(&instrument.to_le_bytes());
        }
        pgen.extend_from_slice(&[0u8; 4]); // terminal

        // Two instruments, each a single zone naming its sample (53 = SAMPLE_ID).
        let mut inst = Vec::new();
        for (i, n) in ["I0", "I1"].iter().enumerate() {
            inst.extend_from_slice(&name(n, 20));
            inst.extend_from_slice(&(i as u16).to_le_bytes());
        }
        inst.extend_from_slice(&name("EOI", 20));
        inst.extend_from_slice(&2u16.to_le_bytes());
        let ibag: Vec<u8> = [0u16, 0, 1, 0, 2, 0]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let mut igen = Vec::new();
        for sample in [0u16, 1] {
            igen.extend_from_slice(&53u16.to_le_bytes());
            igen.extend_from_slice(&sample.to_le_bytes());
        }
        igen.extend_from_slice(&[0u8; 4]); // terminal

        // shdr: the two samples (mono, root key 60) + the EOS terminal record.
        let mut shdr = Vec::new();
        for (n, start, end) in [("S0", 0i32, 1999i32), ("S1", 2000, 3999)] {
            shdr.extend_from_slice(&name(n, 20));
            for v in [start, end, start, end, 44_100] {
                shdr.extend_from_slice(&v.to_le_bytes());
            }
            shdr.push(60); // original pitch
            shdr.push(0); // pitch correction
            shdr.extend_from_slice(&0u16.to_le_bytes()); // link
            shdr.extend_from_slice(&1u16.to_le_bytes()); // type: mono
        }
        shdr.extend_from_slice(&[0u8; 46]); // EOS terminal

        let mut pdta = chunk(b"phdr", &phdr);
        pdta.extend(chunk(b"pbag", &pbag));
        pdta.extend(chunk(b"pmod", &[0u8; 10]));
        pdta.extend(chunk(b"pgen", &pgen));
        pdta.extend(chunk(b"inst", &inst));
        pdta.extend(chunk(b"ibag", &ibag));
        pdta.extend(chunk(b"imod", &[0u8; 10]));
        pdta.extend(chunk(b"igen", &igen));
        pdta.extend(chunk(b"shdr", &shdr));

        let mut body = b"sfbk".to_vec();
        body.extend(list(b"INFO", &chunk(b"ifil", &[2, 0, 0, 0])));
        body.extend(list(b"sdta", &chunk(b"smpl", &wave)));
        body.extend(list(b"pdta", &pdta));
        chunk(b"RIFF", &body)
    }

    #[test]
    fn rejects_bad_score() {
        let err = render_pcm(b"<score-partwise><unclosed", b"not a soundfont", 44_100);
        assert_eq!(err.unwrap_err(), "unparseable");
    }

    #[test]
    fn rejects_bad_soundfont() {
        // Valid score, garbage SoundFont → typed error, no panic.
        let err = render_pcm(MINIMAL.as_bytes(), b"PK\x03\x04 not an sf2", 44_100);
        assert_eq!(err.unwrap_err(), "bad_soundfont");
    }

    #[test]
    fn parse_soundfont_rejects_garbage() {
        assert_eq!(
            parse_soundfont(b"PK\x03\x04 not an sf2").unwrap_err(),
            "bad_soundfont"
        );
    }

    // Pins the keyboard path to the exact behaviour it had before the channel
    // became document-resolved: the retired local `PIANO_CHANNEL = 0` copy is
    // replaced by a resolution that yields the same value for every
    // non-percussion document, so the synth receives byte-identical calls (same
    // font, same events, same channel value ⇒ identical PCM by construction).
    #[test]
    fn keyboard_document_resolves_the_melodic_channel() {
        let doc = decode_and_parse(MINIMAL.as_bytes()).expect("parses");
        assert_eq!(render_channel(&doc), MELODIC_CHANNEL);
        assert_eq!(MELODIC_CHANNEL, 0); // the value the deleted local copy held
    }

    #[test]
    fn percussion_document_resolves_the_drum_channel() {
        let doc = decode_and_parse(PERCUSSION.as_bytes()).expect("parses");
        assert_eq!(render_channel(&doc), DRUM_CHANNEL);
    }

    #[test]
    fn percussion_fixture_renders_non_silence_through_a_kit_shaped_font() {
        // The kit preset (bank 128) is audible and the melodic preset is silent:
        // sound in the PCM proves the render reached the drum channel's bank —
        // through the old hardcoded piano channel, bank 0's silent preset would
        // have answered and the render would be all zeros.
        let font = test_sf2(&[(0, 0, SILENT), (128, 0, AUDIBLE)]);
        let pcm = render_pcm(PERCUSSION.as_bytes(), &font, 44_100).expect("renders");
        assert!(pcm.iter().any(|&s| s != 0.0), "drum channel produces sound");
    }

    #[test]
    fn keyboard_fixture_still_renders_non_silence_on_the_melodic_channel() {
        // The mirror of the percussion test — only the melodic preset is
        // audible, so sound proves the keyboard fixture stayed on channel 0.
        let font = test_sf2(&[(0, 0, AUDIBLE), (128, 0, SILENT)]);
        let pcm = render_pcm(MINIMAL.as_bytes(), &font, 44_100).expect("renders");
        assert!(
            pcm.iter().any(|&s| s != 0.0),
            "melodic channel produces sound"
        );
        // The cached path stays byte-identical: same synth, same font, same
        // resolved channel — only the parse is skipped.
        let parsed = parse_soundfont(&font).expect("parses");
        let again = render_pcm_with(MINIMAL.as_bytes(), &parsed, 44_100).expect("renders");
        assert_eq!(pcm, again);
    }

    #[test]
    fn the_channel_routing_is_observable_in_the_rendered_pcm() {
        // Swap which bank holds the audible instrument: the percussion fixture
        // goes silent, the keyboard fixture sounds — the two families provably
        // resolve their presets in different banks, not through any fallback.
        let font = test_sf2(&[(0, 0, AUDIBLE), (128, 0, SILENT)]);
        let drums = render_pcm(PERCUSSION.as_bytes(), &font, 44_100).expect("renders");
        assert!(
            drums.iter().all(|&s| s == 0.0),
            "kit-less bank 128 is silent"
        );
    }

    // Full render against the app's real SoundFont. Ignored by default (loads a
    // ~57MB asset); run with `cargo test -p cymbra-audio-wasm -- --ignored`.
    #[test]
    #[ignore]
    fn renders_pcm_with_real_soundfont() {
        let sf2 = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../apps/music/assets/soundfonts/UprightPianoKW-20220221.sf2"
        ))
        .expect("app SoundFont present");
        let pcm = render_pcm(MINIMAL.as_bytes(), &sf2, 44_100).expect("renders");
        // One quarter note at 90bpm (~667ms) + 2s tail ≈ 2.7s of stereo audio.
        assert!(pcm.len() > 44_100 * 2, "non-trivial buffer");
        assert_eq!(pcm.len() % 2, 0, "interleaved stereo");
        assert!(pcm.iter().any(|&s| s != 0.0), "produces sound");
        // The cached path (parse once, render many) is byte-identical — it is the same
        // synth fed the same font, only the parse is skipped.
        let font = parse_soundfont(&sf2).expect("parses");
        let again = render_pcm_with(MINIMAL.as_bytes(), &font, 44_100).expect("renders");
        assert_eq!(pcm, again);
    }
}
