import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { isPercussionScore, renderNotation, type NotationRender } from "@/lib/notation/painter";
import type { NoteEvent, RenderedScore } from "@/lib/notation/geometry";
import { useScoreRenderer } from "@/composables/useScoreRenderer";
import { loadNotationWasm, setNotationWasmForTest } from "@/lib/notation/wasm";
import ScorePreview from "@/components/ScorePreview.vue";
import { failure, loading, success } from "@/lib/async";
import { i18n } from "@/i18n";

// --- Fixture geometry (what the wasm renderer returns) -----------------------

function note(over: Partial<NoteEvent>): NoteEvent {
  return {
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
    stem: "Up",
    beams: [],
    lyric: null,
    ...over,
  };
}

// One treble measure: a quarter C5, then a beamed eighth pair (D5, E5).
function makeGeometry(): RenderedScore {
  return {
    document: {
      meta: { title: "Fixture", composer: null },
      staves: 1,
      attributes: {
        divisions: 4,
        clefs: [{ staff: 1, sign: "G", line: 2 }],
        key_fifths: 0,
        time: { beats: 4, beat_type: 4 },
      },
      measures: [
        {
          index: 0,
          min_width: 100,
          clefs: [],
          key_fifths: 0,
          notes: [
            note({ position_divisions: 0 }),
            note({
              position_divisions: 4,
              pitch: { step: "D", octave: 5, alter: 0 },
              note_type: "eighth",
              duration_divisions: 2,
              beams: ["Begin"],
            }),
            note({
              position_divisions: 6,
              pitch: { step: "E", octave: 5, alter: 0 },
              note_type: "eighth",
              duration_divisions: 2,
              beams: ["End"],
            }),
          ],
        },
      ],
    },
    systems: [{ measures: [0], staves: 1 }],
  };
}

const gClef = "\u{E050}";
const percussionClef = "\u{E069}";
const noteheadBlack = "\u{E0A4}";
const noteheadXBlack = "\u{E0A9}";

/** Paint a fixture. Every document draws now — the percussion carve-out is retired
 *  (change: add-drum-notation-render); percussion routes to its own paint path. */
function paint(geo: RenderedScore, width = 1000): NotationRender {
  return renderNotation(geo, width);
}

// The seam now paints in the worker, so `render` resolves a finished RenderResult
// (not raw geometry). The stub mirrors that: lay the fixture out on the spot.
const wasmStub = (geo = makeGeometry()) => ({
  render: vi.fn(async () => renderNotation(geo, 1000)),
  schedule: vi.fn(async () => ({ notes: [], measure_start_ms: [0], song_end_ms: 0, bpm: 90 })),
});

describe("notation painter", () => {
  it("draws staves, a clef, note heads, a stem and a beam", () => {
    const { svg, layout } = paint(makeGeometry());
    expect(svg.startsWith("<svg")).toBe(true);
    // Five staff lines for the single staff.
    expect((svg.match(/class="staff"/g) ?? []).length).toBe(5);
    expect(svg).toContain(gClef); // treble clef glyph
    expect(svg).toContain(noteheadBlack); // quarter + eighth heads
    expect(svg).toContain('class="stem"'); // the quarter note's stem
    expect(svg).toContain('class="beam"'); // the beamed eighth pair
    expect(svg).toContain("font-family:'Bravura'");
    // Layout map exposes the one measure for the playhead.
    expect(layout.measures).toHaveLength(1);
    expect(layout.measures[0].index).toBe(0);
  });

  it("tags pitched note heads with data-note for the playhead", () => {
    const { svg } = paint(makeGeometry());
    // Three pitched notes in measure 0 → indices 0,1,2.
    expect(svg).toContain('data-note="0:0"');
    expect(svg).toContain('data-note="0:1"');
    expect(svg).toContain('data-note="0:2"');
  });

  it("renders read-only — no interactive/edit affordances in the SVG", () => {
    const { svg } = paint(makeGeometry());
    expect(svg).not.toContain("<button");
    expect(svg).not.toContain("<input");
    expect(svg).not.toContain("onclick");
    expect(svg).not.toContain("contenteditable");
  });

  it("draws the armure per system and a key change at a modulation", () => {
    const flat = "\u{E260}";
    const natural = "\u{E261}";
    const measure = (index: number, keyFifths: number) => ({
      index,
      min_width: 100,
      clefs: [],
      key_fifths: keyFifths,
      notes: [note({ position_divisions: 0, pitch: { step: "C", octave: 5, alter: 0 } })],
    });
    // Modulates 4 flats → 1 flat, one measure per system (like Haydn's canzonet).
    const geo = {
      document: {
        meta: { title: "Mod", composer: null },
        staves: 1,
        attributes: {
          divisions: 4,
          clefs: [{ staff: 1, sign: "G", line: 2 }],
          key_fifths: -1,
          time: { beats: 4, beat_type: 4 },
        },
        measures: [measure(0, -4), measure(1, -1)],
      },
      systems: [
        { measures: [0], staves: 1 },
        { measures: [1], staves: 1 },
      ],
    };
    const { svg } = paint(geo);
    // System 1 shows 4 flats; system 2's header shows the change: the three
    // removed flats cancelled by naturals, then the remaining 1 flat.
    expect(svg.split(flat).length - 1).toBeGreaterThanOrEqual(5);
    expect(svg).toContain(natural);
  });

  // --- percussion routing (change: add-drum-notation-render) ------------------

  /** A drum fixture: percussion clef + one unpitched note (snare-position C5). */
  function percussionGeometry(over: { clefSign?: string; unpitched?: boolean } = {}): RenderedScore {
    const geo = makeGeometry();
    geo.document.attributes.clefs = [{ staff: 1, sign: over.clefSign ?? "percussion", line: 2 }];
    if (over.unpitched !== false) {
      geo.document.measures[0].notes = [
        note({
          pitch: null,
          unpitched: { display_step: "C", display_octave: 5, gm_number: 38, head_class: "Oval" },
        }),
      ];
    }
    return geo;
  }

  it("detects percussion from the clef sign or the unpitched channel", () => {
    expect(isPercussionScore(percussionGeometry().document)).toBe(true);
    // Clef alone (no unpitched notes) is enough…
    expect(isPercussionScore(percussionGeometry({ unpitched: false }).document)).toBe(true);
    // …and unpitched notes alone (a drum part exported without its clef) too.
    expect(isPercussionScore(percussionGeometry({ clefSign: "G" }).document)).toBe(true);
    // A mid-piece percussion clef change also marks the score.
    const midPiece = makeGeometry();
    midPiece.document.measures[0].clefs = [{ staff: 1, sign: "percussion", line: 2 }];
    expect(isPercussionScore(midPiece.document)).toBe(true);
    // The keyboard fixture stays keyboard.
    expect(isPercussionScore(makeGeometry().document)).toBe(false);
  });

  it("routes a percussion document to the percussion paint path (percussion clef drawn)", () => {
    const result = renderNotation(percussionGeometry(), 1000);
    expect(result.kind).toBe("notation");
    expect(result.percussion).toBe(true);
    // The clef glyph specifically — the default-to-treble fallback would draw a
    // G clef and a bare "a clef is drawn" assertion would not catch it.
    expect(result.svg).toContain(percussionClef);
    expect(result.svg).not.toContain(gClef);
  });

  it("routes a clef-less drum export (unpitched notes only) to the percussion path", () => {
    // The document declares a G clef, but the unpitched channel marks it as drums:
    // the percussion clef is drawn, not a treble staff's.
    const result = renderNotation(percussionGeometry({ clefSign: "G" }), 1000);
    expect(result.percussion).toBe(true);
    expect(result.svg).toContain(percussionClef);
    expect(result.svg).not.toContain(gClef);
  });

  it("stems a bare percussion note by voice — 1 up, 2 down — and follows an explicit <stem>", () => {
    const geo = percussionGeometry();
    const unp = { display_step: "C", display_octave: 5, gm_number: 38, head_class: "Oval" as const };
    geo.document.measures[0].notes = [
      // Voice 1, no <stem> → up; voice 2, no <stem> → down; voice 2 with an
      // explicit Up → the file wins.
      note({ pitch: null, unpitched: unp, stem: null, voice: 1, position_divisions: 0 }),
      note({ pitch: null, unpitched: unp, stem: null, voice: 2, position_divisions: 4 }),
      note({
        pitch: null,
        unpitched: { ...unp, display_step: "F", display_octave: 4 },
        stem: "Up",
        voice: 2,
        position_divisions: 8,
      }),
    ];
    const { svg } = paint(geo);
    const stems = [...svg.matchAll(/<line x1="[^"]*" y1="([-\d.]+)" x2="[^"]*" y2="([-\d.]+)" class="stem"\/>/g)].map(
      (m) => ({ y1: Number(m[1]), y2: Number(m[2]) }),
    );
    expect(stems).toHaveLength(3);
    expect(stems[0].y2).toBeLessThan(stems[0].y1); // voice 1 → up
    expect(stems[1].y2).toBeGreaterThan(stems[1].y1); // voice 2 → down
    expect(stems[2].y2).toBeLessThan(stems[2].y1); // explicit Up wins over voice 2
  });

  it("offsets the second voice's head on a same-position shared onset (neither hides)", () => {
    const geo = percussionGeometry();
    const unp = { display_step: "C", display_octave: 5, gm_number: 38, head_class: "Oval" as const };
    geo.document.measures[0].notes = [
      note({ pitch: null, unpitched: unp, voice: 1, position_divisions: 0 }),
      note({ pitch: null, unpitched: unp, voice: 2, position_divisions: 0, stem: "Down" }),
    ];
    const { svg } = paint(geo);
    const heads = [...svg.matchAll(/<text x="([-\d.]+)" y="[-\d.]+" class="glyph ink"[^>]*data-note="0:\d+">/g)].map(
      (m) => Number(m[1]),
    );
    // Both heads are engraved, at distinct x positions.
    expect(heads).toHaveLength(2);
    expect(heads[1]).toBeGreaterThan(heads[0]);
  });

  it("keeps rests at the midline in a single-voice percussion measure", () => {
    const geo = percussionGeometry();
    geo.document.measures[0].notes = [
      note({ pitch: null, is_rest: true, note_type: "quarter", voice: 1, position_divisions: 0 }),
    ];
    const { svg } = paint(geo);
    // First system's midline: systemGap + topPad + staffHeight − 2 staff spaces
    // = 36 + 60 + 48 − 24 = 120 (see the painter's constants).
    const rest =
      /<text x="[-\d.]+" y="([-\d.]+)" class="glyph ink" font-size="[\d.]+" text-anchor="middle">\u{E4E5}<\/text>/u.exec(
        svg,
      );
    expect(rest).not.toBeNull();
    expect(Number(rest![1])).toBe(120);
  });

  it("still draws a keyboard score exactly as before (no percussion false positive)", () => {
    const result = renderNotation(makeGeometry(), 1000);
    expect(result.kind).toBe("notation");
    expect(result.percussion).toBe(false);
    expect(result.svg).toContain(gClef);
    expect(result.svg).not.toContain(percussionClef);
    expect(result.svg).not.toContain(noteheadXBlack);
  });
});

describe("useScoreRenderer", () => {
  afterEach(() => setNotationWasmForTest(null));

  it("does not render until bytes arrive, then renders once", async () => {
    const stub = wasmStub();
    setNotationWasmForTest(stub);
    const bytes = ref<Uint8Array | null>(null);
    const scope = effectScope();
    const api = scope.run(() => useScoreRenderer(bytes))!;

    await flushPromises();
    expect(api.notation.value.status).toBe("idle");
    expect(stub.render).not.toHaveBeenCalled();

    bytes.value = new Uint8Array([1, 2, 3]);
    await flushPromises();
    expect(stub.render).toHaveBeenCalledTimes(1);
    expect(api.notation.value.status).toBe("success");
    if (api.notation.value.status === "success" && api.notation.value.data.kind === "notation") {
      expect(api.notation.value.data.svg).toContain("<svg");
      expect(api.notation.value.data.layout.measures.length).toBeGreaterThan(0);
    } else {
      throw new Error("expected a drawn notation");
    }
    scope.stop();
  });

  it("instantiates the wasm module once and reuses it", async () => {
    setNotationWasmForTest(wasmStub());
    const a = await loadNotationWasm();
    const b = await loadNotationWasm();
    expect(a).toBe(b); // one cached instance
  });

  it("degrades to an error state when rendering fails (no throw)", async () => {
    setNotationWasmForTest({
      render: async () => {
        throw new Error("unparseable");
      },
      schedule: async () => ({ notes: [], measure_start_ms: [], song_end_ms: 0, bpm: 90 }),
    });
    const bytes = ref<Uint8Array | null>(new Uint8Array([9]));
    const scope = effectScope();
    const api = scope.run(() => useScoreRenderer(bytes))!;

    await flushPromises();
    expect(api.notation.value.status).toBe("error");
    if (api.notation.value.status === "error") {
      // A stable code, never the raw wasm/exception string.
      expect(api.notation.value.error).toBe("render_failed");
    }
    scope.stop();
  });
});

describe("ScorePreview notation states", () => {
  const withI18n = { plugins: [i18n] };
  const hit = { title: "T", composer: "C", license: "CC0", source: "pdmx" };

  it("injects the rendered SVG and a Play control on success", () => {
    const result = paint(makeGeometry());
    const w = mount(ScorePreview, {
      global: withI18n,
      props: {
        hit: hit as never,
        bytes: new Uint8Array([1]),
        loading: false,
        notation: success(result),
        canPlay: true,
      },
    });
    expect(w.find(".svg-wrap svg").exists()).toBe(true);
    // The ONLY control is Play/Pause (read-only otherwise: no edit affordances/inputs).
    const buttons = w.findAll("button");
    expect(buttons).toHaveLength(1);
    expect(buttons[0].text()).toContain("Play");
    expect(w.findAll("input")).toHaveLength(0);
  });

  it("emits toggle when Play is clicked", async () => {
    const result = paint(makeGeometry());
    const w = mount(ScorePreview, {
      global: withI18n,
      props: {
        hit: hit as never,
        bytes: new Uint8Array([1]),
        loading: false,
        notation: success(result),
        canPlay: true,
      },
    });
    await w.get("button.play").trigger("click");
    expect(w.emitted("toggle")).toHaveLength(1);
  });

  it("disables Play until it can play, and shows Pause while playing", () => {
    const result = paint(makeGeometry());
    const base = { hit: hit as never, bytes: new Uint8Array([1]), loading: false, notation: success(result) };
    const off = mount(ScorePreview, { global: withI18n, props: { ...base, canPlay: false } });
    expect(off.get("button.play").attributes("disabled")).toBeDefined();
    const on = mount(ScorePreview, { global: withI18n, props: { ...base, canPlay: true, playing: true } });
    expect(on.get("button.play").text()).toContain("Pause");
  });

  it("shows a loading spinner while audio (the SoundFont) downloads", () => {
    const result = paint(makeGeometry());
    const w = mount(ScorePreview, {
      global: withI18n,
      props: {
        hit: hit as never,
        bytes: new Uint8Array([1]),
        loading: false,
        notation: success(result),
        canPlay: true,
        audio: loading,
      },
    });
    expect(w.find(".spinner").exists()).toBe(true);
    expect(w.text()).toContain("Loading audio…");
  });

  it("surfaces a non-fatal audio message, not a raw error", () => {
    const result = paint(makeGeometry());
    const w = mount(ScorePreview, {
      global: withI18n,
      props: {
        hit: hit as never,
        bytes: new Uint8Array([1]),
        loading: false,
        notation: success(result),
        canPlay: true,
        audio: failure("audio_failed"),
      },
    });
    expect(w.text()).toContain("Audio unavailable.");
  });

  it("shows a placeholder (not a raw error) when rendering fails", () => {
    const w = mount(ScorePreview, {
      global: withI18n,
      props: { hit: hit as never, bytes: new Uint8Array([1]), loading: false, notation: failure("render_failed") },
    });
    expect(w.find(".svg-wrap").exists()).toBe(false);
    expect(w.text()).toContain("Notation could not be rendered.");
  });

  it("shows a rendering message while the notation loads", () => {
    const w = mount(ScorePreview, {
      global: withI18n,
      props: { hit: hit as never, bytes: new Uint8Array([1]), loading: false, notation: loading },
    });
    expect(w.text()).toContain("Rendering notation…");
  });

  // --- percussion (change: add-drum-notation-render lifts the preview guard) --

  it("renders a percussion score's SVG while the Play guard shows its localised note", () => {
    const geo = makeGeometry();
    geo.document.attributes.clefs = [{ staff: 1, sign: "percussion", line: 2 }];
    geo.document.measures[0].notes = [
      {
        ...geo.document.measures[0].notes[0],
        pitch: null,
        unpitched: { display_step: "C", display_octave: 5, gm_number: 38, head_class: "Oval" },
      },
    ];
    const w = mount(ScorePreview, {
      global: withI18n,
      props: {
        hit: hit as never,
        bytes: new Uint8Array([1]),
        loading: false,
        notation: success(paint(geo)),
        // The Play guard (add-drum-audio-channel's to lift): no playable schedule…
        canPlay: false,
        // …and the view flags the percussion reason for the transport note.
        percussionGuard: true,
      },
    });
    // The notation IS drawn now — no unpreviewable panel, no failure wording.
    expect(w.find(".svg-wrap svg").exists()).toBe(true);
    expect(w.text()).not.toContain("not previewable");
    expect(w.text()).not.toContain("Notation could not be rendered.");
    // Play stays refused, as an explicit localised state distinct from an error.
    expect(w.get("button.play").attributes("disabled")).toBeDefined();
    expect(w.get('[data-testid="drums-no-audition"]').text()).toContain("not auditionable yet");
  });
});
