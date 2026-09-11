import { WasmAnalyzerPort } from "./engine.ts";
import { MessagingLinguaPort } from "./messaging-port.ts";
import type { LinguaPort } from "./port.ts";

// Pick the AnalyzerPort implementation for the build target (esbuild `define`s
// __TARGET__). Firefox's content-script CSP always blocks WASM, so it forwards to the
// engine hosted in the event page over messaging. On Chromium the engine runs in the
// content script — except that a strict *page* CSP (e.g. GitHub) also blocks WASM
// codegen in the isolated world; see `resolveContentPort`. The reading and review code
// depend only on the LinguaPort seam, so nothing else changes between variants.

/**
 * The port for an **extension page** (popup, side panel): those run under the
 * extension's own CSP (`wasm-unsafe-eval`), so the in-content WASM engine always works
 * on Chromium; Firefox still forwards to the event-page engine.
 */
export function createLinguaPort(): LinguaPort {
  return __TARGET__ === "firefox" ? new MessagingLinguaPort() : new WasmAnalyzerPort();
}

/**
 * The port for the **content script**, injected into an arbitrary web page. On Firefox
 * it is always the messaging port. On Chromium it prefers the in-content WASM engine,
 * but a page whose CSP forbids WASM codegen (GitHub, X, many SPAs) makes instantiation
 * throw; there we fall back to the engine hosted in the service worker — reached over
 * the same RPC as Firefox — which runs under the extension CSP, immune to the page's.
 * The probe (`calibration()`) forces instantiation once; on success the same engine is
 * reused (its build is cached), so a passing page pays no extra cost.
 */
export async function resolveContentPort(): Promise<LinguaPort> {
  if (__TARGET__ === "firefox") return new MessagingLinguaPort();
  const inContent = new WasmAnalyzerPort();
  try {
    await inContent.calibration();
    return inContent;
  } catch {
    return new MessagingLinguaPort();
  }
}
