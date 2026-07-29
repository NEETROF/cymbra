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

describe("notation painter", () => {
  it("draws staves, a clef, note heads, a stem and a beam", () => {
    const svg = renderNotation(makeGeometry(), 1000);
    expect(svg.startsWith("<svg")).toBe(true);
    // Five staff lines for the single staff.
    expect((svg.match(/class="staff"/g) ?? []).length).toBe(5);
    expect(svg).toContain(gClef); // treble clef glyph
    expect(svg).toContain(noteheadBlack); // quarter + eighth heads
    expect(svg).toContain('class="stem"'); // the quarter note's stem
    expect(svg).toContain('class="beam"'); // the beamed eighth pair
    expect(svg).toContain("font-family:'Bravura'");
  });

  it("renders read-only — no interactive/edit affordances", () => {
    const svg = renderNotation(makeGeometry(), 1000);
    expect(svg).not.toContain("<button");
    expect(svg).not.toContain("<input");
    expect(svg).not.toContain("onclick");
    expect(svg).not.toContain("contenteditable");
  });
});

describe("useScoreRenderer", () => {
  afterEach(() => setNotationWasmForTest(null));

  it("does not render until bytes arrive, then renders once", async () => {
    const render = vi.fn(() => makeGeometry());
    setNotationWasmForTest({ render });
    const bytes = ref<Uint8Array | null>(null);
    const scope = effectScope();
    const api = scope.run(() => useScoreRenderer(bytes))!;

    await flushPromises();
    expect(api.notation.value.status).toBe("idle");
    expect(render).not.toHaveBeenCalled();

    bytes.value = new Uint8Array([1, 2, 3]);
    await flushPromises();
    expect(render).toHaveBeenCalledTimes(1);
    expect(api.notation.value.status).toBe("success");
    if (api.notation.value.status === "success") {
      expect(api.notation.value.data).toContain("<svg");
    }
    scope.stop();
  });

  it("instantiates the wasm module once and reuses it", async () => {
    const stub = { render: vi.fn(() => makeGeometry()) };
    setNotationWasmForTest(stub);
    const a = await loadNotationWasm();
    const b = await loadNotationWasm();
    expect(a).toBe(b); // one cached instance
  });

  it("degrades to an error state when rendering fails (no throw)", async () => {
    setNotationWasmForTest({
      render: () => {
        throw new Error("unparseable");
      },
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

  it("injects the rendered SVG on success", () => {
    const svg = renderNotation(makeGeometry(), 1000);
    const w = mount(ScorePreview, {
      global: withI18n,
      props: { hit: hit as never, bytes: new Uint8Array([1]), loading: false, notation: success(svg) },
    });
    expect(w.find(".svg-wrap svg").exists()).toBe(true);
    // Presentational + read-only: no edit controls in the preview.
    expect(w.findAll("button")).toHaveLength(0);
    expect(w.findAll("input")).toHaveLength(0);
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
