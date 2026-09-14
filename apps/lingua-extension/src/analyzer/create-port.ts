import { WasmAnalyzerPort } from "./engine.ts";
import { MessagingLinguaPort } from "./messaging-port.ts";
import type { LinguaPort } from "./port.ts";

// Pick the AnalyzerPort implementation for the build target (esbuild defines
// __ENGINE_IN_EVENT_PAGE__). Firefox's content-script CSP always blocks WASM, and Safari
// takes the same path, so both forward to the engine hosted in the event page over
// messaging. On Chromium the engine runs in the content script — except that a strict
// *page* CSP (e.g. GitHub) also blocks WASM codegen in the isolated world; see
// `resolveContentPort`. The reading and review code depend only on the LinguaPort seam,
// so nothing else changes between variants.

/**
 * The port for an **extension page** (popup, side panel): those run under the
 * extension's own CSP (`wasm-unsafe-eval`), so the in-content WASM engine always works
 * on Chromium; Firefox and Safari still forward to the event-page engine.
 */
export function createLinguaPort(): LinguaPort {
  return __ENGINE_IN_EVENT_PAGE__ ? new MessagingLinguaPort() : new WasmAnalyzerPort();
}

/**
 * The port for the **content script**, injected into an arbitrary web page. On Firefox
 * and Safari it is always the messaging port. On Chromium it prefers the in-content WASM engine,
 * but a page whose CSP forbids WASM codegen (GitHub, X, many SPAs) makes instantiation
 * throw; there we fall back to the engine hosted in the service worker — reached over
 * the same RPC as Firefox — which runs under the extension CSP, immune to the page's.
 * The probe (`calibration()`) forces instantiation once; on success the same engine is
 * reused (its build is cached), so a passing page pays no extra cost.
 */
export async function resolveContentPort(): Promise<LinguaPort> {
  if (__ENGINE_IN_EVENT_PAGE__) return new MessagingLinguaPort();
  const inContent = new WasmAnalyzerPort();
  try {
    await inContent.calibration();
    return inContent;
  } catch {
    return new MessagingLinguaPort();
  }
}
