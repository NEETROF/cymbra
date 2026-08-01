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
  sign: string; // "G" | "F" | "C"
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

export interface NoteEvent {
  staff: number;
  voice: number;
  position_divisions: number;
  pitch?: Pitch | null;
  is_rest: boolean;
  is_chord: boolean;
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
}

export interface NotationMeasure {
  index: number;
  notes: NoteEvent[];
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
