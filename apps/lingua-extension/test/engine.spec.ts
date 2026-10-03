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
      addPack(): string {
        received.push(["addPack", []]);
        return "es";
      }
      studiedLanguages(...args: unknown[]): string {
        received.push(["studiedLanguages", args]);
        return '["es","en"]';
      }
      setStudiedLanguages(...args: unknown[]): void {
        received.push(["setStudiedLanguages", args]);
      }
    }
    const mod = { default: async () => {}, LinguaEngine } as unknown as WasmModule;
    return { load: async () => mod, received };
  }

  it("hands the engine the view's language, and none on a whole-reader call", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    await port.for("es").analyse(["El faro"]);
    expect(await port.for("en").gloss("city")).toBe("ville");
    expect(await port.exportStatusOps()).toEqual([]);
    expect(await port.languages()).toEqual(["en", "es"]);

    expect(glue.received).toEqual([
      ["addPack", []],
      ["analyse", [["El faro"], "es"]],
      ["gloss", ["city", "en"]],
      ["exportStatusOps", []],
      ["languages", []],
    ]);
  });

  it("forwards the reader's studied languages as whole-reader calls, loading no pack", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    expect(await port.studiedLanguages()).toEqual(["es", "en"]);
    await port.setStudiedLanguages(["es", "en"]);

    expect(glue.received).toEqual([
      ["studiedLanguages", []],
      ["setStudiedLanguages", [["es", "en"]]],
    ]);
  });

  it("binds the view to the language it was asked", () => {
    const port = new WasmAnalyzerPort(recordingGlue().load);
    expect(port.for("es").language).toBe("es");
    expect(port.for("en").language).toBe("en");
  });
});

describe("WasmAnalyzerPort packs", () => {
  const EN_FR = "ext://assets/packs/en-fr.lingua";
  const ES_FR = "ext://assets/packs/es-fr.lingua";
  let fetched: string[];
  let failing: Set<string>;

  beforeEach(() => {
    fetched = [];
    failing = new Set();
    vi.stubGlobal("chrome", { runtime: { getURL: (path: string) => `ext://${path}` } });
    // A pack's bytes are its URL, so the fake engine knows which language it was handed.
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        fetched.push(url);
        if (failing.delete(url)) throw new Error("fetch failed");
        return { arrayBuffer: async () => new TextEncoder().encode(url).buffer };
      }),
    );
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  /** The packs fetched so far, without the WASM module's own fetch. */
  const packs = () => fetched.filter((url) => url.includes("/packs/"));

  /** A glue whose engine holds the packs it is handed and records each call, with the languages it held. */
  function packGlue() {
    const calls: Array<[string, unknown[]]> = [];
    const studied = (bytes: Uint8Array) => /packs\/([a-z]{2})-/.exec(new TextDecoder().decode(bytes))?.[1] ?? "?";
    class LinguaEngine {
      private readonly held: string[];
      constructor(bytes: Uint8Array) {
        this.held = [studied(bytes)];
      }
      addPack(bytes: Uint8Array): string {
        const language = studied(bytes);
        calls.push(["addPack", [language]]);
        this.held.push(language);
        return language;
      }
      analyse(_blocks: string[], language?: string | null): string {
        calls.push(["analyse", [language]]);
        if (!this.held.includes(language ?? "")) throw new Error(`no pack for "${language}"`);
        return '{"analyzer_version":"1.1.0","analysable":false,"tokens":[],"counted":0,"known":0,"percent":null}';
      }
      applyStatusChanges(): number {
        calls.push(["applyStatusChanges", [[...this.held]]]);
        return 0;
      }
      applyCardOps(): number {
        calls.push(["applyCardOps", [[...this.held]]]);
        return 0;
      }
      applyDeclaredLevelChanges(): number {
        calls.push(["applyDeclaredLevelChanges", [[...this.held]]]);
        return 0;
      }
      languages(): string {
        return JSON.stringify(this.held);
      }
      backup(): string {
        return "{}";
      }
      restore(): void {}
      dueCount(): number {
        return 0;
      }
      deckCount(): number {
        return 0;
      }
      trackedCount(): number {
        return 0;
      }
      exportStatusOps(): string {
        return "[]";
      }
      exportCardOps(): string {
        return "[]";
      }
      exportDeclaredLevels(): string {
        return "[]";
      }
    }
    const mod = { default: async () => {}, LinguaEngine } as unknown as WasmModule;
    return { load: async () => mod, calls };
  }

  it("starts with the default pair's pack alone", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load);

    await port.for("en").analyse(["The lighthouse"]);

    expect(packs()).toEqual([EN_FR]);
    expect(glue.calls).toEqual([["analyse", ["en"]]]);
  });

  it("refuses a language no shipped pair studies, before the engine", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr"]);

    await expect(port.for("es").analyse(["El faro"])).rejects.toThrow(
      'no shipped pack studies "es" (shipped pairs: en-fr)',
    );

    expect(glue.calls).toEqual([]);
    expect(packs()).toEqual([]);
  });

  it("adds another listed pair's pack once, before its language's first call", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    await port.for("es").analyse(["El faro"]);
    await port.for("es").analyse(["La ciudad"]);
    await port.for("en").analyse(["The city"]);

    expect(packs()).toEqual([EN_FR, ES_FR]);
    expect(glue.calls).toEqual([
      ["addPack", ["es"]],
      ["analyse", ["es"]],
      ["analyse", ["es"]],
      ["analyse", ["en"]],
    ]);
    expect(await port.languages()).toEqual(["en", "es"]);
  });

  it("shares one load between first calls made at the same time", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    await Promise.all([port.for("es").analyse(["El faro"]), port.for("es").analyse(["La ciudad"])]);

    expect(packs()).toEqual([EN_FR, ES_FR]);
    expect(glue.calls.filter(([name]) => name === "addPack")).toEqual([["addPack", ["es"]]]);
  });

  it("fetches a pack again after its load failed", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);
    failing.add(ES_FR);

    await expect(port.for("es").analyse(["El faro"])).rejects.toThrow("fetch failed");
    await port.for("es").analyse(["El faro"]);

    expect(packs()).toEqual([EN_FR, ES_FR, ES_FR]);
    expect(glue.calls).toEqual([
      ["addPack", ["es"]],
      ["analyse", ["es"]],
    ]);
  });

  const statusChange = (language: string) => ({
    language,
    lemma: "faro",
    status: "known",
    provenance: "manual",
    updated_at: 1,
  });
  const cardOp = (language: string) => ({
    client_id: "faro",
    language,
    lemma: "faro",
    surface_form: "faro",
    source_sentence: "El faro.",
    source: "",
    gloss: "phare",
    fsrs_state: "{}",
    deleted: false,
    client_ts: 1,
    device_id: "d",
  });
  const levelChange = (language: string) => ({ language, level: "A2", updated_at: 1 });

  it.each([
    [
      "applyStatusChanges",
      (port: WasmAnalyzerPort, language: string) => port.applyStatusChanges([statusChange(language)]),
    ],
    ["applyCardOps", (port: WasmAnalyzerPort, language: string) => port.applyCardOps([cardOp(language)])],
    [
      "applyDeclaredLevelChanges",
      (port: WasmAnalyzerPort, language: string) => port.applyDeclaredLevelChanges([levelChange(language)]),
    ],
  ])("%s first loads the pack of each listed language its records name", async (method, apply) => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    await apply(port, "en");
    await apply(port, "pt"); // no listed pair studies it: handed to the engine, which skips it
    expect(packs()).toEqual([EN_FR]);
    await apply(port, "es");

    expect(packs()).toEqual([EN_FR, ES_FR]);
    expect(glue.calls).toEqual([
      [method, [["en"]]],
      [method, [["en"]]],
      ["addPack", ["es"]],
      [method, [["en", "es"]]],
    ]);
  });

  it("loads no pack for a call about the whole reader", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    await port.restore("{}");
    await port.backup();
    await port.dueCount(0);
    await port.deckCount();
    await port.trackedCount();
    await port.exportStatusOps();
    await port.exportCardOps();
    await port.exportDeclaredLevels();
    await port.languages();

    expect(packs()).toEqual([EN_FR]);
    expect(glue.calls).toEqual([]);
  });
});
