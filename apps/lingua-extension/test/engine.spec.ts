import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { WasmAnalyzerPort, type WasmModule } from "@/analyzer/engine.ts";

/**
 * A glue module that behaves like wasm-pack's: init() returns at once when a previous call
 * has finished, but a call made before that instantiates the module again and replaces its
 * memory — and an engine only works on the memory it was built in.
 */
function fakeGlue(failures = 0) {
  let instantiations = 0;
  let live = 0;
  let toFail = failures;
  const init = vi.fn(async () => {
    if (live !== 0) return;
    const id = ++instantiations;
    await new Promise((resolve) => setTimeout(resolve, 0));
    if (toFail > 0) {
      toFail--;
      throw new Error("instantiation failed");
    }
    live = id;
  });
  class LinguaEngine {
    private readonly memory = live;
    calibration(): number {
      if (this.memory !== live) throw new Error("memory access out of bounds");
      return 3000;
    }
  }
  const mod = { default: init, LinguaEngine } as unknown as WasmModule;
  return { load: async () => mod, init };
}

describe("WasmAnalyzerPort", () => {
  beforeEach(() => {
    vi.stubGlobal("chrome", { runtime: { getURL: (path: string) => `ext://${path}` } });
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({ arrayBuffer: async () => new ArrayBuffer(1) })),
    );
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("instantiates the module once for engines built at the same time", async () => {
    const glue = fakeGlue();
    const reading = new WasmAnalyzerPort(glue.load);
    const sync = new WasmAnalyzerPort(glue.load);

    await Promise.all([reading.for("en").calibration(), sync.for("en").calibration()]);

    expect(await reading.for("en").calibration()).toBe(3000);
    expect(await sync.for("en").calibration()).toBe(3000);
    expect(glue.init).toHaveBeenCalledTimes(1);
  });

  it("reuses the module for an engine built later", async () => {
    const glue = fakeGlue();
    const first = new WasmAnalyzerPort(glue.load);
    await first.for("en").calibration();

    await new WasmAnalyzerPort(glue.load).for("en").calibration();

    expect(glue.init).toHaveBeenCalledTimes(1);
    expect(await first.for("en").calibration()).toBe(3000);
  });

  it("retries an initialisation that failed", async () => {
    const glue = fakeGlue(1);

    await expect(new WasmAnalyzerPort(glue.load).for("en").calibration()).rejects.toThrow("instantiation failed");
    expect(await new WasmAnalyzerPort(glue.load).for("en").calibration()).toBe(3000);

    expect(glue.init).toHaveBeenCalledTimes(2);
  });
});

describe("WasmAnalyzerPort language views", () => {
  beforeEach(() => {
    vi.stubGlobal("chrome", { runtime: { getURL: (path: string) => `ext://${path}` } });
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => ({ arrayBuffer: async () => new ArrayBuffer(1) })),
    );
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  /** A glue whose engine records every call it receives, with its arguments. */
  function recordingGlue() {
    const received: Array<[string, unknown[]]> = [];
    class LinguaEngine {
      analyse(...args: unknown[]): string {
        received.push(["analyse", args]);
        return '{"analyzer_version":"1.1.0","analysable":false,"tokens":[],"counted":0,"known":0,"percent":null}';
      }
      gloss(...args: unknown[]): string | undefined {
        received.push(["gloss", args]);
        return "ville";
      }
      exportStatusOps(...args: unknown[]): string {
        received.push(["exportStatusOps", args]);
        return "[]";
      }
      languages(...args: unknown[]): string {
        received.push(["languages", args]);
        return '["en","es"]';
      }
    }
    const mod = { default: async () => {}, LinguaEngine } as unknown as WasmModule;
    return { load: async () => mod, received };
  }

  it("hands the engine the view's language, and none on a whole-reader call", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load);

    await port.for("es").analyse(["El faro"]);
    expect(await port.for("en").gloss("city")).toBe("ville");
    expect(await port.exportStatusOps()).toEqual([]);
    expect(await port.languages()).toEqual(["en", "es"]);

    expect(glue.received).toEqual([
      ["analyse", [["El faro"], "es"]],
      ["gloss", ["city", "en"]],
      ["exportStatusOps", []],
      ["languages", []],
    ]);
  });

  it("binds the view to the language it was asked", () => {
    const port = new WasmAnalyzerPort(recordingGlue().load);
    expect(port.for("es").language).toBe("es");
    expect(port.for("en").language).toBe("en");
  });
});
