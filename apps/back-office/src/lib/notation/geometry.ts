// TypeScript mirror of the notation geometry the wasm renderer returns
// (`cymbra-musicxml-wasm::RenderedScore`, serialized by serde with the Rust field
// names). This is the single source of truth for the layout — produced by the app's
// own `layout_systems`, so the console draws exactly what the app draws. Keys are
// snake_case to match serde; optionals arrive as `null` (serde_json) or `undefined`
// (serde-wasm-bindgen), so callers compare with `== null`.

export interface Pitch {
  step: string; // "C".."B"
  octave: number;
  alter: number; // semitone alteration (− flat, + sharp)
}

export interface Clef {
  staff: number;
  sign: string; // "G" | "F" | "C" | "percussion"
  line: number;
}

export interface TimeSignature {
  beats: number;
  beat_type: number;
}

export interface Attributes {
  divisions: number;
  clefs: Clef[];
  key_fifths: number;
  time: TimeSignature;
}

export type StemDir = "Up" | "Down";
export type BeamState = "Begin" | "Continue" | "End";

export interface Tuplet {
  actual: number;
  normal: number;
}

export interface Lyric {
  syllabic?: string | null;
  text: string;
}

/** How an unpitched note's head is engraved (change: add-drum-notation-render),
 * mirroring `cymbra-musicxml-core::HeadClass`. Derived once in the crate beside
 * the resolved General MIDI number, so this painter never owns GM ranges. */
export type HeadClass = "Oval" | "X" | "XOpen";

/** A percussion note's written staff position and resolved sound (`<unpitched>`),
 * mirroring `cymbra-musicxml-core::Unpitched` (change: add-unpitched-notation). */
export interface Unpitched {
  /** `display-step`: the written staff placement's diatonic step (A–G). */
  display_step: string;
  /** `display-octave`: the written staff placement's octave. */
  display_octave: number;
  /** General MIDI percussion number (0-based), or null when unresolved. */
  gm_number?: number | null;
  /** Engraved head class the crate derived from the GM number: "X" for cymbals,
   * "XOpen" for the open hi-hat (GM 46, x head + open mark), "Oval" for drums
   * and unresolved notes. Absent on pre-drum-render wasm builds (treated as
   * "Oval" — the ordinary head, never a re-derivation from GM ranges). */
  head_class?: HeadClass;
}

export interface NoteEvent {
  staff: number;
  voice: number;
  position_divisions: number;
  pitch?: Pitch | null;
  is_rest: boolean;
  is_chord: boolean;
  /** Grace note (`<grace/>`): ornamental, occupies no musical time (duration 0
   * at its principal's position); engraved offset left of the principal. */
  is_grace: boolean;
  duration_divisions: number;
  note_type?: string | null;
  dots: number;
  accidental?: string | null;
  tie_start: boolean;
  tie_stop: boolean;
  slur_start: boolean;
  slur_stop: boolean;
  tuplet?: Tuplet | null;
  stem?: StemDir | null;
  beams: BeamState[];
  lyric?: Lyric | null;
  /** Percussion channel (change: add-drums-access): the written placement of an
   * unpitched note. Distinct from `pitch` — a note is exactly one of pitched,
   * unpitched, or a rest. Absent/null on pitched notes and pre-drums wasm builds. */
  unpitched?: Unpitched | null;
  /** The note's `<instrument id>` reference when present. */
  instrument_id?: string | null;
}

/** Repeat notation engraved on one measure (serde snake_case), mirroring
 *  `cymbra-musicxml-core::RepeatMarks`. Absent on pre-repeat wasm builds. */
export interface RepeatMarks {
  forward: boolean;
  backward_times: number;
  ending_start: number[];
  ending_stop: boolean;
  ending_discontinue: boolean;
  measure_repeat_of?: number | null;
  measure_repeat_slashes: number;
  segno: boolean;
  coda: boolean;
  sound_dacapo: boolean;
  sound_dalsegno: boolean;
  sound_tocoda: boolean;
  sound_fine: boolean;
  sound_forward_repeat: boolean;
}

export interface NotationMeasure {
  index: number;
  notes: NoteEvent[];
  /** Repeat notation on this measure; absent on pre-repeat wasm builds. */
  repeats?: RepeatMarks;
  // directions are present in the model but not drawn by the v1 painter.
  clefs: Clef[];
  // Key signature (fifths) in force during this measure, carried from the last
  // <key> change — so the painter draws the right armure per system and a key
  // change where it differs from the previous measure.
  key_fifths: number;
  min_width: number;
}

export interface ScoreMeta {
  title?: string | null;
  composer?: string | null;
}

export interface ScoreDocument {
  meta: ScoreMeta;
  staves: number;
  attributes: Attributes;
  measures: NotationMeasure[];
}

export interface System {
  measures: number[];
  staves: number;
}

/** What the wasm `render(bytes, width)` entry point returns. */
export interface RenderedScore {
  document: ScoreDocument;
  systems: System[];
}
