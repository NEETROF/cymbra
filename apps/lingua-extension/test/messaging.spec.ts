import { describe, expect, it, vi } from "vitest";
import { WasmAnalyzerPort, type WasmModule } from "@/analyzer/engine.ts";
import { MessagingLinguaPort } from "@/analyzer/messaging-port.ts";
import { handleRpc, isRpcRequest } from "@/analyzer/rpc-host.ts";
import type { LinguaPort } from "@/analyzer/port.ts";
import { RPC_TYPE, type RpcRequest } from "@/analyzer/rpc.ts";
import { makeFakePort } from "./helpers.ts";

describe("MessagingLinguaPort", () => {
  it("forwards each call as an RPC and returns the transport result", async () => {
    const send = vi.fn(async (method: string) => {
      if (method === "analyse")
        return { analysable: true, tokens: [], counted: 1, known: 1, percent: 100, analyzer_version: "1.0.0" };
      if (method === "deckCount") return 3;
      if (method === "backup") return "BACKUP";
      if (method === "licences") return ["A", "B"];
      return undefined;
    });
    const port = new MessagingLinguaPort(send);

    expect((await port.for("en").analyse(["a", "b"])).percent).toBe(100);
    expect(send).toHaveBeenCalledWith("analyse", [["a", "b"]], "en");
    expect(await port.deckCount()).toBe(3);
    expect(await port.backup()).toBe("BACKUP");
    expect(await port.for("en").licences()).toEqual(["A", "B"]);

    await port
      .for("en")
      .addCard({ lemma: "seldom", surface: "seldom", sentence: "s", url: "u", gloss: null, capturedAt: 1 });
    expect(send).toHaveBeenCalledWith(
      "addCard",
      [{ lemma: "seldom", surface: "seldom", sentence: "s", url: "u", gloss: null, capturedAt: 1 }],
      "en",
    );

    await port.reviewGrade("good", 42);
    expect(send).toHaveBeenCalledWith("reviewGrade", ["good", 42]);
  });

  it("forwards a selection to the engine's phrase gloss", async () => {
    const answer = {
      tokens: [{ surface: "gave", lemma: "give", class: "Known", gloss: "donner", function_word: false }],
    };
    const send = vi.fn(async (method: string) => (method === "phraseGloss" ? answer : undefined));
    const port = new MessagingLinguaPort(send);

    expect(await port.for("en").phraseGloss("gave up")).toBe(answer);
    expect(send).toHaveBeenCalledWith("phraseGloss", ["gave up"], "en");
  });

  it("forwards a word card's grammar question: the word as written and its dictionary form", async () => {
    const answer = { gloss: "Aller", senses: [], readings: [{ pos: "VERB" }], others: [], pieces: [] };
    const send = vi.fn(async (method: string) => (method === "wordGrammar" ? answer : undefined));
    const port = new MessagingLinguaPort(send);

    expect(await port.for("en").wordGrammar("went", "go")).toBe(answer);
    expect(send).toHaveBeenCalledWith("wordGrammar", ["went", "go"], "en");
  });

  it("forwards the sync methods", async () => {
    const send = vi.fn(async (method: string) =>
      method === "applyStatusChanges" || method === "applyCardOps" ? 2 : [],
    );
    const port = new MessagingLinguaPort(send);

    await port.for("en").setStatusAt("seldom", "known", 1700);
    expect(send).toHaveBeenCalledWith("setStatusAt", ["seldom", "known", 1700], "en");
    await port.exportStatusOps();
    expect(send).toHaveBeenCalledWith("exportStatusOps", []);
    expect(await port.applyStatusChanges([{ language: "en", lemma: "run", status: "known", updated_at: 5 }])).toBe(2);
    await port.exportCardOps();
    expect(send).toHaveBeenCalledWith("exportCardOps", []);
    expect(await port.applyCardOps([])).toBe(2);
  });
  // The host resolves a call by looking the method name up ON the port (`port[method]`),
  // so the wire name has to BE the method name. Nothing else checks that: a forwarder
  // sending "reviewMark" for reviewMarkKnown type-checks, and breaks only on the two
  // browsers that host the engine in the event page. So walk every method there is.
  it("forwards every method under its own name, with its arguments untouched — a view adds its language", async () => {
    const forwarded: Array<[string, unknown[], string | undefined]> = [];
    const send = vi.fn(async (method: string, args: unknown[], language?: string) => {
      forwarded.push([method, args, language]);
      return `answer:${method}`;
    });
    const root = new MessagingLinguaPort(send);
    const sides: Array<[object, string | undefined]> = [
      [root, undefined],
      [root.for("es"), "es"],
    ];

    let total = 0;
    for (const [side, language] of sides) {
      const port = side as unknown as Record<string, (...args: unknown[]) => Promise<unknown>>;
      const methods = Object.getOwnPropertyNames(Object.getPrototypeOf(side)).filter(
        (m) => m !== "constructor" && m !== "rpc" && m !== "for",
      );
      total += methods.length;
      for (const name of methods) {
        forwarded.length = 0;
        // Sentinels, not realistic values: what is under test is that they arrive in order.
        const args = Array.from({ length: port[name]!.length }, (_, i) => `${name}#${i}`);
        const answer = await port[name]!(...args);
        expect([name, forwarded]).toEqual([name, [[name, args, language]]]);
        expect(answer).toBe(`answer:${name}`);
      }
    }
    expect(total).toBeGreaterThan(30); // the seam is wide; this guards the filter
  });
});

describe("rpc host", () => {
  const req = (method: string, args: unknown[] = [], language?: string): RpcRequest =>
    language === undefined ? { type: RPC_TYPE, method, args } : { type: RPC_TYPE, method, args, language };

  it("recognises well-formed RPC requests only", () => {
    expect(isRpcRequest(req("analyse", [[]]))).toBe(true);
    expect(isRpcRequest({ type: "stats", pct: 1 })).toBe(false);
    expect(isRpcRequest(null)).toBe(false);
    expect(isRpcRequest({ type: RPC_TYPE, method: "x" })).toBe(false); // no args array
    expect(isRpcRequest(req("gloss", ["run"], "es"))).toBe(true);
    expect(isRpcRequest({ type: RPC_TYPE, method: "gloss", args: [], language: 3 })).toBe(false);
  });

  it("hydrates once, then dispatches to the engine port", async () => {
    const { port, calls } = makeFakePort();
    const ensure = vi.fn(async () => {});
    const res1 = await handleRpc(port, ensure, req("setCalibration", [2500], "en"));
    const res2 = await handleRpc(port, ensure, req("calibration", [], "en"));
    expect(res1).toEqual({ ok: true, result: undefined });
    expect(res2).toEqual({ ok: true, result: 2500 });
    expect(calls.setCalibration).toEqual([2500]);
    expect(calls.languages).toEqual(["en", "en"]); // answered on the English view
    expect(ensure).toHaveBeenCalledTimes(2); // ensure() is idempotent; caller memoises
  });

  it("answers a language-bound call on the port's view, a whole-reader call on the port", async () => {
    const asked: string[] = [];
    const port = {
      for: (language: string) => {
        asked.push(language);
        return { language, gloss: async (lemma: string) => `${language}:${lemma}` };
      },
      backup: async () => "BACKUP",
    } as unknown as LinguaPort;
    const ensure = async () => {};

    expect(await handleRpc(port, ensure, req("gloss", ["run"], "es"))).toEqual({ ok: true, result: "es:run" });
    expect(asked).toEqual(["es"]);
    expect(await handleRpc(port, ensure, req("backup"))).toEqual({ ok: true, result: "BACKUP" });
    // Each side has only its own methods: a call sent to the wrong one is unknown, never defaulted.
    expect(await handleRpc(port, ensure, req("gloss", ["run"]))).toEqual({ ok: false, error: "unknown method: gloss" });
    expect(await handleRpc(port, ensure, req("backup", [], "en"))).toEqual({
      ok: false,
      error: "unknown method: backup",
    });
  });

  it("returns an error result for an unknown method", async () => {
    const { port } = makeFakePort();
    const res = await handleRpc(port, async () => {}, req("nope", []));
    expect(res).toEqual({ ok: false, error: "unknown method: nope" });
  });

  it("answers a call in a language nothing ships with the engine port's refusal", async () => {
    const glue = vi.fn(async (): Promise<WasmModule> => {
      throw new Error("an engine was built");
    });
    const port = new WasmAnalyzerPort(glue, ["en-fr"]);

    expect(await handleRpc(port, async () => {}, req("analyse", [["El faro"]], "es"))).toEqual({
      ok: false,
      error: 'no shipped pack studies "es" (shipped pairs: en-fr)',
    });
    expect(glue).not.toHaveBeenCalled();
  });

  it("forwards the reader's studied languages as whole-reader calls", async () => {
    const send = vi.fn(async (method: string) => (method === "studiedLanguages" ? ["es", "en"] : undefined));
    const port = new MessagingLinguaPort(send);

    expect(await port.studiedLanguages()).toEqual(["es", "en"]);
    await port.setStudiedLanguages(["en"]);

    expect(send.mock.calls).toEqual([
      ["studiedLanguages", []],
      ["setStudiedLanguages", [["en"]]],
    ]);
    const { port: engine } = makeFakePort();
    expect(await handleRpc(engine, async () => {}, req("setStudiedLanguages", [["es", "en"]]))).toEqual({
      ok: true,
      result: undefined,
    });
    expect(await handleRpc(engine, async () => {}, req("studiedLanguages"))).toEqual({
      ok: true,
      result: ["es", "en"],
    });
    expect(await handleRpc(engine, async () => {}, req("detectLanguage", [["Hola"], ["es", "en"], null]))).toEqual({
      ok: true,
      result: "es",
    });
    const detect = vi.fn(async () => "es");
    await new MessagingLinguaPort(detect).detectLanguage(["Hola"], ["es", "en"], "es");
    expect(detect).toHaveBeenCalledWith("detectLanguage", [["Hola"], ["es", "en"], "es"]);
  });

  it("captures a thrown error as a failed result", async () => {
    const { port } = makeFakePort();
    const boom = async () => {
      throw new Error("kaboom");
    };
    const res = await handleRpc(port, boom, req("calibration", []));
    expect(res).toEqual({ ok: false, error: "kaboom" });
  });
});
