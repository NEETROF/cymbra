// Ambient module declarations for the extension build.

// Build-time target, injected by esbuild `define` per manifest variant. Selects the
// AnalyzerPort implementation (Chromium: WASM in the content script; Firefox: WASM in
// the event page reached over messaging) and the panel surface.
declare const __TARGET__: "chromium" | "firefox";

// tokens.css (and other .css) imported as a string, inlined into the injected
// closed shadow root so the single token sheet stays the only home for colors.
declare module "*.css" {
  const css: string;
  export default css;
}

// The wasm-pack `--target web` output (produced by `yarn gen:wasm` into
// src/wasm/pkg/). Typed minimally here; the real .d.ts is generated and gitignored,
// and the content script loads the glue at runtime via chrome.runtime.getURL.
declare module "@/wasm/pkg/lingua_wasm.js" {
  export class LinguaEngine {
    constructor(packBytes: Uint8Array);
    free(): void;
    setCalibration(threshold: number): void;
    setStatus(lemma: string, status: string): void;
    analyse(blocks: string[]): string;
    gloss(lemma: string): string | undefined;
  }
  export default function init(
    moduleOrPath?: { module_or_path: string | Request | Response | URL } | string | Request | Response | URL,
  ): Promise<unknown>;
}
