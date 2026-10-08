import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { reprofileBackup, WasmAnalyzerPort, type WasmModule } from "@/analyzer/engine.ts";

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
      detectLanguage(...args: unknown[]): string {
        received.push(["detectLanguage", args]);
        return "es";
      }
      dueCount(...args: unknown[]): number {
        received.push(["dueCount", args]);
        return 2;
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
    expect(await port.deckCount(["es"])).toBe(3);
    await port.deckCount();

    expect(glue.received).toEqual([
      ["dueCount", [100, ["es"]]],
      ["dueCount", [100, null]],
      ["startReview", [100, ["es"]]],
      ["startReview", [100, null]],
      ["deckCount", [["es"]]],
      ["deckCount", [null]],
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
    /** Each restore, with the native language of the engine it landed in. */
    const restores: Array<[string, string]> = [];
    const studied = (bytes: Uint8Array) => /packs\/([a-z]{2})-/.exec(new TextDecoder().decode(bytes))?.[1] ?? "?";
    const glossedIn = (bytes: Uint8Array) =>
      /packs\/[a-z]{2}-([a-z]{2})/.exec(new TextDecoder().decode(bytes))?.[1] ?? "?";
    class LinguaEngine {
      private readonly held: string[];
      private readonly native: string;
      /** The native language the restored backup's profile names: `{"native": "en"}` in these specs. */
      private profileNative: string;
      constructor(bytes: Uint8Array) {
        this.held = [studied(bytes)];
        this.native = glossedIn(bytes);
        this.profileNative = this.native;
      }
      nativeLanguage(): string {
        return this.native;
      }
      profileNativeLanguage(): string {
        return this.profileNative;
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

      expect(packs()).toEqual([EN_FR, ES_FR]);
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

      expect(packs()).toEqual([ES_EN]);
      expect(glue.calls).toEqual([["analyse", ["es"]]]);
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
      expect(packs()).toEqual([EN_FR]);
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
      expect(packs()).toEqual([EN_FR, ES_FR]);
    });

    describe("a backup of another native language (add-lingua-native-language-choice D3)", () => {
      const ENGLISH_NATIVE = '{"native":"en"}';

      it("rebuilds the engine for it and restores the state there", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "fr");
        await port.for("es").analyse(["El faro"]);
        expect(packs()).toEqual([EN_FR, ES_FR]);

        await port.restore(ENGLISH_NATIVE);

        expect(await port.nativeLanguage()).toBe("en");
        expect(packs()).toEqual([EN_FR, ES_FR, ES_EN]);
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
        expect(packs()).toEqual([EN_FR, ES_FR, ES_EN]);
        expect(await port.languages()).toEqual(["es"]);
      });

      it("keeps the engine when the backup names its own native language, or one nothing is glossed in", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-fr"], async () => "fr");

        await port.restore('{"native":"fr"}');
        await port.restore(ENGLISH_NATIVE); // no listed pair is glossed in English: served in French (M22)

        expect(await port.nativeLanguage()).toBe("fr");
        expect(packs()).toEqual([EN_FR]);
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
        expect(packs()).toEqual([EN_FR, ES_FR, ES_EN]);
        expect(glue.restores).toEqual([["fr", ENGLISH_NATIVE]]);

        await port.restore(ENGLISH_NATIVE);

        expect(await port.nativeLanguage()).toBe("en");
        expect(packs()).toEqual([EN_FR, ES_FR, ES_EN, ES_EN]);
        expect(glue.restores).toEqual([
          ["fr", ENGLISH_NATIVE],
          ["fr", ENGLISH_NATIVE],
          ["en", ENGLISH_NATIVE],
        ]);
      });

      it("follows the reader back to French", async () => {
        const glue = packGlue();
        const port = new WasmAnalyzerPort(glue.load, MIXED, async () => "en");
        await port.for("es").analyse(["El faro"]);

        await port.restore('{"native":"fr"}');

        expect(await port.nativeLanguage()).toBe("fr");
        expect(packs()).toEqual([ES_EN, EN_FR]);
        await port.for("es").analyse(["El faro"]); // es-fr added again, for this engine
        expect(packs()).toEqual([ES_EN, EN_FR, ES_FR]);
      });
    });

    it("loads a French reader's records' packs alone", async () => {
      const glue = packGlue();
      const port = new WasmAnalyzerPort(glue.load, ["en-fr", "es-en"], async () => "fr");

      await port.applyStatusChanges([statusChange("es")]);

      expect(packs()).toEqual([EN_FR]);
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

    expect(packs()).toEqual([EN_FR]);
    expect(glue.calls).toEqual([]);
  });
});
