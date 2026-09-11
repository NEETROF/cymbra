import { WasmAnalyzerPort } from "./engine.ts";
import { MessagingLinguaPort } from "./messaging-port.ts";
import type { LinguaPort } from "./port.ts";

// Pick the AnalyzerPort implementation for the build target (esbuild `define`s
// __TARGET__): Chromium runs the WASM engine in the content script; Firefox's CSP
// blocks that, so it forwards to the engine hosted in the event page over messaging.
// The reading and review code depend only on the LinguaPort seam, so nothing else
// changes between variants.
export function createLinguaPort(): LinguaPort {
  return __TARGET__ === "firefox" ? new MessagingLinguaPort() : new WasmAnalyzerPort();
}
