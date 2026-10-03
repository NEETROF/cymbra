import { describe, expect, it, vi } from "vitest";
import type { EngineAccess, EngineReply } from "@/translate/host/engine.ts";
import { relayTranslation, relayWarm } from "@/translate/host/relay.ts";

/** An engine that records the markup it is handed, and in which language, and answers what the test says. */
function engine(answer: (markup: string) => EngineReply | Promise<EngineReply>) {
  const seen: string[] = [];
  const languages: string[] = [];
  const access: Pick<EngineAccess, "translate"> = {
    translate: async (markup, language) => {
      seen.push(markup);
      languages.push(language);
      return answer(markup);
    },
  };
  return { access, seen, languages };
}

const sentence = "She gave up after the third attempt.";
const selection = { start: 4, end: 11 }; // "gave up"

describe("relayTranslation", () => {
  it("asks both translations in the request's language (generalise-lingua-translation-model-state D5)", async () => {
    const { access, languages } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence, selection, language: "es" });
    expect(languages).toEqual(["es", "es"]);
  });

  it("marks the selection in its sentence and reads the translation back", async () => {
    const { access, seen } = engine((markup) => ({
      ok: true,
      html: markup === "gave up" ? "A abandonné" : "Elle <b>a abandonné</b> après la troisième tentative.",
    }));
    const result = await relayTranslation(access, { sentence, selection, language: "en" });

    // The sentence with the selection tagged, then the selection alone — to check the tag.
    expect(seen).toEqual(["She <b>gave up</b> after the third attempt.", "gave up"]);
    expect(result.kind).toBe("translated");
    if (result.kind !== "translated") return;
    const { sentence: fr, marks } = result.translation;
    expect(fr).toBe("Elle a abandonné après la troisième tentative.");
    expect(marks.map((m) => fr.slice(m.start, m.end))).toEqual(["a abandonné"]);
  });

  it("escapes page text before it reaches the engine", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence: "a <b> b", selection: { start: 0, end: 1 }, language: "en" });
    expect(seen).toEqual(["<b>a</b> &lt;b&gt; b", "a"]);
  });

  it("escapes the selection it sends alone, too", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence: "a <b> b", selection: { start: 2, end: 5 }, language: "en" });
    expect(seen).toEqual(["a <b>&lt;b&gt;</b> b", "&lt;b&gt;"]);
  });

  it("translates the sentence unmarked when the selection's place is unknown", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "Elle a abandonné." }));
    const result = await relayTranslation(access, { sentence: "She gave up.", selection: null, language: "en" });
    expect(seen).toEqual(["She gave up."]);
    expect(result).toEqual({ kind: "translated", translation: { sentence: "Elle a abandonné.", marks: [] } });
  });

  describe("the check against the selection alone", () => {
    const friday = "They seldom ship on Friday, even when the customer asks nicely.";
    const seldom = { start: 5, end: 11 };
    const tagged = "Ils <b>expédient</b> rarement le vendredi, même lorsque le client demande bien.";

    const marked = async (alone: (markup: string) => EngineReply | Promise<EngineReply>) => {
      const { access } = engine((markup) => (markup === "seldom" ? alone(markup) : { ok: true, html: tagged }));
      const result = await relayTranslation(access, { sentence: friday, selection: seldom, language: "en" });
      if (result.kind !== "translated") throw new Error("expected a translation");
      const { sentence: fr, marks } = result.translation;
      return marks.map((m) => fr.slice(m.start, m.end));
    };

    it("moves a mark the engine put on the wrong word", async () => {
      // Measured: the tag lands on "expédient"; "seldom" alone is "rarement".
      await expect(marked(() => ({ ok: true, html: "rarement" }))).resolves.toEqual(["rarement"]);
    });

    it("keeps the tag's own mark when the selection alone gets no translation", async () => {
      await expect(marked(() => ({ ok: false, reason: "the translation timed out" }))).resolves.toEqual(["expédient"]);
    });

    it("keeps the tag's own mark when asking for the selection alone throws", async () => {
      await expect(marked(() => Promise.reject(new Error("gone")))).resolves.toEqual(["expédient"]);
    });
  });

  it("answers unavailable, and logs why, when the engine gives nothing", async () => {
    const log = vi.fn();
    const { access } = engine(() => ({ ok: false, reason: "the engine did not start" }));
    await expect(relayTranslation(access, { sentence, selection, language: "en" }, log)).resolves.toEqual({
      kind: "unavailable",
    });
    expect(log).toHaveBeenCalledWith("no translation:", "the engine did not start");
  });

  it("answers unavailable when reaching the engine throws", async () => {
    const log = vi.fn();
    const access: Pick<EngineAccess, "translate"> = { translate: () => Promise.reject(new Error("gone")) };
    await expect(relayTranslation(access, { sentence, selection, language: "en" }, log)).resolves.toEqual({
      kind: "unavailable",
    });
    expect(log).toHaveBeenCalledOnce();
  });

  it("does not wake the engine for an empty sentence", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await expect(relayTranslation(access, { sentence: "   ", selection: null, language: "en" })).resolves.toEqual({
      kind: "unavailable",
    });
    expect(seen).toEqual([]);
  });
});

describe("relayWarm (add-lingua-translation-android D2, D3)", () => {
  const warmEngine = (loaded: boolean | Error) => ({
    warm: vi.fn(async () => {
      if (loaded instanceof Error) throw loaded;
      return loaded;
    }),
  });

  it("loads nothing when no model is ready — not even to find the model missing", async () => {
    const engine = warmEngine(true);
    const onFailed = vi.fn();
    await expect(relayWarm(async () => false, engine, "en", onFailed)).resolves.toBe(false);
    expect(engine.warm).not.toHaveBeenCalled();
    expect(onFailed).not.toHaveBeenCalled();
  });

  it("warms the engine when the model is ready", async () => {
    const engine = warmEngine(true);
    await expect(relayWarm(async () => true, engine, "en")).resolves.toBe(true);
    expect(engine.warm).toHaveBeenCalledOnce();
  });

  it("asks whether the warm's language is ready, and warms that language's route", async () => {
    const engine = warmEngine(true);
    const ready = vi.fn(async (language: string) => language === "en");
    await expect(relayWarm(ready, engine, "es")).resolves.toBe(false);
    expect(engine.warm).not.toHaveBeenCalled();
    await expect(relayWarm(ready, engine, "en")).resolves.toBe(true);
    expect(engine.warm).toHaveBeenCalledWith("en");
  });

  it("reports a ready model the engine could not load, and a warm that threw", async () => {
    const onFailed = vi.fn();
    const log = vi.fn();
    await expect(relayWarm(async () => true, warmEngine(false), "en", onFailed, log)).resolves.toBe(false);
    await expect(relayWarm(async () => true, warmEngine(new Error("gone")), "en", onFailed, log)).resolves.toBe(false);
    expect(onFailed).toHaveBeenCalledTimes(2);
    expect(log).toHaveBeenCalledOnce();
  });

  it("an unreadable setting counts as not ready", async () => {
    const engine = warmEngine(true);
    await expect(relayWarm(async () => Promise.reject(new Error("storage")), engine, "en")).resolves.toBe(false);
    expect(engine.warm).not.toHaveBeenCalled();
  });
});
