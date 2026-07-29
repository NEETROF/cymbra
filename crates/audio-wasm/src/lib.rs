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
//! The pure [`render_pcm`] is host-testable; the `#[wasm_bindgen]` `render` export
//! (wasm target only) is a thin shell returning a `Float32Array`.

use std::io::Cursor;
use std::sync::Arc;

use cymbra_musicxml_core::{DEFAULT_VELOCITY, decode_and_parse, schedule};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

/// MIDI channel the piano preset plays on (matches the app).
const PIANO_CHANNEL: i32 = 0;
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

/// Render `score_bytes` to interleaved-stereo (`L,R,L,R,…`) PCM at `sample_rate`,
/// synthesised from `sf2_bytes`. Returns a stable error **code** string
/// (`RejectReason::code()`, or `bad_soundfont`/`synth_init`) on failure — never
/// panics. The buffer length is `frames * 2`.
pub fn render_pcm(
    score_bytes: &[u8],
    sf2_bytes: &[u8],
    sample_rate: u32,
) -> Result<Vec<f32>, String> {
    let document = decode_and_parse(score_bytes).map_err(|e| e.code().to_string())?;
    let sched = schedule(&document);

    let mut cursor = Cursor::new(sf2_bytes);
    let sound_font =
        Arc::new(SoundFont::new(&mut cursor).map_err(|_| "bad_soundfont".to_string())?);
    let settings = SynthesizerSettings::new(sample_rate as i32);
    let mut synth =
        Synthesizer::new(&sound_font, &settings).map_err(|_| "synth_init".to_string())?;

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
                synth.note_on(PIANO_CHANNEL, e.key, i32::from(DEFAULT_VELOCITY));
            } else {
                synth.note_off(PIANO_CHANNEL, e.key);
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

/// wasm-bindgen entry point: interleaved-stereo PCM as a `Float32Array`. wasm target
/// only. Errors surface as the stable code string.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render(
    score_bytes: &[u8],
    sf2_bytes: &[u8],
    sample_rate: u32,
) -> Result<Vec<f32>, wasm_bindgen::JsValue> {
    render_pcm(score_bytes, sf2_bytes, sample_rate).map_err(|e| wasm_bindgen::JsValue::from_str(&e))
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
    }
}
