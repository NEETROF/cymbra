// SMuFL (Bravura) glyph codepoints + engraving metrics, mirroring the Flutter app's
// smufl.dart so the web preview draws the same notation. Glyphs are positioned in
// *staff spaces*; Bravura's em = 4 staff spaces, so a glyph drawn at font-size
// `4 * staffSpace` with the alphabetic baseline at the SMuFL origin (y = 0) renders
// at the right size — which is exactly SVG <text>'s default baseline behaviour.

import type { NoteEvent } from "./geometry";

export const FONT_FAMILY = "Bravura";

// --- Glyph codepoints (SMuFL standard) ---------------------------------------
export const noteheadBlack = "\u{E0A4}";
export const noteheadHalf = "\u{E0A3}";
export const noteheadWhole = "\u{E0A2}";

export const gClef = "\u{E050}";
export const fClef = "\u{E062}";
export const cClef = "\u{E05C}";

export const flag8thUp = "\u{E240}";
export const flag8thDown = "\u{E241}";
export const flag16thUp = "\u{E242}";
export const flag16thDown = "\u{E243}";
export const flag32ndUp = "\u{E244}";
export const flag32ndDown = "\u{E245}";

export const accidentalFlat = "\u{E260}";
export const accidentalNatural = "\u{E261}";
export const accidentalSharp = "\u{E262}";
export const accidentalDoubleSharp = "\u{E263}";
export const accidentalDoubleFlat = "\u{E264}";

export const restWhole = "\u{E4E3}";
export const restHalf = "\u{E4E4}";
export const restQuarter = "\u{E4E5}";
export const rest8th = "\u{E4E6}";
export const rest16th = "\u{E4E7}";

export const augmentationDot = "\u{E1E7}";

// --- Engraving defaults (staff spaces), from bravura_metadata.json -----------
export const staffLineThickness = 0.13;
export const stemThickness = 0.12;
export const beamThickness = 0.5;
export const legerLineThickness = 0.16;
export const legerLineExtension = 0.4;
// Repeat notation (change: add-repeat-unrolling).
export const segno = "\u{E047}";
export const coda = "\u{E048}";
export const repeat1Bar = "\u{E500}";
export const repeat2Bars = "\u{E501}";
export const thickBarlineThickness = 0.5;

export const thinBarlineThickness = 0.16;

// noteheadBlack bbox width + stem-attachment anchors (staff spaces).
export const noteheadWidth = 1.18;
export const stemUpAnchorX = 1.18;
export const stemUpAnchorY = 0.168;
export const stemDownAnchorX = 0.0;
export const stemDownAnchorY = -0.168;

/** Time-signature number as glyph string (timeSig0 = U+E080). */
export function timeSigNumber(n: number): string {
  return String(n)
    .split("")
    .map((c) => String.fromCodePoint(0xe080 + Number(c)))
    .join("");
}

/** Tuplet number as glyph string (tuplet0 = U+E880). */
export function tupletNumber(n: number): string {
  return String(n)
    .split("")
    .map((c) => String.fromCodePoint(0xe880 + Number(c)))
    .join("");
}

// Key-signature accidental vertical positions (diatonic staff steps from the treble
// bottom line E4, each step = half a staff space), in fifths order. Bass = −2.
export const sharpSteps = [8, 5, 9, 6, 3, 7, 4]; // F C G D A E B
export const flatSteps = [4, 7, 3, 6, 2, 5, 1]; // B E A D G C F

/** Clef glyph for a MusicXML clef sign (G/F/C), defaulting to treble. */
export function clefGlyph(sign: string): string {
  switch (sign) {
    case "F":
      return fClef;
    case "C":
      return cClef;
    default:
      return gClef;
  }
}

/** Accidental glyph for a MusicXML accidental token, or null if unsupported. */
export function accidentalGlyph(token: string): string | null {
  switch (token) {
    case "flat":
      return accidentalFlat;
    case "natural":
      return accidentalNatural;
    case "sharp":
      return accidentalSharp;
    case "double-sharp":
    case "sharp-sharp":
      return accidentalDoubleSharp;
    case "flat-flat":
      return accidentalDoubleFlat;
    default:
      return null;
  }
}

/** Note-head glyph for a note, from its type or (absent a type) its duration. */
export function headGlyph(note: NoteEvent, divisions: number): string {
  switch (note.note_type) {
    case "whole":
      return noteheadWhole;
    case "half":
      return noteheadHalf;
    case null:
    case undefined:
      if (note.duration_divisions >= 4 * divisions) return noteheadWhole;
      if (note.duration_divisions >= 2 * divisions) return noteheadHalf;
      return noteheadBlack;
    default:
      return noteheadBlack;
  }
}

/** Flag glyph for an unbeamed eighth-or-shorter note, or null. */
export function flagGlyph(note: NoteEvent, up: boolean): string | null {
  switch (note.note_type) {
    case "eighth":
      return up ? flag8thUp : flag8thDown;
    case "16th":
      return up ? flag16thUp : flag16thDown;
    case "32nd":
      return up ? flag32ndUp : flag32ndDown;
    default:
      return null;
  }
}

/** Number of flags/beams a note carries (16th = 2, …). */
export function flagCount(note: NoteEvent): number {
  switch (note.note_type) {
    case "eighth":
      return 1;
    case "16th":
      return 2;
    case "32nd":
      return 3;
    case "64th":
      return 4;
    default:
      return 0;
  }
}

/** Rest glyph for a note-type token, defaulting to a quarter rest. */
export function restGlyph(note: NoteEvent): string {
  switch (note.note_type) {
    case "whole":
      return restWhole;
    case "half":
      return restHalf;
    case "eighth":
      return rest8th;
    case "16th":
      return rest16th;
    default:
      return restQuarter;
  }
}
