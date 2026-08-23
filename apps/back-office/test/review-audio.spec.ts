import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { createMemoryHistory, createRouter } from "vue-router";
import ReviewView from "@/views/ReviewView.vue";
import { i18n } from "@/i18n";
import { setClientsForTest } from "@/lib/api";
import { setNotationWasmForTest } from "@/lib/notation/wasm";
import { setAudioWasmForTest } from "@/lib/audio/synth";
import { setSoundFontForTest } from "@/lib/audio/soundfont";
import { useAuthStore } from "@/stores/auth";
import { makeFakeClients, makeJwt } from "./fakes";
import type { PlaybackSchedule } from "@/lib/notation/schedule";

// A fake Web Audio stack that records which PCM actually reached the speakers, and
// whether that node was ever stopped. An orphaned (never-stopped) node is exactly what
// the moderator hears as "the previous score playing over the new one".

interface FakeBuffer {
  marker: number;
  duration: number;
  copyToChannel: (data: Float32Array, channel: number) => void;
}
interface StartedNode {
  marker: number;
  stopped: boolean;
}
const started: StartedNode[] = [];

class FakeSource {
  buffer: FakeBuffer | null = null;
  onended: unknown = null;
  private rec: StartedNode | null = null;
  connect() {}
  disconnect() {}
  start() {
    this.rec = { marker: this.buffer?.marker ?? -1, stopped: false };
    started.push(this.rec);
  }
  stop() {
    if (this.rec) this.rec.stopped = true;
  }
}

class FakeCtx {
  currentTime = 0;
  sampleRate = 44100;
  destination = {};
  async resume() {}
  async close() {}
  createBufferSource() {
    return new FakeSource();
  }
  createBuffer(_channels: number, length: number, rate: number): FakeBuffer {
    const buf: FakeBuffer = {
      marker: -1,
      duration: length / rate,
      copyToChannel: (data: Float32Array) => {
        if (buf.marker < 0) buf.marker = data[0];
      },
    };
    return buf;
  }
}

/** Markers identifying each score's rendered PCM. */
const A = "a".charCodeAt(0);
const B = "b".charCodeAt(0);
/** First byte of the kit font's bytes, distinct from the default piano's `1`. */
const KIT = 75;
const liveMarkers = () => started.filter((s) => !s.stopped).map((s) => s.marker);
/** First byte of the SoundFont each render was fed — which font actually played. */
const renderedWith: number[] = [];

const schedule: PlaybackSchedule = {
  notes: [{ midi: 60, onset_ms: 0, duration_ms: 500, staff: 1, measure_index: 0, note_index: 0 }],
  measure_start_ms: [0],
  song_end_ms: 1000,
  bpm: 120,
};

const hit = (id: string) => ({
  id,
  title: id.toUpperCase(),
  composer: "",
  arranger: "",
  level: "",
  source: "pdmx",
  license: "CC0",
  moderationStatus: "pending",
});

/** Distinct bytes per score, so the rendered PCM carries an identifiable marker. */
const scoreBytes = (id: string) => new Uint8Array([id.charCodeAt(0)]);

/** Mounted views, unmounted after each test — ReviewView listens on `window` for the
 *  decision shortcuts, and a leaked listener would decide another test's score. */
const mounted: { unmount: () => void }[] = [];

function mountReview(opts: { audioDelay?: () => Promise<void>; percussion?: string[]; kit?: boolean } = {}) {
  setActivePinia(createPinia());
  const token = makeJwt({ roles: ["moderator"], sub: "u1" });
  const { clients, state } = makeFakeClients({ tokens: { accessToken: token, refreshToken: "r" } });
  // A row's recorded instrument (change: add-drums-access): ids listed in
  // `opts.percussion` are drum scores; since add-drum-audio-channel they audition
  // through a percussion-family font instead of being gated out.
  const hitOf = (id: string) => ({
    ...hit(id),
    ...(opts.percussion?.includes(id) ? { instrument: "percussion" } : {}),
  });
  // `kit: true` seeds the public catalog with an accepted kit font, served over a
  // stubbed fetch with the KIT marker byte (the delivery route, jsdom has no
  // Cache API so the plain-fetch path answers).
  if (opts.kit) {
    Object.assign(clients.score as unknown as Record<string, unknown>, {
      listSoundFonts: async () => ({
        soundfonts: [
          { id: "upright-piano-kw", label: "Upright", instrument: "keyboard", license: "", attribution: "" },
          { id: "rock-kit", label: "Rock Kit", instrument: "percussion", license: "", attribution: "" },
        ],
      }),
    });
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.resolve(new Response(new Uint8Array([KIT, 0, 0]), { status: 200 }))),
    );
  }
  const rows = new Map([
    ["a", hitOf("a")],
    ["b", hitOf("b")],
  ]);
  let page = 0;
  Object.assign(clients.score as unknown as Record<string, unknown>, {
    searchCatalog: async () => {
      page += 1;
      return page === 1
        ? { hits: [rows.get("a"), rows.get("b")], total: 2, nextOffset: 2 }
        : { hits: [], total: 0, nextOffset: 0 };
    },
    getCatalogScoreBytes: async ({ catalogId }: { catalogId: string }) => ({ data: scoreBytes(catalogId) }),
    getCatalogScore: async ({ catalogId }: { catalogId: string }) => {
      state.getCatalogScoreCalls += 1;
      return rows.get(catalogId);
    },
    updateCatalogScore: async (req: { scoreId: string; title?: string }) => {
      state.editCalls.push(req);
      rows.set(req.scoreId, { ...hit(req.scoreId), ...req, id: req.scoreId });
      return {};
    },
    setModerationStatus: async (req: { scoreId: string; status: string }) => {
      state.evaluateCalls.push(req);
      return {};
    },
  });
  setClientsForTest(clients);
  useAuthStore().setToken(token);

  setNotationWasmForTest({
    render: async () => ({
      kind: "notation",
      svg: "<svg></svg>",
      layout: { width: 10, height: 10, measures: [] },
      percussion: false,
    }),
    schedule: async () => schedule,
  });
  setAudioWasmForTest({
    render: async (bytes: Uint8Array, sf2: Uint8Array) => {
      if (opts.audioDelay) await opts.audioDelay();
      renderedWith.push(sf2[0]); // which font the synth was actually fed
      const marker = bytes[0];
      return { left: new Float32Array([marker]), right: new Float32Array([marker]), frames: 1 };
    },
  });
  setSoundFontForTest(new Uint8Array([1, 2, 3]));

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: "/", name: "music-queue", component: { template: "<div/>" } }],
  });
  const w = mount(ReviewView, { global: { plugins: [i18n, router] } });
  mounted.push(w);
  return { w, state };
}

beforeEach(() => {
  started.length = 0;
  renderedWith.length = 0;
  vi.stubGlobal("AudioContext", FakeCtx);
  vi.stubGlobal("requestAnimationFrame", () => 1);
  vi.stubGlobal("cancelAnimationFrame", () => {});
});
afterEach(() => {
  mounted.splice(0).forEach((w) => w.unmount());
  document.body.innerHTML = ""; // the drawer teleports here
  setNotationWasmForTest(null);
  setAudioWasmForTest(null);
  setSoundFontForTest(null);
  setClientsForTest(null as never);
  vi.unstubAllGlobals();
});

describe("review mode playback", () => {
  it("stops the decided score and plays only the next one", async () => {
    const { w } = mountReview();
    await flushPromises();
    // The transport sits in the decision row, not inside the preview.
    expect(w.find(".actions button.play").exists()).toBe(true);
    expect(w.find(".preview .transport").exists()).toBe(false);
    expect(started.map((s) => s.marker)).toEqual([A]); // "a" auto-played

    await w.get("button.accept").trigger("click");
    await flushPromises();
    await flushPromises();

    expect(liveMarkers()).toEqual([B]);
  });

  it("drops an in-flight render when the moderator advances mid-synthesis", async () => {
    const releases: (() => void)[] = [];
    const { w } = mountReview({ audioDelay: () => new Promise<void>((r) => releases.push(r)) });
    await flushPromises();
    expect(started).toHaveLength(0); // "a" is still being synthesised

    await w.get("button.accept").trigger("click");
    await flushPromises();
    releases.forEach((release) => release());
    await flushPromises();
    await flushPromises();

    expect(liveMarkers()).not.toContain(A);
  });

  // The reported bug: pressing Play while the first (slow) load is still running used
  // to start a SECOND source for the same score. Only the last one was tracked, so
  // advancing stopped that one and the orphan kept playing over the next score.
  it("never leaves an orphaned source when Play is pressed during the load", async () => {
    const releases: (() => void)[] = [];
    const { w } = mountReview({ audioDelay: () => new Promise<void>((r) => releases.push(r)) });
    await flushPromises();
    await w.get("button.play").trigger("click"); // autoplay is still rendering
    await flushPromises();
    releases.forEach((release) => release());
    await flushPromises();
    await flushPromises();
    expect(started).toHaveLength(1); // one source for "a", not two

    await w.get("button.accept").trigger("click");
    await flushPromises();
    releases.forEach((release) => release());
    await flushPromises();
    await flushPromises();

    expect(liveMarkers()).not.toContain(A);
  });

  // Play lift (change: add-drum-audio-channel): a percussion row auditions through
  // the wasm renderer — which resolves the drum channel from the document itself —
  // with the picked kit font, never through a keyboard font.
  it("auditions a percussion score with the kit font once its bytes are in hand", async () => {
    const { w } = mountReview({ percussion: ["a"], kit: true });
    // The chain is longer than a keyboard row's: catalog listing → kit selection →
    // kit bytes → schedule → autoplay. Drain until it settles.
    for (let i = 0; i < 6; i++) await flushPromises();

    // The drum score auto-played like any other row…
    expect(liveMarkers()).toEqual([A]);
    // …rendered with the KIT font's bytes — never the default piano's.
    expect(renderedWith).toEqual([KIT]);
    expect(w.get(".actions button.play").attributes("disabled")).toBeUndefined();
    // No refusal note, no kit-missing note, no error.
    expect(w.find('[data-testid="no-drum-kit"]').exists()).toBe(false);
    expect(w.find(".error").exists()).toBe(false);
    // The instrument badge still identifies the drum proposal up front.
    expect(w.get("h2.score-title").text()).toContain("Drums");
  });

  it("shows the localised no-kit state when the catalog holds no accepted kit", async () => {
    const { w } = mountReview({ percussion: ["a"] }); // public catalog stays empty
    await flushPromises();
    await flushPromises();

    // No source ever started — and in particular never through a piano font.
    expect(started).toHaveLength(0);
    // The Play control is a quiet disabled/explained affordance, not an error.
    expect(w.get(".actions button.play").attributes("disabled")).toBeDefined();
    expect(w.get('[data-testid="no-drum-kit"]').text()).toContain("No drum kit available");
    expect(w.find(".error").exists()).toBe(false);

    // The spacebar shortcut stays inert too.
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true }));
    await flushPromises();
    expect(started).toHaveLength(0);

    // Deciding it advances to the keyboard score, which auditions as before.
    await w.get("button.accept").trigger("click");
    await flushPromises();
    await flushPromises();
    expect(liveMarkers()).toEqual([B]);
    expect(w.find('[data-testid="no-drum-kit"]').exists()).toBe(false);
  });
});

describe("review mode editing", () => {
  it("saves a curatorial edit for the current score and reloads the row", async () => {
    const { w, state } = mountReview();
    await flushPromises();

    // The metadata is read-only on the page; editing goes through the drawer.
    expect(document.querySelector("dialog.drawer")).toBeNull();
    await w.get("button.edit").trigger("click");
    const title = document.querySelector("dialog.drawer input") as HTMLInputElement;
    title.value = "Gymnopédie no 1";
    title.dispatchEvent(new Event("input"));
    await flushPromises();
    document.querySelector("form.edit")!.dispatchEvent(new Event("submit"));
    await flushPromises();

    expect(state.editCalls).toEqual([
      { scoreId: "a", title: "Gymnopédie no 1", composer: "", arranger: "", level: "" },
    ]);
    // The row is re-read from the backend (it recomputes the derived search keys) and
    // the header reflects it, without leaving the burn-down.
    expect(state.getCatalogScoreCalls).toBe(1);
    expect(w.get("h2.score-title").text()).toBe("Gymnopédie no 1");
    // …including the read-only summary the moderator reads (not just the header).
    expect(w.get(".preview dl.meta").text()).toContain("Gymnopédie no 1");
    // A successful save closes the drawer and stays on score "a" — editing doesn't
    // consume the queue.
    expect(document.querySelector("dialog.drawer")).toBeNull();
    expect(state.evaluateCalls).toHaveLength(0);
  });

  it("keeps the decision shortcuts off the open drawer", async () => {
    const { w, state } = mountReview({});
    await flushPromises();

    await w.get("button.edit").trigger("click");
    // "a" typed into the level <select> (or anywhere behind the open drawer) must
    // not accept the score.
    document
      .querySelector("form.edit select")!
      .dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
    await flushPromises();
    expect(state.evaluateCalls).toHaveLength(0);

    // Escape closes the drawer instead of leaving review mode…
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await flushPromises();
    expect(document.querySelector("dialog.drawer")).toBeNull();

    // …and the shortcuts drive the burn-down again.
    document.body.dispatchEvent(new KeyboardEvent("keydown", { key: "a", bubbles: true }));
    await flushPromises();
    expect(state.evaluateCalls).toEqual([{ scoreId: "a", status: "accepted" }]);
  });
});
