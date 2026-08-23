import { describe, expect, it } from "vitest";
import { renderNotation } from "@/lib/notation/painter";
import type { NotationMeasure, RenderedScore, RepeatMarks } from "@/lib/notation/geometry";
import { type PlaybackSchedule, startOfWrittenMeasure, writtenMeasureAt } from "@/lib/notation/schedule";

// --- Schedule mapping (played slots ↔ written measures) ----------------------

const schedule: PlaybackSchedule = {
  notes: [],
  measure_start_ms: [0, 2000, 4000],
  written_measure: [0, 0, 1], // measure 0 repeated, then measure 1
  song_end_ms: 6000,
  bpm: 120,
};

describe("repeat-aware schedule mapping", () => {
  it("maps a played slot to the written measure it performs", () => {
    expect(writtenMeasureAt(schedule, 0)).toBe(0);
    expect(writtenMeasureAt(schedule, 1)).toBe(0); // second pass, same bar
    expect(writtenMeasureAt(schedule, 2)).toBe(1);
  });

  it("is the identity without a mapping (no repeats / older wasm)", () => {
    const plain: PlaybackSchedule = { ...schedule, written_measure: undefined };
    expect(writtenMeasureAt(plain, 2)).toBe(2);
  });

  it("seeks a written measure to its first played slot", () => {
    expect(startOfWrittenMeasure(schedule, 0)).toBe(0); // first pass, not 2000
    expect(startOfWrittenMeasure(schedule, 1)).toBe(4000);
    expect(startOfWrittenMeasure(schedule, 9)).toBeNull();
  });
});

// --- Repeat notation engraving ----------------------------------------------

const marks = (over: Partial<RepeatMarks>): RepeatMarks => ({
  forward: false,
  backward_times: 0,
  ending_start: [],
  ending_stop: false,
  ending_discontinue: false,
  measure_repeat_of: null,
  measure_repeat_slashes: 0,
  segno: false,
  coda: false,
  sound_dacapo: false,
  sound_dalsegno: false,
  sound_tocoda: false,
  sound_fine: false,
  sound_forward_repeat: false,
  ...over,
});

const measure = (index: number, repeats?: RepeatMarks): NotationMeasure => ({
  index,
  notes: [],
  repeats,
  clefs: [],
  key_fifths: 0,
  min_width: 160,
});

const rendered = (measures: NotationMeasure[]): RenderedScore => ({
  document: {
    meta: { title: null, composer: null },
    staves: 1,
    attributes: {
      divisions: 4,
      clefs: [{ staff: 1, sign: "G", line: 2 }],
      key_fifths: 0,
      time: { beats: 4, beat_type: 4 },
    },
    measures,
  },
  systems: [{ measures: measures.map((m) => m.index), staves: 1 }],
});

describe("repeat notation engraving", () => {
  it("draws repeat barline dots, the volta label, % and segno", () => {
    const { svg } = renderNotation(
      rendered([
        measure(0, marks({ forward: true, segno: true })),
        measure(1, marks({ backward_times: 2, ending_start: [1], ending_stop: true })),
        measure(2, marks({ measure_repeat_of: 1, measure_repeat_slashes: 1 })),
      ]),
      900,
    );
    expect(svg).toContain("<circle"); // repeat dots
    expect(svg).toContain("1."); // volta label
    expect(svg).toContain("\u{E500}"); // % (repeat1Bar)
    expect(svg).toContain("\u{E047}"); // segno
  });

  it("is unchanged for measures without repeat marks", () => {
    const { svg } = renderNotation(rendered([measure(0), measure(1)]), 900);
    expect(svg).not.toContain("<circle");
    expect(svg).not.toContain("\u{E500}");
  });
});
