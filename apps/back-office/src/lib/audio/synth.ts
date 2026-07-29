// Seam to the WebAssembly audio renderer (cymbra-audio-wasm). Lazy — the module is
// only fetched/instantiated when a moderator first hits Play — and injectable so unit
// tests stub it without a wasm toolchain, mirroring the notation `wasm.ts` seam.

export interface AudioWasm {
  /** Render the whole score to interleaved-stereo PCM (`L,R,L,R,…`). Throws on
   *  non-MusicXML input or a bad SoundFont. */
  render(scoreBytes: Uint8Array, sf2Bytes: Uint8Array, sampleRate: number): Float32Array;
}

let cached: Promise<AudioWasm> | null = null;

/** Load (once) and return the wasm audio renderer. */
export function loadAudioWasm(): Promise<AudioWasm> {
  if (cached) return cached;
  cached = (async () => {
    const mod = await import("@/wasm/pkg-audio/audio_wasm.js");
    await mod.default();
    return {
      render: (scoreBytes: Uint8Array, sf2Bytes: Uint8Array, sampleRate: number) =>
        mod.render(scoreBytes, sf2Bytes, sampleRate) as Float32Array,
    };
  })();
  return cached;
}

/** Test seam: inject a stub renderer (or reset with `null`). */
export function setAudioWasmForTest(w: AudioWasm | null): void {
  cached = w ? Promise.resolve(w) : null;
}
