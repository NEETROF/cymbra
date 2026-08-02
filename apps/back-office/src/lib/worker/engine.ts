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
    // later action re-creates it.
    for (const p of pending.values()) p.reject(new Error("engine_worker_error"));
    pending.clear();
    worker = null;
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

/** Whole-score PCM (deinterleaved planar channels, transferred zero-copy). */
export function engineRenderAudio(bytes: Uint8Array, sf2: Uint8Array, sampleRate: number): Promise<AudioPcm> {
  return call<AudioPcm>({ kind: "audio", bytes, sf2, sampleRate });
}
