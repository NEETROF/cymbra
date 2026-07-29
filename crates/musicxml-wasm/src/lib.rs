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

//! WebAssembly wrapper exposing the app's notation layout to a browser.
//!
//! The single read-only capability is score `bytes -> geometry`: parse the
//! MusicXML (or `.mxl`) with [`cymbra_musicxml_core::parse`], then lay it out with
//! [`cymbra_musicxml_core::layout_systems`] at a given width. The result is the
//! same `ScoreDocument` + `Vec<System>` the Flutter app draws, so the browser and
//! the app derive notation from one engine.
//!
//! No IO, no network, no FFI to native libraries, and nothing that mutates a
//! score. The pure [`render_geometry`] function is host-testable; the
//! `#[wasm_bindgen]` `render` export (wasm target only) is a thin serializing shell
//! over it.

use cymbra_musicxml_core::{
    PlaybackSchedule, RejectReason, ScoreDocument, System, decode_and_parse, layout_systems,
};
use serde::Serialize;

/// A laid-out score ready to paint: the parsed document plus its per-line system
/// layout for the requested width. Serialized across the wasm boundary as
/// `{ document, systems }`.
#[derive(Debug, Clone, Serialize)]
pub struct RenderedScore {
    pub document: ScoreDocument,
    pub systems: Vec<System>,
}

/// Parse `bytes` and lay the score out at `available_width`, returning the geometry
/// to paint. Pure and read-only — the host-testable core of the `render` entry point.
/// The shared [`decode_and_parse`] gate rejects non-MusicXML/oversized/undecodable
/// input as a typed [`RejectReason`] and decodes `.mxl`. Never panics.
pub fn render_geometry(bytes: &[u8], available_width: f64) -> Result<RenderedScore, RejectReason> {
    let document = decode_and_parse(bytes)?;
    let systems = layout_systems(&document, available_width);
    Ok(RenderedScore { document, systems })
}

/// Derive the playback schedule (onset-sorted notes + measure times + tempo) from
/// `bytes`. Pure and host-testable — the core of the `schedule` entry point.
pub fn schedule_for(bytes: &[u8]) -> Result<PlaybackSchedule, RejectReason> {
    let document = decode_and_parse(bytes)?;
    Ok(cymbra_musicxml_core::schedule(&document))
}

/// wasm-bindgen entry point: `bytes -> { document, systems }` as a JS value.
///
/// Compiled for the wasm target only. Errors surface as a JS string so the JS
/// renderer's `Async` seam can fold them into a non-fatal state.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn render(
    bytes: &[u8],
    available_width: f64,
) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    // RejectReason carries a stable `.code()`; surface it so the JS `Async` seam
    // can distinguish reasons without string matching.
    let rendered = render_geometry(bytes, available_width)
        .map_err(|e| wasm_bindgen::JsValue::from_str(e.code()))?;
    serde_wasm_bindgen::to_value(&rendered)
        .map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
}

/// wasm-bindgen entry point: `bytes -> playback schedule` as a JS value (notes with
/// onset/duration ms, per-measure start times, tempo) for driving the audio + the
/// animated playhead. Compiled for the wasm target only.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn schedule(bytes: &[u8]) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue> {
    let s = schedule_for(bytes).map_err(|e| wasm_bindgen::JsValue::from_str(e.code()))?;
    serde_wasm_bindgen::to_value(&s).map_err(|e| wasm_bindgen::JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal single-staff score: one 4/4 measure of two quarter notes.
    const MINIMAL: &str = r#"<?xml version="1.0"?>
<score-partwise version="3.1">
  <work><work-title>Little Tune</work-title></work>
  <identification><creator type="composer">A. Composer</creator></identification>
  <part-list><score-part id="P1"><part-name>Piano</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>4</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note>
        <pitch><step>C</step><octave>5</octave></pitch>
        <duration>4</duration><voice>1</voice><type>quarter</type><staff>1</staff>
      </note>
      <note>
        <pitch><step>D</step><octave>5</octave></pitch>
        <duration>4</duration><voice>1</voice><type>quarter</type><staff>1</staff>
      </note>
    </measure>
  </part>
</score-partwise>"#;

    #[test]
    fn lays_out_a_valid_score() {
        let rendered = render_geometry(MINIMAL.as_bytes(), 800.0).expect("valid score lays out");
        assert_eq!(rendered.document.measures.len(), 1);
        assert_eq!(rendered.document.measures[0].notes.len(), 2);
        // One measure at a generous width fits on a single system.
        assert_eq!(rendered.systems.len(), 1);
        assert_eq!(rendered.systems[0].measures, vec![0]);
    }

    #[test]
    fn serializes_geometry_to_json() {
        let rendered = render_geometry(MINIMAL.as_bytes(), 800.0).unwrap();
        let json = serde_json::to_string(&rendered).expect("geometry serializes");
        // The JS painter consumes `document` + `systems`.
        assert!(json.contains("\"document\""));
        assert!(json.contains("\"systems\""));
        assert!(json.contains("Little Tune"));
    }

    #[test]
    fn schedule_for_returns_timed_notes() {
        let s = schedule_for(MINIMAL.as_bytes()).expect("valid score schedules");
        assert_eq!(s.notes.len(), 2); // two quarter notes
        assert_eq!(s.notes[0].onset_ms, 0);
        assert!(s.song_end_ms > 0);
        assert_eq!(s.measure_start_ms, vec![0]);
    }

    #[test]
    fn malformed_bytes_error_not_panic() {
        // Arbitrary XML is rejected by the shared gate as unparseable — a typed
        // error, not a panic and not empty geometry.
        let err = render_geometry(b"<score-partwise><unclosed", 800.0);
        assert_eq!(err.unwrap_err(), RejectReason::Unparseable);
    }
}
