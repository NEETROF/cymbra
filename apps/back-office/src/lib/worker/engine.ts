// Main-thread client for the score engine worker (engine.worker.ts). A single lazy
// worker, a request-id → promise map, and thin typed calls the seams delegate to. The
// worker is created on first real use only — unit tests inject stubs at the seam level
// (setNotationWasmForTest / setAudioWasmForTest), so no Worker is ever constructed in
// jsdom.

import type { RenderResult } from "@/lib/notation/painter";
import type { PlaybackSchedule } from "@/lib/notation/schedule";

/** Rendered PCM handed back from the worker: deinterleaved planar channels ready to
 *  drop into an AudioBuffer, plus the frame count. */
export interface AudioPcm {
  left: Float32Array;
  right: Float32Array;
  frames: number;
}

interface Reply {
  id: number;
  ok: boolean;
  result?: unknown;
  error?: string;
}
type Pending = { resolve: (value: unknown) => void; reject: (reason: unknown) => void };

let worker: Worker | null = null;
let seq = 0;
const pending = new Map<number, Pending>();

function ensureWorker(): Worker {
  if (worker) return worker;
  const w = new Worker(new URL("./engine.worker.ts", import.meta.url), { type: "module" });
  w.onmessage = (e: MessageEvent<Reply>) => {
    const { id, ok, result, error } = e.data;
    const p = pending.get(id);
    if (!p) return;
    pending.delete(id);
    if (ok) p.resolve(result);
    else p.reject(new Error(error ?? "engine_error"));
  };
  w.onerror = () => {
    // A worker-level failure (e.g. wasm instantiation) rejects every in-flight
    // request; each caller degrades to its Async error state. Drop the worker so a
    // later action re-creates it — along with what we believed it had cached.
    for (const p of pending.values()) p.reject(new Error("engine_worker_error"));
    pending.clear();
    worker = null;
    workerFonts.clear();
  };
  worker = w;
  return w;
}

function call<T>(message: Record<string, unknown>, transfer: Transferable[] = []): Promise<T> {
  const id = ++seq;
  const w = ensureWorker();
  return new Promise<T>((resolve, reject) => {
    pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
    w.postMessage({ ...message, id }, transfer);
  });
}

/** Playback schedule (measure times + timed notes) for the playhead. */
export function engineSchedule(bytes: Uint8Array): Promise<PlaybackSchedule> {
  return call<PlaybackSchedule>({ kind: "schedule", bytes });
}

/** Laid-out notation SVG + layout map (painted in the worker, ready to inject). */
export function engineRenderNotation(bytes: Uint8Array, width: number): Promise<RenderResult> {
  return call<RenderResult>({ kind: "notation", bytes, width });
}

// --- SoundFont reuse across renders -----------------------------------------
// Parsing a `.sf2` costs seconds of CPU and posting one costs a structured clone of
// tens of MB, and neither depends on the score. So each byte array gets a stable
// identity key, the worker caches the *parsed* font under it, and we only ship the
// bytes when the worker doesn't have that key (change: cache-soundfont-delivery).
// Identity, not content: two arrays holding the same font just miss (as before) — the
// loader hands out one instance per catalog id, so in practice it's one parse per font.

let fontSeq = 0;
const fontKeys = new WeakMap<Uint8Array, string>();
/** Keys we believe the live worker holds. Cleared when the worker is dropped. */
const workerFonts = new Set<string>();

function fontKeyFor(sf2: Uint8Array): string {
  let key = fontKeys.get(sf2);
  if (key === undefined) {
    key = `sf${++fontSeq}`;
    fontKeys.set(sf2, key);
  }
  return key;
}

/** Whole-score PCM (deinterleaved planar channels, transferred zero-copy). The
 *  SoundFont crosses to the worker at most once per byte array. */
export async function engineRenderAudio(bytes: Uint8Array, sf2: Uint8Array, sampleRate: number): Promise<AudioPcm> {
  const fontKey = fontKeyFor(sf2);
  if (workerFonts.has(fontKey)) {
    try {
      return await call<AudioPcm>({ kind: "audio", bytes, fontKey, sampleRate });
    } catch (e) {
      // The worker's one-slot cache dropped it (another font was rendered since):
      // fall through and re-send the bytes. Any other failure is the caller's.
      if (!String(e).includes("font_not_loaded")) throw e;
      workerFonts.delete(fontKey);
    }
  }
  const pcm = await call<AudioPcm>({ kind: "audio", bytes, fontKey, sf2, sampleRate });
  workerFonts.add(fontKey);
  return pcm;
}
