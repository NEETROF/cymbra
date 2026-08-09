/// <reference lib="webworker" />
// Off-main-thread score engine. The heavy WebAssembly work — notation layout + paint,
// the playback schedule, and the whole-score audio synthesis — runs here so a big
// score or a long render never freezes the moderation console's UI thread (which was
// triggering Chrome's "page unresponsive" dialog on Play).
//
// One worker hosts both wasm modules, each lazy-loaded on first use (mirroring the
// main-thread seams). Results are posted back by request id; the audio PCM is
// deinterleaved into planar channels here and transferred (zero-copy) so the main
// thread only has to copy them into an AudioBuffer.

import { renderNotation, type RenderResult } from "@/lib/notation/painter";
import type { RenderedScore } from "@/lib/notation/geometry";
import type { PlaybackSchedule } from "@/lib/notation/schedule";

// An `audio` request names its SoundFont by `fontKey` and carries `sf2` only when the
// main thread believes this worker doesn't hold that font yet — so the common case
// posts a score, not tens of MB of samples (change: cache-soundfont-delivery).
type Req =
  | { id: number; kind: "schedule"; bytes: Uint8Array }
  | { id: number; kind: "notation"; bytes: Uint8Array; width: number }
  | {
      id: number;
      kind: "audio";
      bytes: Uint8Array;
      fontKey: string;
      sf2?: Uint8Array;
      sampleRate: number;
    };

interface NotationApi {
  render(bytes: Uint8Array, width: number): RenderedScore;
  schedule(bytes: Uint8Array): PlaybackSchedule;
}
interface AudioApi {
  hasFont(key: string): boolean;
  /** Parse + cache a font under `key` (no-op if already cached). Throws on a bad `.sf2`. */
  loadFont(key: string, sf2: Uint8Array): void;
  /** Render with the font cached under `key`; throws `font_not_loaded` on a miss. */
  render(bytes: Uint8Array, key: string, sampleRate: number): Float32Array;
}

// Lazy wasm loaders — the ~300KB notation module and the audio module are only
// instantiated when a moderator first needs them, and only once each.
let notationMod: Promise<NotationApi> | null = null;
function notation(): Promise<NotationApi> {
  notationMod ??= (async () => {
    const mod = await import("@/wasm/pkg/musicxml_wasm.js");
    await mod.default();
    return {
      render: (bytes, width) => mod.render(bytes, width) as RenderedScore,
      schedule: (bytes) => mod.schedule(bytes) as PlaybackSchedule,
    };
  })();
  return notationMod;
}

let audioMod: Promise<AudioApi> | null = null;
function audio(): Promise<AudioApi> {
  audioMod ??= (async () => {
    const mod = await import("@/wasm/pkg-audio/audio_wasm.js");
    await mod.default();
    return {
      hasFont: (key) => mod.has_soundfont(key) as boolean,
      loadFont: (key, sf2) => mod.load_soundfont(key, sf2),
      render: (bytes, key, sr) => mod.render_cached(bytes, key, sr) as Float32Array,
    };
  })();
  return audioMod;
}

const ctx = self as unknown as {
  postMessage(message: unknown, transfer?: Transferable[]): void;
  onmessage: ((e: MessageEvent<Req>) => void) | null;
};

async function handle(req: Req): Promise<void> {
  try {
    if (req.kind === "schedule") {
      const result = (await notation()).schedule(req.bytes);
      ctx.postMessage({ id: req.id, ok: true, result });
    } else if (req.kind === "notation") {
      const geometry = (await notation()).render(req.bytes, req.width);
      const result: RenderResult = renderNotation(geometry, req.width);
      ctx.postMessage({ id: req.id, ok: true, result });
    } else {
      const api = await audio();
      // Bytes present → (re)load the font; absent → it must already be cached here. A
      // miss is reported so the main thread can retry once *with* the bytes rather than
      // surfacing an audio error (the one-slot wasm cache evicts on a font switch).
      if (req.sf2) api.loadFont(req.fontKey, req.sf2);
      else if (!api.hasFont(req.fontKey)) throw new Error("font_not_loaded");
      const pcm = api.render(req.bytes, req.fontKey, req.sampleRate); // interleaved L,R
      const frames = Math.floor(pcm.length / 2);
      const left = new Float32Array(frames);
      const right = new Float32Array(frames);
      for (let i = 0; i < frames; i++) {
        left[i] = pcm[i * 2];
        right[i] = pcm[i * 2 + 1];
      }
      ctx.postMessage({ id: req.id, ok: true, result: { left, right, frames } }, [left.buffer, right.buffer]);
    }
  } catch (err) {
    // Post a stable-ish string; the main-thread composables fold it into their
    // Async error state (never shown raw to the user).
    ctx.postMessage({ id: req.id, ok: false, error: String(err) });
  }
}

// `handle` catches its own errors and reports them back to the caller, so the
// floating promise is intentional (the void-returning onmessage slot).
ctx.onmessage = (e: MessageEvent<Req>) => {
  void handle(e.data);
};
