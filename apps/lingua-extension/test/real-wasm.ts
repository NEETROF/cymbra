import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { vi } from "vitest";
import { WasmAnalyzerPort, type WasmModule } from "@/analyzer/engine.ts";
import * as glue from "@/wasm/pkg/lingua_wasm.js";

// The real engine under vitest (add-lingua-comeback-hints D1, D2): the wasm-pack glue `yarn gen:wasm`
// writes into src/wasm/pkg, instantiated synchronously from its .wasm file, and the committed en-fr
// testdata pack. For a spec that holds the extension's arithmetic to the engine's, not for one
// that pins a view: those run on the fake port.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

/** The committed en-fr testdata pack (`yarn gen:fixtures`). */
const PACK = readFileSync(join(root, "test/fixtures/en-fr.testdata.lingua"));

let initialised = false;

/** The glue, its module instantiated once for the whole run. */
export function realGlue(): WasmModule {
  if (!initialised) {
    glue.initSync({ module: readFileSync(join(root, "src/wasm/pkg/lingua_wasm_bg.wasm")) });
    initialised = true;
  }
  return glue as unknown as WasmModule;
}

/**
 * A port on the real engine over the en-fr testdata pack. The port's own loading — the module's
 * file and the pack, fetched from the package — is answered from disk: `chrome` and `fetch` are
 * stubbed, so the caller unstubs them (`vi.unstubAllGlobals`).
 */
export function realPort(): WasmAnalyzerPort {
  const mod = realGlue();
  vi.stubGlobal("chrome", { runtime: { getURL: (path: string) => `ext://${path}` } });
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => ({
      arrayBuffer: async () =>
        String(url).endsWith(".lingua") ? PACK.buffer.slice(PACK.byteOffset, PACK.byteOffset + PACK.byteLength) : new ArrayBuffer(0),
    })),
  );
  return new WasmAnalyzerPort(async () => mod, ["en-fr"], async () => "fr");
}
