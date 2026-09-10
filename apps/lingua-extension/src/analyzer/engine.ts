import type { AnalyzerPort } from "./port.ts";
import type { LemmaStatus, PageAnalysis } from "./types.ts";

// The Chromium AnalyzerPort implementation: the lingua-core WASM module instantiated
// lazily in the content script's isolated world (design D2). This is the only place
// that touches the wasm-pack output; everything else consumes the AnalyzerPort seam.
//
// MV3 loading notes: a classic content script cannot statically import the wasm-pack
// ES module, and init()'s bare auto-fetch is unreliable under chrome-extension://, so
// we dynamic-import the glue by its extension URL and hand init() the explicit .wasm
// Response. All three files (glue, _bg.wasm, pack) are web_accessible_resources.

interface WasmModule {
  default: (init: Response | string | URL) => Promise<unknown>;
  LinguaEngine: new (packBytes: Uint8Array) => {
    setCalibration(threshold: number): void;
    setStatus(lemma: string, status: string): void;
    analyse(blocks: string[]): string;
    gloss(lemma: string): string | undefined;
    free(): void;
  };
}

/** Paths of the vendored wasm output + pack within the built extension. */
const GLUE_PATH = "wasm/lingua_wasm.js";
const WASM_PATH = "wasm/lingua_wasm_bg.wasm";
const PACK_PATH = "assets/pack.lingua";

/** The status string that clears an explicit status in the WASM engine. */
const CLEAR = "clear";

type Engine = InstanceType<WasmModule["LinguaEngine"]>;

export class WasmAnalyzerPort implements AnalyzerPort {
  private enginePromise: Promise<Engine> | null = null;

  /** Instantiated once per tab, on the first analyse() call. */
  private engine(): Promise<Engine> {
    return (this.enginePromise ??= this.build());
  }

  private async build(): Promise<Engine> {
    const mod = (await import(/* @vite-ignore */ chrome.runtime.getURL(GLUE_PATH))) as WasmModule;
    await mod.default(await fetch(chrome.runtime.getURL(WASM_PATH)));
    const packBytes = new Uint8Array(await (await fetch(chrome.runtime.getURL(PACK_PATH))).arrayBuffer());
    // Throws if the pack is malformed or built for an incompatible analyzer_version.
    return new mod.LinguaEngine(packBytes);
  }

  async analyse(blocks: string[]): Promise<PageAnalysis> {
    const engine = await this.engine();
    return JSON.parse(engine.analyse(blocks)) as PageAnalysis;
  }

  async setCalibration(threshold: number): Promise<void> {
    (await this.engine()).setCalibration(threshold);
  }

  async setStatus(lemma: string, status: LemmaStatus | null): Promise<void> {
    (await this.engine()).setStatus(lemma, status ?? CLEAR);
  }

  async gloss(lemma: string): Promise<string | undefined> {
    return (await this.engine()).gloss(lemma);
  }
}
