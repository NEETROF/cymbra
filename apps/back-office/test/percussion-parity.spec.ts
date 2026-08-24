// Percussion engraving over the REAL fixtures and the REAL wasm engine (change:
// add-drum-notation-render, tasks 2.6 + 6.2). This is the CONSOLE side of the
// app ↔ console PARITY check: the same fixture (groove_ouvert) is asserted for the
// same named facts — percussion clef glyph, written positions, head classes, open
// mark, voice rests, shared onsets — as the app's staff/partition painter tests, so
// a rule change that lands on one side only breaks a named test on the other.
//
// Unlike the geometry-level suites, the wasm module is loaded for real here
// (`initSync` over the locally built `musicxml_wasm_bg.wasm`), so these tests also
// catch a stale local wasm: a build predating the crate's `head_class` field fails
// the head-class assertions instead of silently drawing the old behaviour.

import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";
import { initSync, render } from "@/wasm/pkg/musicxml_wasm.js";
import { renderNotation, type NotationRender } from "@/lib/notation/painter";
import type { RenderedScore } from "@/lib/notation/geometry";

// --- Real inputs --------------------------------------------------------------

// Vitest runs from apps/back-office (its config root), so cwd-relative paths reach
// both the locally built wasm and the app's fixture corpus.
const wasmPath = "src/wasm/pkg/musicxml_wasm_bg.wasm";
// The app's shipping drum fixture (task 2.6 names it as the minimum corpus).
const fixturePath = "../music/assets/scores/intermediate/groove_ouvert.musicxml";

const WIDTH = 1000; // same layout width the console renderer uses

function layout(bytes: Uint8Array): { geo: RenderedScore; result: NotationRender } {
  const geo = render(bytes, WIDTH) as RenderedScore;
  return { geo, result: renderNotation(geo, WIDTH) };
}

// --- SMuFL glyphs the named facts assert --------------------------------------

const percussionClef = "\u{E069}"; // unpitchedPercussionClef1
const gClef = "\u{E050}";
const noteheadBlack = "\u{E0A4}";
const noteheadXBlack = "\u{E0A9}";
const accidentals = ["\u{E260}", "\u{E261}", "\u{E262}"]; // flat, natural, sharp
const restGlyphs = new Set(["\u{E4E3}", "\u{E4E4}", "\u{E4E5}", "\u{E4E6}", "\u{E4E7}"]);

// --- First-system staff frame, from the painter's constants -------------------
// staff space 12; bottom line y = systemGap(36) + topPad(60) + staffHeight(48).

const sp = 12;
const bottomLine = 144;
const midline = bottomLine - 2 * sp; // 120
const snareY = bottomLine - 2.5 * sp; // C5 — third space (114)
const kickY = bottomLine - 0.5 * sp; // F4 — bottom space (138)
const hiHatY = bottomLine - 4.5 * sp; // G5 — just above the top line (90)
const crashY = bottomLine - 5 * sp; // A5 — first ledger line (84)

// --- SVG extraction helpers ---------------------------------------------------

interface Head {
  x: number;
  y: number;
  measure: number;
  note: number;
  glyph: string;
}

/** Every engraved note head (they carry `data-note="measure:note"`). */
function headsOf(svg: string): Head[] {
  const re =
    /<text x="([-\d.]+)" y="([-\d.]+)" class="glyph ink" font-size="[\d.]+" data-note="(\d+):(\d+)">([^<]+)<\/text>/g;
  return [...svg.matchAll(re)].map((m) => ({
    x: Number(m[1]),
    y: Number(m[2]),
    measure: Number(m[3]),
    note: Number(m[4]),
    glyph: m[5],
  }));
}

/** Every engraved rest (centred glyphs filtered to the rest codepoints). */
function restsOf(svg: string): { y: number; glyph: string }[] {
  const re =
    /<text x="[-\d.]+" y="([-\d.]+)" class="glyph ink" font-size="[\d.]+" text-anchor="middle">([^<]+)<\/text>/g;
  return [...svg.matchAll(re)].map((m) => ({ y: Number(m[1]), glyph: m[2] })).filter((g) => restGlyphs.has(g.glyph));
}

function count(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

let fixtureXml: string;
let groove: { geo: RenderedScore; result: NotationRender };

beforeAll(() => {
  // `--target web` glue, instantiated synchronously from the local build — the
  // same module the console's worker loads (no fetch in Node).
  initSync({ module: readFileSync(wasmPath) });
  fixtureXml = readFileSync(fixturePath, "utf-8");
  groove = layout(new TextEncoder().encode(fixtureXml));
});

describe("percussion engraving parity — console painter over groove_ouvert (mirrors the app painter tests)", () => {
  it("opens every system with the percussion clef glyph — never a treble clef — and still draws the time signature", () => {
    const { geo, result } = groove;
    expect(result.percussion).toBe(true);
    // The glyph specifically, once per system: the default-to-treble fallback
    // would pass a bare "a clef is drawn" assertion while drawing a G clef.
    expect(count(result.svg, percussionClef)).toBe(geo.systems.length);
    expect(result.svg).not.toContain(gClef);
    // 4/4 engraves as on any staff (timeSig4 = U+E084, numerator + denominator).
    expect(count(result.svg, "\u{E084}")).toBe(2);
  });

  it("places the snare C5 on the third space and the kick F4 in the bottom space (y-coordinates)", () => {
    const { geo, result } = groove;
    const m0 = geo.document.measures[0].notes;
    const heads = headsOf(result.svg).filter((h) => h.measure === 0);
    const headOf = (ni: number): Head | undefined => heads.find((h) => h.note === ni);

    const snareIdx = m0.findIndex((n) => n.unpitched?.display_step === "C" && n.unpitched.display_octave === 5);
    const kickIdx = m0.findIndex((n) => n.unpitched?.display_step === "F" && n.unpitched.display_octave === 4);
    expect(snareIdx).toBeGreaterThanOrEqual(0);
    expect(kickIdx).toBeGreaterThanOrEqual(0);
    // The kick is written in voice 2 (feet) in this fixture.
    expect(m0[kickIdx].voice).toBe(2);

    expect(headOf(snareIdx)?.y).toBe(snareY);
    expect(headOf(kickIdx)?.y).toBe(kickY);
  });

  it("draws x heads on hi-hat and crash and oval heads on snare and kick, from the crate's head_class", () => {
    const { geo, result } = groove;
    const m0 = geo.document.measures[0].notes;
    // The wasm carries the crate-derived classification (a stale build without
    // `head_class` fails here rather than silently drawing ovals everywhere).
    expect(m0.some((n) => n.unpitched?.head_class === "X")).toBe(true);

    const heads = headsOf(result.svg).filter((h) => h.measure === 0);
    for (const h of heads) {
      const cls = m0[h.note].unpitched?.head_class ?? "Oval";
      if (cls === "X" || cls === "XOpen") expect(h.glyph, `note 0:${h.note}`).toBe(noteheadXBlack);
      else expect(h.glyph, `note 0:${h.note}`).toBe(noteheadBlack);
    }
    // The named positions carry the named forms: x above the staff (hi-hat G5,
    // crash A5), oval on the third space (snare) and in the bottom space (kick).
    expect(heads.some((h) => h.glyph === noteheadXBlack && h.y === hiHatY)).toBe(true);
    expect(heads.some((h) => h.glyph === noteheadXBlack && h.y === crashY)).toBe(true);
    expect(heads.some((h) => h.glyph === noteheadBlack && h.y === snareY)).toBe(true);
    expect(heads.some((h) => h.glyph === noteheadBlack && h.y === kickY)).toBe(true);
    // …and no class leaks to the other family's position.
    expect(heads.some((h) => h.glyph === noteheadBlack && (h.y === hiHatY || h.y === crashY))).toBe(false);
    expect(heads.some((h) => h.glyph === noteheadXBlack && (h.y === snareY || h.y === kickY))).toBe(false);
  });

  it("marks each open hi-hat (GM 46 / XOpen) with the open circle and leaves closed strokes unmarked", () => {
    const { geo, result } = groove;
    const openCount = geo.document.measures
      .flatMap((m) => m.notes)
      .filter((n) => n.unpitched?.head_class === "XOpen").length;
    const closedCount = geo.document.measures
      .flatMap((m) => m.notes)
      .filter((n) => n.unpitched?.head_class === "X").length;
    expect(openCount).toBeGreaterThan(0); // the fixture alternates 42 and 46
    expect(closedCount).toBeGreaterThan(0);
    // Exactly one mark per open stroke — closed x heads carry none.
    expect(count(result.svg, 'class="open-mark"')).toBe(openCount);
  });

  it("draws no armature and no accidental glyphs even when the file declares fifths = 2", () => {
    // Synthetic variant of the same fixture: an exporter-leftover key signature.
    const withKey = fixtureXml.replace("<fifths>0</fifths>", "<fifths>2</fifths>");
    expect(withKey).not.toBe(fixtureXml); // the substitution really happened
    const { result } = layout(new TextEncoder().encode(withKey));
    expect(result.percussion).toBe(true);
    for (const glyph of accidentals) expect(result.svg).not.toContain(glyph);
    expect(result.svg).toContain(percussionClef); // still the percussion staff
  });

  it("displaces the voice-2 rest below the middle line in a two-voice measure", () => {
    const { result } = groove;
    // First system only (its midline is 120; the next system sits far lower).
    const rests = restsOf(result.svg).filter((g) => g.y > bottomLine - 4 * sp && g.y < bottomLine + 2 * sp);
    expect(rests.length).toBeGreaterThan(0);
    // Voice 2's rests sit one space BELOW the midline, clear of the hi-hat line…
    expect(rests.some((g) => g.y === midline + sp)).toBe(true);
    // …and no rest of this two-voice fixture sits ON the midline.
    expect(rests.some((g) => g.y === midline)).toBe(false);
  });

  it("engraves both heads of the coinciding kick + hi-hat/crash onset at the shared column", () => {
    const { geo, result } = groove;
    const m0 = geo.document.measures[0].notes;
    const heads = headsOf(result.svg).filter((h) => h.measure === 0);
    // Beat 1 of the fixture: crash+hi-hat (voice 1) and kick (voice 2) share onset 0.
    const kickIdx = m0.findIndex((n) => n.voice === 2 && n.position_divisions === 0 && n.unpitched != null);
    const crashIdx = m0.findIndex((n) => n.voice === 1 && n.position_divisions === 0 && n.unpitched != null);
    const kick = heads.find((h) => h.note === kickIdx);
    const crash = heads.find((h) => h.note === crashIdx);
    expect(kick).toBeDefined();
    expect(crash).toBeDefined();
    // Same time column (they sit on different staff positions, so no offset), the
    // hi-hat side up top and the kick in the bottom space — neither obscured.
    expect(kick!.x).toBeCloseTo(crash!.x, 5);
    expect(crash!.y).toBeLessThan(kick!.y);
  });

  it("still engraves a note whose GM number is unresolved, with the ordinary oval head", () => {
    // Point the crash's note-level instrument reference at an undeclared id: the
    // GM number becomes unresolved. Omission-when-unresolved is a playback rule —
    // the ENGRAVING keeps the head at its written position (A5, first ledger).
    const broken = fixtureXml.replaceAll('<instrument id="P1-I49"/>', '<instrument id="P1-I99"/>');
    expect(broken).not.toBe(fixtureXml);
    const { geo, result } = layout(new TextEncoder().encode(broken));
    const m0 = geo.document.measures[0].notes;
    const crashIdx = m0.findIndex((n) => n.unpitched?.display_step === "A" && n.unpitched.display_octave === 5);
    expect(crashIdx).toBeGreaterThanOrEqual(0);
    expect(m0[crashIdx].unpitched?.gm_number == null).toBe(true);
    const head = headsOf(result.svg).find((h) => h.measure === 0 && h.note === crashIdx);
    expect(head?.glyph).toBe(noteheadBlack); // unresolved → oval, never a guess
    expect(head?.y).toBe(crashY); // …and still at its written position
  });
});
