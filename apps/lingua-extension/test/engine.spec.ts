import { readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { reprofileBackup, reviewOutlookJson, WasmAnalyzerPort, type WasmModule } from "@/analyzer/engine.ts";
import { localDayStart } from "@/review/session.ts";
import { realGlue, realPort } from "./real-wasm.ts";

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
    /** The engine the port starts, on a pack's base and layer (ship-lingua-pack-bases-once D2). */
    static assemble(): LinguaEngine {
      return new LinguaEngine();
    }
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

  it("rewrites a backup's profile through the glue, once the module is initialised (D2)", async () => {
    const glue = fakeGlue();
    const reprofile = vi.fn((backup: string, native: string, studied: string[]) =>
      JSON.stringify({ backup, native, studied }),
    );
    const mod = { ...(await glue.load()), reprofileBackup: reprofile } as WasmModule;

    const rewritten = await reprofileBackup("{}", "en", ["es"], async () => mod);

    expect(JSON.parse(rewritten)).toEqual({ backup: "{}", native: "en", studied: ["es"] });
    expect(glue.init).toHaveBeenCalledTimes(1);
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
      static assemble(): LinguaEngine {
        return new LinguaEngine();
      }
      analyse(...args: unknown[]): string {
        received.push(["analyse", args]);
        return '{"analyzer_version":"1.1.0","analysable":false,"tokens":[],"counted":0,"known":0,"percent":null}';
      }
      gloss(...args: unknown[]): string | undefined {
        received.push(["gloss", args]);
        return "ville";
      }
      wordGrammar(...args: unknown[]): string {
        received.push(["wordGrammar", args]);
        return '{"gloss":"être","senses":[],"readings":[],"others":[],"pieces":[]}';
      }
      frequencyRank(...args: unknown[]): number | undefined {
        received.push(["frequencyRank", args]);
        return args[0] === "ser" ? 22 : undefined;
      }
      exportStatusOps(...args: unknown[]): string {
        received.push(["exportStatusOps", args]);
        return "[]";
      }
      languages(...args: unknown[]): string {
        received.push(["languages", args]);
        return '["en","es"]';
      }
      addParts(): string {
        received.push(["addParts", []]);
        return "es";
      }
      studiedLanguages(...args: unknown[]): string {
        received.push(["studiedLanguages", args]);
        return '["es","en"]';
      }
      setStudiedLanguages(...args: unknown[]): void {
        received.push(["setStudiedLanguages", args]);
      }
      detectLanguage(...args: unknown[]): string {
        received.push(["detectLanguage", args]);
        return "es";
      }
      dueCount(...args: unknown[]): number {
        received.push(["dueCount", args]);
        return 2;
      }
      reviewableCount(...args: unknown[]): number {
        received.push(["reviewableCount", args]);
        return 1;
      }
      deckCount(...args: unknown[]): number {
        received.push(["deckCount", args]);
        return 3;
      }
      startReview(...args: unknown[]): number {
        received.push(["startReview", args]);
        return 2;
      }
      reviewCurrent(): string {
        return '{"headword":"faro","surface":"faro","sentence":"El faro.","gloss":"phare","revealed":false,"remaining":2}';
      }
      reviewCurrentLanguage(): string {
        return "es";
      }
      reviewGrade(...args: unknown[]): boolean {
        received.push(["reviewGrade", args]);
        return true;
      }
      reviewIgnore(...args: unknown[]): void {
        received.push(["reviewIgnore", args]);
      }
      keepGloss(...args: unknown[]): boolean {
        received.push(["keepGloss", args]);
        return true;
      }
      reviewSummary(): string {
        return '{"reviewed":3,"recovered":1,"holding":1,"known":0,"hidden":1}';
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
      ["addParts", []],
      ["analyse", [["El faro"], "es"]],
      ["gloss", ["city", "en"]],
      ["exportStatusOps", []],
      ["languages", []],
    ]);
  });

  it("keeps a gloss on the view's language, the clock last (add-lingua-review-translation-fallback D5)", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    expect(await port.for("es").keepGloss("faro", "lumière", 1_770_000_000)).toBe(true);

    expect(glue.received).toEqual([
      ["addParts", []],
      ["keepGloss", ["faro", "lumière", "es", 1_770_000_000]],
    ]);
  });

  it("answers a word's grammar with its frequency rank, null when unranked (add-lingua-card-frequency)", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    expect(await port.for("es").wordGrammar("Es", "ser")).toEqual({
      gloss: "être",
      senses: [],
      readings: [],
      others: [],
      pieces: [],
      rank: 22,
    });
    expect((await port.for("es").wordGrammar("Madrid", "madrid")).rank).toBeNull();
    expect(glue.received).toContainEqual(["wordGrammar", ["Es", "ser", "es"]]);
    expect(glue.received).toContainEqual(["frequencyRank", ["ser", "es"]]);
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

  it("asks a document's language as a whole-reader call, loading no pack", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);
    expect(await port.detectLanguage(["El faro"], ["en", "es"], "es")).toBe("es");
    expect(glue.received).toEqual([["detectLanguage", [["El faro"], ["en", "es"], "es"]]]);
  });

  it("reviews in some languages or in all, as whole-reader calls loading no pack", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    expect(await port.dueCount(100, ["es"])).toBe(2);
    await port.dueCount(100);
    await port.startReview(100, ["es"]);
    await port.startReview(100);
    await port.startReview(100, undefined, { limit: 10, newPerDay: 5, dayStart: 86_400 });
    await port.startReview(100, ["es"], { limit: 10, practice: true });
    expect(await port.deckCount(["es"])).toBe(3);
    await port.deckCount();
    expect(await port.reviewableCount(100, ["es"], { newPerDay: 10, dayStart: 86_400 })).toBe(1);
    await port.reviewableCount(100);

    expect(glue.received).toEqual([
      ["dueCount", [100, ["es"]]],
      ["dueCount", [100, null]],
      ["startReview", [100, ["es"], null, null, null, null]],
      ["startReview", [100, null, null, null, null, null]],
      ["startReview", [100, null, 10, 5, 86_400, null]],
      ["startReview", [100, ["es"], 10, null, null, true]],
      ["deckCount", [["es"]]],
      ["deckCount", [null]],
      ["reviewableCount", [100, ["es"], 10, 86_400]],
      ["reviewableCount", [100, null, null, null]],
    ]);
  });

  it("answers, hides and sums up a session (refine-lingua-review-session)", async () => {
    const glue = recordingGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    expect(await port.reviewGrade("again", 100)).toBe(true);
    await port.reviewIgnore(100);
    expect(await port.reviewSummary()).toEqual({ reviewed: 3, recovered: 1, holding: 1, known: 0, hidden: 1 });

    expect(glue.received).toEqual([
      ["reviewGrade", ["again", 100]],
      ["reviewIgnore", [100]],
    ]);
  });

  it("gives the card being reviewed its language, beside the view the baseline pins", async () => {
    const port = new WasmAnalyzerPort(recordingGlue().load, ["en-fr", "es-fr"]);

    expect(await port.reviewCurrent()).toEqual({
      headword: "faro",
      surface: "faro",
      sentence: "El faro.",
      gloss: "phare",
      revealed: false,
      remaining: 2,
      language: "es",
    });
  });

  it("binds the view to the language it was asked", () => {
    const port = new WasmAnalyzerPort(recordingGlue().load);
    expect(port.for("es").language).toBe("es");
    expect(port.for("en").language).toBe("en");
  });
});

describe("WasmAnalyzerPort packs", () => {
  // A pair's pack is its studied language's base and its own layer (ship-lingua-pack-bases-once D2).
  const EN = "ext://assets/bases/en.lingua";
  const ES = "ext://assets/bases/es.lingua";
  const FR = "ext://assets/bases/fr.lingua";
  const EN_FR = "ext://assets/packs/en-fr.lingua";
  const ES_FR = "ext://assets/packs/es-fr.lingua";
  let fetched: string[];
  let failing: Set<string>;

  beforeEach(() => {
    fetched = [];
    failing = new Set();
    vi.stubGlobal("chrome", { runtime: { getURL: (path: string) => `ext://${path}` } });
    // A part's bytes are its URL, so the fake engine knows which language it was handed.
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

  /** The parts fetched so far — bases and layers —, without the WASM module's own fetch. */
  const packs = () => fetched.filter((url) => url.endsWith(".lingua"));

  /**
   * A glue whose engine holds the packs it is handed — a base and a layer each, which it refuses to
   * assemble across two languages, as the engine does — and records each call, with the languages it
   * held.
   */
  function packGlue() {
    const calls: Array<[string, unknown[]]> = [];
    /** Each restore, with the native language of the engine it landed in. */
    const restores: Array<[string, string]> = [];
    const url = (bytes: Uint8Array) => new TextDecoder().decode(bytes);
    /** A base's language, checked against its layer's. */
    const studied = (base: Uint8Array, layer: Uint8Array) => {
      const language = /bases\/([a-z]{2})\.lingua$/.exec(url(base))?.[1] ?? "?";
      const layers = /packs\/([a-z]{2})-[a-z]{2}\.lingua$/.exec(url(layer))?.[1];
      if (language !== layers) throw new Error(`the layer ${url(layer)} is not of the base ${url(base)}`);
      return language;
    };
    const glossedIn = (layer: Uint8Array) => /packs\/[a-z]{2}-([a-z]{2})/.exec(url(layer))?.[1] ?? "?";
    class LinguaEngine {
      static assemble(base: Uint8Array, layer: Uint8Array): LinguaEngine {
        return new LinguaEngine(base, layer);
      }
      private readonly held: string[];
      private readonly native: string;
      /** The native language the restored backup's profile names: `{"native": "en"}` in these specs. */
      private profileNative: string;
      private constructor(base: Uint8Array, layer: Uint8Array) {
        this.held = [studied(base, layer)];
        this.native = glossedIn(layer);
        this.profileNative = this.native;
      }
      nativeLanguage(): string {
        return this.native;
      }
      profileNativeLanguage(): string {
        return this.profileNative;
      }
      addParts(base: Uint8Array, layer: Uint8Array): string {
        const language = studied(base, layer);
        calls.push(["addParts", [language]]);
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
      restore(json: string): void {
        restores.push([this.native, json]);
        this.profileNative = (JSON.parse(json) as { native?: string }).native ?? this.native;
      }
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
    return { load: async () => mod, calls, restores };
  }

  it("starts with the default pair's base and layer alone", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load);

    await port.for("en").analyse(["The lighthouse"]);

    expect(packs()).toEqual([EN, EN_FR]);
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

  it("adds another listed pair's base and layer once, before its language's first call", async () => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);

    await port.for("es").analyse(["El faro"]);
    await port.for("es").analyse(["La ciudad"]);
    await port.for("en").analyse(["The city"]);

    expect(packs()).toEqual([EN, EN_FR, ES, ES_FR]);
    expect(glue.calls).toEqual([
      ["addParts", ["es"]],
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

    expect(packs()).toEqual([EN, EN_FR, ES, ES_FR]);
    expect(glue.calls.filter(([name]) => name === "addParts")).toEqual([["addParts", ["es"]]]);
  });

  it.each([
    ["its layer", ES_FR],
    ["its base", ES],
  ])("fetches both parts again after %s failed to load", async (_part, failed) => {
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);
    failing.add(failed);

    await expect(port.for("es").analyse(["El faro"])).rejects.toThrow("fetch failed");
    await port.for("es").analyse(["El faro"]);

    // The language's load is forgotten whole, whichever part failed (ship-lingua-pack-bases-once D2).
    expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_FR]);
    expect(glue.calls).toEqual([
      ["addParts", ["es"]],
      ["analyse", ["es"]],
    ]);
  });

  it("names the part a failed response left out, and fetches both parts again on the next call", async () => {
    // The Apple app's page answers a file it cannot find with a 404 and an empty body: the load fails
    // naming the file and the status, never as a malformed container. An extension scheme's status 0
    // is a response like any other.
    const glue = packGlue();
    const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"]);
    const served = vi.mocked(fetch).getMockImplementation()!;
    vi.mocked(fetch).mockImplementation(async (url) => {
      const response = await served(url);
      if (String(url) === ES)
        return { ...response, status: 404, arrayBuffer: async () => new ArrayBuffer(0) } as Response;
      return { ...response, status: 0 } as Response;
    });

    await port.for("en").analyse(["The lighthouse"]);
    await expect(port.for("es").analyse(["El faro"])).rejects.toThrow(
      "assets/bases/es.lingua could not be read (HTTP 404)",
    );
    expect(glue.calls.filter(([name]) => name === "addParts")).toEqual([]);

    vi.mocked(fetch).mockImplementation(served);
    await port.for("es").analyse(["El faro"]);
    expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_FR]);
    expect(await port.languages()).toEqual(["en", "es"]);
  });

  it("fetches each language's base once per engine, beside one layer per language", async () => {
    // The six pairs, read by an English speaker: Spanish through es-en, French through fr-en; the
    // Spanish base is es-en's whatever es-fr's is, and no file is fetched twice.
    const glue = packGlue();
    const port = new WasmAnalyzerPort(
      glue.load,
      ["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"],
      async () => "en",
    );

    await port.for("es").analyse(["El faro"]);
    await port.for("fr").analyse(["Le phare"]);
    await port.for("es").analyse(["La ciudad"]);
    await port.for("fr").analyse(["La ville"]);
    await expect(port.for("en").analyse(["The city"])).rejects.toThrow('no shipped pack studies "en"');

    expect(packs()).toEqual([ES, "ext://assets/packs/es-en.lingua", FR, "ext://assets/packs/fr-en.lingua"]);
    expect(new Set(packs()).size).toBe(packs().length);
    expect(await port.languages()).toEqual(["es", "fr"]);
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
    gloss_language: "fr",
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
    expect(packs()).toEqual([EN, EN_FR]);
    await apply(port, "es");

    expect(packs()).toEqual([EN, EN_FR, ES, ES_FR]);
    expect(glue.calls).toEqual([
      [method, [["en"]]],
      [method, [["en"]]],
      ["addParts", ["es"]],
      [method, [["en", "es"]]],
    ]);
  });

  describe("for the reader's native language (generalise-lingua-native-language D7)", () => {
    const ES_EN = "ext://assets/packs/es-en.lingua";
    const MIXED = ["en-fr", "es-fr", "es-en"];

    it("loads a French reader's pairs alone, never es-en, asking the native language once", async () => {
      const glue = packGlue();
      const resolve = vi.fn(async () => "fr" as const);
      const port = new WasmAnalyzerPort(glue.load, MIXED, resolve);

      await port.for("en").analyse(["The lighthouse"]);
      await port.for("es").analyse(["El faro"]);
      await port.for("es").analyse(["La ciudad"]);

      expect(packs()).toEqual([EN, EN_FR, ES, ES_FR]);
      expect(packs()).not.toContain(ES_EN);
      expect(await port.nativeLanguage()).toBe("fr");
      expect(resolve).toHaveBeenCalledOnce();
    });

    it("starts an English reader on es-en, and refuses English before the engine", async () => {
      const glue = packGlue();
      const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "en");

      await port.for("es").analyse(["El faro"]);
      await expect(port.for("en").analyse(["The lighthouse"])).rejects.toThrow(
        'no shipped pack studies "en" (shipped pairs: es-en)',
      );

      expect(packs()).toEqual([ES, ES_EN]);
      expect(glue.calls).toEqual([["analyse", ["es"]]]);
    });

    it("reads French through the reader's pair: fr-en beside es-en, fr-es beside en-es, none for a French reader (enable-lingua-french D3)", async () => {
      const SIX = ["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"];
      const pack = (pair: string) => `ext://assets/packs/${pair}.lingua`;
      const base = (studied: string) => `ext://assets/bases/${studied}.lingua`;

      const english = packGlue();
      const en = new WasmAnalyzerPort(english.load, SIX, async () => "en");
      await en.for("fr").analyse(["Le phare"]);
      await en.for("es").analyse(["El faro"]);
      expect(packs()).toEqual([base("es"), pack("es-en"), base("fr"), pack("fr-en")]);
      expect(english.calls).toEqual([
        ["addParts", ["fr"]],
        ["analyse", ["fr"]],
        ["analyse", ["es"]],
      ]);

      fetched.length = 0;
      const spanish = packGlue();
      const es = new WasmAnalyzerPort(spanish.load, SIX, async () => "es");
      await es.for("fr").analyse(["Le phare"]);
      expect(packs()).toEqual([base("en"), pack("en-es"), base("fr"), pack("fr-es")]);

      fetched.length = 0;
      const french = packGlue();
      const fr = new WasmAnalyzerPort(french.load, SIX, async () => "fr");
      await expect(fr.for("fr").analyse(["Le phare"])).rejects.toThrow(
        'no shipped pack studies "fr" (shipped pairs: en-fr, es-fr)',
      );
      expect(packs()).toEqual([]);
      expect(french.calls).toEqual([]);
    });

    it("answers the native language without fetching a pack", async () => {
      const glue = packGlue();
      const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "en");

      expect(await port.nativeLanguage()).toBe("en");
      expect(fetched).toEqual([]);
      // The default resolver: every reader today.
      expect(await new WasmAnalyzerPort(glue.load, MIXED).nativeLanguage()).toBe("fr");
      expect(fetched).toEqual([]);
    });

    it("fails before any fetch for a native language no listed pair is glossed in", async () => {
      const glue = packGlue();
      const load = vi.fn(glue.load);
      const port = new WasmAnalyzerPort(load, ["en-fr", "es-fr"], async () => "es");

      await expect(port.for("en").analyse(["The lighthouse"])).rejects.toThrow(
        'no shipped pair is glossed in "es" (shipped pairs: en-fr, es-fr)',
      );
      await expect(port.languages()).rejects.toThrow(/glossed in "es"/);
      await expect(port.nativeLanguage()).rejects.toThrow(/glossed in "es"/);

      expect(fetched).toEqual([]);
      expect(load).not.toHaveBeenCalled();
    });

    it("keeps a French reader's refusal word for word", async () => {
      const glue = packGlue();
      const port = new WasmAnalyzerPort(glue.load, ["en-fr"], async () => "fr");

      await expect(port.for("es").analyse(["El faro"])).rejects.toThrow(
        'no shipped pack studies "es" (shipped pairs: en-fr)',
      );
      expect(fetched).toEqual([]);
    });

    it("asks again after the store failed to answer", async () => {
      const glue = packGlue();
      const resolve = vi
        .fn<() => Promise<"fr">>()
        .mockRejectedValueOnce(new Error("store unavailable"))
        .mockResolvedValue("fr");
      const port = new WasmAnalyzerPort(glue.load, MIXED, resolve);

      await expect(port.for("en").analyse(["The lighthouse"])).rejects.toThrow("store unavailable");
      await port.for("en").analyse(["The lighthouse"]);

      expect(resolve).toHaveBeenCalledTimes(2);
      expect(packs()).toEqual([EN, EN_FR]);
    });

    // The content script's probe and hydration reach the engine through a whole-reader call first:
    // a store that failed to answer it must not leave every later call failing with it.
    it.each([
      ["languages", (port: WasmAnalyzerPort) => port.languages()],
      ["backup", (port: WasmAnalyzerPort) => port.backup()],
      ["restore", (port: WasmAnalyzerPort) => port.restore("{}")],
    ])("asks again after the store failed to answer a first %s call", async (_name, first) => {
      const glue = packGlue();
      const load = vi.fn(glue.load);
      const resolve = vi
        .fn<() => Promise<"fr">>()
        .mockRejectedValueOnce(new Error("store unavailable"))
        .mockResolvedValue("fr");
      const port = new WasmAnalyzerPort(load, MIXED, resolve);

      await expect(first(port)).rejects.toThrow("store unavailable");
      expect(load).not.toHaveBeenCalled();
      expect(await port.languages()).toEqual(["en"]);
      await port.for("es").analyse(["El faro"]);

      expect(resolve).toHaveBeenCalledTimes(2);
      expect(load).toHaveBeenCalledOnce();
      expect(packs()).toEqual([EN, EN_FR, ES, ES_FR]);
    });

    describe("a backup of another native language (add-lingua-native-language-choice D3)", () => {
      const ENGLISH_NATIVE = '{"native":"en"}';

      it("rebuilds the engine for it and restores the state there", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "fr");
        await port.for("es").analyse(["El faro"]);
        expect(packs()).toEqual([EN, EN_FR, ES, ES_FR]);

        await port.restore(ENGLISH_NATIVE);

        expect(await port.nativeLanguage()).toBe("en");
        // The engine built for English fetches its own parts, the Spanish base among them.
        expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_EN]);
        // Restored first where it arrived, then in the engine built for its native language.
        expect(glue.restores).toEqual([
          ["fr", ENGLISH_NATIVE],
          ["en", ENGLISH_NATIVE],
        ]);
        // Spanish is es-en's now, the engine's first pack: nothing is added for it.
        await port.for("es").analyse(["La ciudad"]);
        await expect(port.for("en").analyse(["The city"])).rejects.toThrow(
          'no shipped pack studies "en" (shipped pairs: es-en)',
        );
        expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_EN]);
        expect(await port.languages()).toEqual(["es"]);
      });

      it("keeps the engine when the backup names its own native language, or one nothing is glossed in", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"], async () => "fr");

        await port.restore('{"native":"fr"}');
        await port.restore(ENGLISH_NATIVE); // no listed pair is glossed in English: served in French (M22)

        expect(await port.nativeLanguage()).toBe("fr");
        expect(packs()).toEqual([EN, EN_FR]);
        expect(glue.restores.map(([native]) => native)).toEqual(["fr", "fr"]);
      });

      it("shares one rebuild between the restores of one change", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "fr");

        await Promise.all([port.restore(ENGLISH_NATIVE), port.restore(ENGLISH_NATIVE)]);

        expect(packs().filter((url) => url === ES_EN)).toHaveLength(1);
        expect(glue.restores.filter(([native]) => native === "en")).toHaveLength(2);
        expect(await port.nativeLanguage()).toBe("en");
      });

      it("keeps serving the restored engine after a rebuild whose pack failed to load, then builds again", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "fr");
        await port.for("es").analyse(["El faro"]);
        failing.add(ES_EN);

        await expect(port.restore(ENGLISH_NATIVE)).rejects.toThrow("fetch failed");

        // The French engine, the backup restored in it and its Spanish pack still there: no fresh,
        // empty engine is built for a surface to serve or persist.
        expect(await port.nativeLanguage()).toBe("fr");
        expect(await port.languages()).toEqual(["en", "es"]);
        await port.for("es").analyse(["El faro"]);
        await port.backup();
        expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_EN]);
        expect(glue.restores).toEqual([["fr", ENGLISH_NATIVE]]);

        await port.restore(ENGLISH_NATIVE);

        expect(await port.nativeLanguage()).toBe("en");
        expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_EN, ES, ES_EN]);
        expect(glue.restores).toEqual([
          ["fr", ENGLISH_NATIVE],
          ["fr", ENGLISH_NATIVE],
          ["en", ENGLISH_NATIVE],
        ]);
      });

      it("adds a pack again to the engine a round trip builds: French, English, French, es-fr added first", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "fr");
        await port.for("es").analyse(["El faro"]); // es-fr added to the first French engine

        await port.restore(ENGLISH_NATIVE);
        await port.restore('{"native":"fr"}');
        await port.for("es").analyse(["El faro"]);

        // The second French engine is handed es-fr again: what the first held is not its own.
        expect(packs()).toEqual([EN, EN_FR, ES, ES_FR, ES, ES_EN, EN, EN_FR, ES, ES_FR]);
        expect(glue.calls.filter(([name]) => name !== "analyse")).toEqual([
          ["addParts", ["es"]],
          ["addParts", ["es"]],
        ]);
      });

      it("follows the reader back to French", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "en");
        await port.for("es").analyse(["El faro"]);

        await port.restore('{"native":"fr"}');

        expect(await port.nativeLanguage()).toBe("fr");
        expect(packs()).toEqual([ES, ES_EN, EN, EN_FR]);
        await port.for("es").analyse(["El faro"]); // es-fr added again, for this engine
        expect(packs()).toEqual([ES, ES_EN, EN, EN_FR, ES, ES_FR]);
      });
    });

    it("loads a French reader's records' packs alone", async () => {
      const glue = packGlue();
      const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-en"], async () => "fr");

      await port.applyStatusChanges([statusChange("es")]);

      expect(packs()).toEqual([EN, EN_FR]);
      expect(glue.calls).toEqual([["applyStatusChanges", [["en"]]]]);
    });
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

    expect(packs()).toEqual([EN, EN_FR]);
    expect(glue.calls).toEqual([]);
  });
});

describe("the committed fixture's parts (ship-lingua-pack-bases-once D6)", () => {
  // test/fixtures/ holds the en-fr testdata pack whole and as its base and layer, all three written
  // by `yarn gen:fixtures`: assembled, the parts answer as the whole pack does, so the tests on the
  // real engine, which load the parts, read the pack the whole fixture is.
  const fixture = (name: string) => new Uint8Array(readFileSync(join(__dirname, "fixtures", name)));

  it("answers every analysis and gloss as the whole fixture does", () => {
    const glue = realGlue();
    const whole = new glue.LinguaEngine(fixture("en-fr.testdata.lingua"));
    const parts = glue.LinguaEngine.assemble(
      fixture("en.testdata.base.lingua"),
      fixture("en-fr.testdata.layer.lingua"),
    );
    const blocks = [
      ["The city runs; he ran and walked, seldom running."],
      ["Leaves can put a conundrum: it went, it was gone, it does."],
      ["Cities ran", "A conundrum seldom goes"],
    ];
    for (const block of blocks) expect(parts.analyse(block, "en")).toBe(whole.analyse(block, "en"));
    for (const lemma of [
      "run",
      "city",
      "seldom",
      "conundrum",
      "go",
      "leave",
      "leaf",
      "can",
      "put",
      "walk",
      "nothing",
    ]) {
      expect(parts.gloss(lemma, "en"), lemma).toBe(whole.gloss(lemma, "en"));
    }
    expect(parts.gloss("city", "en")).toBe("ville");
    expect(parts.notice("en")).toBe(whole.notice("en"));
    expect(parts.languages()).toBe(whole.languages());
    whole.free();
    parts.free();
  });
});

describe("the real engine's count of what a review would hold (add-lingua-comeback-hints D1)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("leaves out the new words beyond the day's allowance, as a session does", async () => {
    const port = realPort();
    const dayStart = localDayStart(1_790_000_000);
    const now = dayStart + 8 * 3_600;
    const english = port.for("en");
    for (let i = 0; i < 30; i++) {
      await english.addCard({
        lemma: `word${i}`,
        surface: `word${i}`,
        sentence: "",
        url: "",
        gloss: null,
        capturedAt: dayStart - 86_400 + i,
      });
    }
    const allowance = { newPerDay: 10, dayStart };

    expect(await port.dueCount(now, ["en"])).toBe(30);
    expect(await port.reviewableCount(now, ["en"], allowance)).toBe(10);
    expect(await port.reviewableCount(now, ["en"])).toBe(30);

    // Four new words answered this morning: six of today's allowance are left, as a session finds.
    await port.startReview(now, ["en"], { limit: 4, ...allowance });
    for (let i = 0; i < 4; i++) {
      await port.reviewReveal();
      await port.reviewGrade("good", now);
    }
    const later = now + 60;
    expect(await port.reviewableCount(later, ["en"], allowance)).toBe(6);
    expect(await port.startReview(later, ["en"], allowance)).toBe(6);
  });

  it("reads a backup's outlook through the glue, with no engine", async () => {
    const port = realPort();
    await port.for("en").addCard({
      lemma: "seldom",
      surface: "seldom",
      sentence: "",
      url: "",
      gloss: null,
      capturedAt: 100,
    });
    const json = await reviewOutlookJson(await port.backup(), 200, 0, async () => realGlue());
    expect(JSON.parse(json)).toMatchObject({
      version: 1,
      computedAt: 200,
      dayStart: 0,
      languages: [{ language: "en", cards: 1, due: 0, upcoming: [], fresh: 1, introduced: 0 }],
    });
    await expect(reviewOutlookJson("{ not json", 0, 0, async () => realGlue())).rejects.toThrow(/malformed backup/);
  });
});
