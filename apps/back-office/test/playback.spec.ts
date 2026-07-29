import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";
import { flushPromises } from "@vue/test-utils";
import { measureAt, playingNoteIds, type PlaybackSchedule } from "@/lib/notation/schedule";
import { useScorePlayer } from "@/composables/useScorePlayer";
import { usePlayhead } from "@/composables/usePlayhead";
import type { MeasureRect } from "@/lib/notation/painter";
import { setNotationWasmForTest } from "@/lib/notation/wasm";
import { setAudioWasmForTest } from "@/lib/audio/synth";
import { loadSoundFont, setSoundFontForTest } from "@/lib/audio/soundfont";

const schedule: PlaybackSchedule = {
  notes: [
    { midi: 60, onset_ms: 0, duration_ms: 500, staff: 1, measure_index: 0, note_index: 0 },
    { midi: 64, onset_ms: 1000, duration_ms: 500, staff: 1, measure_index: 0, note_index: 1 },
    { midi: 67, onset_ms: 2000, duration_ms: 1000, staff: 1, measure_index: 1, note_index: 0 },
  ],
  measure_start_ms: [0, 2000],
  song_end_ms: 3000,
  bpm: 120,
};

describe("playhead maths", () => {
  it("maps an elapsed time to the measure + fraction", () => {
    expect(measureAt(schedule, -1)).toBeNull(); // before the first measure
    expect(measureAt(schedule, 0)).toEqual({ index: 0, fraction: 0 });
    expect(measureAt(schedule, 1000)).toEqual({ index: 0, fraction: 0.5 });
    expect(measureAt(schedule, 2000)).toEqual({ index: 1, fraction: 0 });
    expect(measureAt(schedule, 2500)).toEqual({ index: 1, fraction: 0.5 });
    expect(measureAt(schedule, 3000)).toBeNull(); // past the end
  });

  it("reports the note ids sounding at a given time", () => {
    expect([...playingNoteIds(schedule, 100)]).toEqual(["0:0"]); // C4 ringing
    expect(playingNoteIds(schedule, 700).size).toBe(0); // between notes
    expect([...playingNoteIds(schedule, 1200)]).toEqual(["0:1"]);
    expect([...playingNoteIds(schedule, 2500)]).toEqual(["1:0"]);
  });
});

describe("useScorePlayer", () => {
  afterEach(() => {
    setNotationWasmForTest(null);
    setAudioWasmForTest(null);
    setSoundFontForTest(null);
  });

  it("derives the schedule when bytes arrive (for the playhead)", async () => {
    setNotationWasmForTest({ render: vi.fn(), schedule: () => schedule });
    const bytes = ref<Uint8Array | null>(null);
    const scope = effectScope();
    const player = scope.run(() => useScorePlayer(bytes))!;

    await flushPromises();
    expect(player.schedule.value.status).toBe("idle");
    expect(player.canPlay.value).toBe(false);

    bytes.value = new Uint8Array([1]);
    await flushPromises();
    expect(player.schedule.value.status).toBe("success");
    expect(player.canPlay.value).toBe(true);
    scope.stop();
  });

  it("degrades to audio error when no AudioContext is available (jsdom)", async () => {
    // jsdom has no AudioContext, so toggle() must fail gracefully, not throw.
    setNotationWasmForTest({ render: vi.fn(), schedule: () => schedule });
    setAudioWasmForTest({ render: () => new Float32Array([0, 0]) });
    setSoundFontForTest(new Uint8Array([1, 2, 3]));
    const bytes = ref<Uint8Array | null>(new Uint8Array([1]));
    const scope = effectScope();
    const player = scope.run(() => useScorePlayer(bytes))!;
    await flushPromises();

    player.toggle();
    await flushPromises();
    expect(player.playing.value).toBe(false);
    expect(player.audio.value.status).toBe("error");
    scope.stop();
  });
});

describe("loadSoundFont (backend delivery route)", () => {
  afterEach(() => {
    setSoundFontForTest(null);
    vi.unstubAllGlobals();
  });

  it("fetches from the delivery route with the bearer token and returns bytes", async () => {
    const fetchMock = vi.fn(() => Promise.resolve(new Response(new Uint8Array([1, 2, 3]), { status: 200 })));
    vi.stubGlobal("fetch", fetchMock);
    const bytes = await loadSoundFont("tok123");
    expect(bytes).toEqual(new Uint8Array([1, 2, 3]));
    const init = (fetchMock.mock.calls[0] as unknown[])[1] as RequestInit | undefined;
    expect(init?.headers).toMatchObject({ Authorization: "Bearer tok123" });
  });

  it("does not permanently cache a failure — a retry can succeed", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(new Response("no", { status: 401 }))
      .mockResolvedValueOnce(new Response(new Uint8Array([9]), { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    await expect(loadSoundFont("t")).rejects.toThrow();
    const bytes = await loadSoundFont("t"); // retry after (re-)auth
    expect(bytes).toEqual(new Uint8Array([9]));
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
});

describe("usePlayhead click-to-seek", () => {
  it("emits the measure index when a measure is clicked", async () => {
    const container = document.createElement("div");
    container.innerHTML = "<svg></svg>";
    document.body.appendChild(container);
    const measures: MeasureRect[] = [
      { index: 0, x: 0, width: 100, top: 0, bottom: 50 },
      { index: 1, x: 100, width: 100, top: 0, bottom: 50 },
    ];
    const seek = vi.fn();
    const scope = effectScope();
    scope.run(() =>
      usePlayhead({
        container: ref(container),
        svg: ref("<svg></svg>"),
        layout: ref({ measures }),
        schedule: ref(null),
        elapsedMs: ref(0),
        playing: ref(false),
        onSeekMeasure: seek,
      }),
    );
    await flushPromises();

    const hit = container.querySelector('[data-measure="1"]');
    expect(hit).toBeTruthy();
    hit!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(seek).toHaveBeenCalledWith(1);
    scope.stop();
    container.remove();
  });
});
