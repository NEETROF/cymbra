// The single seam to the notation engine. The heavy work (wasm layout + SVG paint,
// and the playback schedule) runs in a Web Worker (see lib/worker/) so it never blocks
// the UI thread — this function just exposes async proxies to it. Kept behind a seam
// so unit tests inject a stub via `setNotationWasmForTest` (jsdom can't spawn the
// worker or run wasm), mirroring the gRPC `api()` seam.

import type { RenderResult } from "./painter";
import type { PlaybackSchedule } from "./schedule";
import { engineRenderNotation, engineSchedule } from "@/lib/worker/engine";

export interface NotationWasm {
  /** Lay out score bytes at `width` and paint the SVG (+ layout map). Rejects on
   *  non-MusicXML input. Runs off the main thread. */
  render(bytes: Uint8Array, width: number): Promise<RenderResult>;
  /** Derive the playback schedule (timed notes + measure times + tempo). */
  schedule(bytes: Uint8Array): Promise<PlaybackSchedule>;
}

let injected: NotationWasm | null = null;

// The real implementation just forwards to the worker; there is no per-call state to
// cache on the main thread (the worker owns the wasm instance).
const workerBacked: NotationWasm = {
  render: engineRenderNotation,
  schedule: engineSchedule,
};

/** Return the notation engine (the injected stub in tests, else the worker-backed one). */
export function loadNotationWasm(): Promise<NotationWasm> {
  return Promise.resolve(injected ?? workerBacked);
}

/** Test seam: inject a stub renderer (or reset with `null`). */
export function setNotationWasmForTest(w: NotationWasm | null): void {
  injected = w;
}
