import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { renderNotation } from "@/lib/notation/painter";
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
const noteheadBlack = "\u{E0A4}";

const wasmStub = (geo = makeGeometry()) => ({
  render: vi.fn(() => geo),
  schedule: vi.fn(() => ({ notes: [], measure_start_ms: [0], song_end_ms: 0, bpm: 90 })),
});

describe("notation painter", () => {
  it("draws staves, a clef, note heads, a stem and a beam", () => {
    const { svg, layout } = renderNotation(makeGeometry(), 1000);
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
    const { svg } = renderNotation(makeGeometry(), 1000);
    // Three pitched notes in measure 0 → indices 0,1,2.
    expect(svg).toContain('data-note="0:0"');
    expect(svg).toContain('data-note="0:1"');
    expect(svg).toContain('data-note="0:2"');
  });

  it("renders read-only — no interactive/edit affordances in the SVG", () => {
    const { svg } = renderNotation(makeGeometry(), 1000);
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
    const { svg } = renderNotation(geo, 1000);
    // System 1 shows 4 flats; system 2's header shows the change: the three
    // removed flats cancelled by naturals, then the remaining 1 flat.
    expect(svg.split(flat).length - 1).toBeGreaterThanOrEqual(5);
    expect(svg).toContain(natural);
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
    if (api.notation.value.status === "success") {
      expect(api.notation.value.data.svg).toContain("<svg");
      expect(api.notation.value.data.layout.measures.length).toBeGreaterThan(0);
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
      render: () => {
        throw new Error("unparseable");
      },
      schedule: () => ({ notes: [], measure_start_ms: [], song_end_ms: 0, bpm: 90 }),
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
    const result = renderNotation(makeGeometry(), 1000);
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
    const result = renderNotation(makeGeometry(), 1000);
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
    const result = renderNotation(makeGeometry(), 1000);
    const base = { hit: hit as never, bytes: new Uint8Array([1]), loading: false, notation: success(result) };
    const off = mount(ScorePreview, { global: withI18n, props: { ...base, canPlay: false } });
    expect(off.get("button.play").attributes("disabled")).toBeDefined();
    const on = mount(ScorePreview, { global: withI18n, props: { ...base, canPlay: true, playing: true } });
    expect(on.get("button.play").text()).toContain("Pause");
  });

  it("shows a loading spinner while audio (the SoundFont) downloads", () => {
    const result = renderNotation(makeGeometry(), 1000);
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
    const result = renderNotation(makeGeometry(), 1000);
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
});
