import { describe, expect, it } from "vitest";
import * as S from "@/lib/notation/smufl";
import type { NoteEvent } from "@/lib/notation/geometry";

const note = (over: Partial<NoteEvent>): NoteEvent => ({
  staff: 1,
  voice: 1,
  position_divisions: 0,
  pitch: { step: "C", octave: 5, alter: 0 },
  is_rest: false,
  is_chord: false,
  is_grace: false,
  duration_divisions: 4,
  note_type: "quarter",
  dots: 0,
  accidental: null,
  tie_start: false,
  tie_stop: false,
  slur_start: false,
  slur_stop: false,
  tuplet: null,
  stem: null,
  beams: [],
  lyric: null,
  ...over,
});

describe("smufl glyph helpers", () => {
  it("maps clefs (G/F/C, default treble)", () => {
    expect(S.clefGlyph("G")).toBe(S.gClef);
    expect(S.clefGlyph("F")).toBe(S.fClef);
    expect(S.clefGlyph("C")).toBe(S.cClef);
    expect(S.clefGlyph("?")).toBe(S.gClef);
  });

  it("maps accidental tokens (unknown → null)", () => {
    expect(S.accidentalGlyph("flat")).toBe(S.accidentalFlat);
    expect(S.accidentalGlyph("sharp")).toBe(S.accidentalSharp);
    expect(S.accidentalGlyph("natural")).toBe(S.accidentalNatural);
    expect(S.accidentalGlyph("double-sharp")).toBe(S.accidentalDoubleSharp);
    expect(S.accidentalGlyph("flat-flat")).toBe(S.accidentalDoubleFlat);
    expect(S.accidentalGlyph("weird")).toBeNull();
  });

  it("picks the head glyph from note-type, else from duration", () => {
    expect(S.headGlyph(note({ note_type: "whole" }), 4)).toBe(S.noteheadWhole);
    expect(S.headGlyph(note({ note_type: "half" }), 4)).toBe(S.noteheadHalf);
    expect(S.headGlyph(note({ note_type: "quarter" }), 4)).toBe(S.noteheadBlack);
    // No note_type → derive from duration vs divisions.
    expect(S.headGlyph(note({ note_type: null, duration_divisions: 16 }), 4)).toBe(S.noteheadWhole);
    expect(S.headGlyph(note({ note_type: null, duration_divisions: 8 }), 4)).toBe(S.noteheadHalf);
    expect(S.headGlyph(note({ note_type: null, duration_divisions: 4 }), 4)).toBe(S.noteheadBlack);
  });

  it("picks flags by type + stem direction, and flag counts", () => {
    expect(S.flagGlyph(note({ note_type: "eighth" }), true)).toBe(S.flag8thUp);
    expect(S.flagGlyph(note({ note_type: "eighth" }), false)).toBe(S.flag8thDown);
    expect(S.flagGlyph(note({ note_type: "16th" }), true)).toBe(S.flag16thUp);
    expect(S.flagGlyph(note({ note_type: "32nd" }), false)).toBe(S.flag32ndDown);
    expect(S.flagGlyph(note({ note_type: "quarter" }), true)).toBeNull();
    expect(S.flagCount(note({ note_type: "eighth" }))).toBe(1);
    expect(S.flagCount(note({ note_type: "16th" }))).toBe(2);
    expect(S.flagCount(note({ note_type: "64th" }))).toBe(4);
    expect(S.flagCount(note({ note_type: "quarter" }))).toBe(0);
  });

  it("maps rests by type (default quarter)", () => {
    expect(S.restGlyph(note({ note_type: "whole" }))).toBe(S.restWhole);
    expect(S.restGlyph(note({ note_type: "half" }))).toBe(S.restHalf);
    expect(S.restGlyph(note({ note_type: "eighth" }))).toBe(S.rest8th);
    expect(S.restGlyph(note({ note_type: "16th" }))).toBe(S.rest16th);
    expect(S.restGlyph(note({ note_type: "quarter" }))).toBe(S.restQuarter);
  });

  it("builds time-signature and tuplet number glyph strings", () => {
    // Each digit maps to a SMuFL codepoint (timeSig0 = U+E080, tuplet0 = U+E880).
    expect(S.timeSigNumber(4)).toBe(String.fromCodePoint(0xe084));
    expect(S.timeSigNumber(12)).toBe(String.fromCodePoint(0xe081) + String.fromCodePoint(0xe082));
    expect(S.tupletNumber(3)).toBe(String.fromCodePoint(0xe883));
  });
});
