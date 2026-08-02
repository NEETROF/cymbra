// Seam to the audio renderer. The actual rustysynth PCM synthesis runs in a Web Worker
// (see lib/worker/) so a whole-score render never freezes the UI thread; this function
// just exposes an async proxy. Injectable so unit tests stub it without a wasm
// toolchain or a worker, mirroring the notation `wasm.ts` seam.

import { engineRenderAudio, type AudioPcm } from "@/lib/worker/engine";

export type { AudioPcm };

export interface AudioWasm {
  /** Render the whole score to deinterleaved planar PCM (left/right channels),
   *  off the main thread. Rejects on non-MusicXML input or a bad SoundFont. */
  render(scoreBytes: Uint8Array, sf2Bytes: Uint8Array, sampleRate: number): Promise<AudioPcm>;
}

let injected: AudioWasm | null = null;

const workerBacked: AudioWasm = { render: engineRenderAudio };

/** Return the audio engine (the injected stub in tests, else the worker-backed one). */
export function loadAudioWasm(): Promise<AudioWasm> {
  return Promise.resolve(injected ?? workerBacked);
}

/** Test seam: inject a stub renderer (or reset with `null`). */
export function setAudioWasmForTest(w: AudioWasm | null): void {
  injected = w;
}
