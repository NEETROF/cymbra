import { describe, expect, it, vi } from "vitest";
import type { EngineAccess, EngineReply } from "@/translate/host/engine.ts";
import { pairOf, relayTranslation, relayWarm } from "@/translate/host/relay.ts";
import { MARKED_PAIRS } from "@/translate/markup.ts";

/** An engine that records the markup it is handed, and through which pair, and answers what the test says. */
function engine(answer: (markup: string) => EngineReply | Promise<EngineReply>) {
  const seen: string[] = [];
  const pairs: string[] = [];
  const access: Pick<EngineAccess, "translate"> = {
    translate: async (markup, pair) => {
      seen.push(markup);
      pairs.push(pair);
      return answer(markup);
    },
  };
  return { access, seen, pairs };
}

const sentence = "She gave up after the third attempt.";
const selection = { start: 4, end: 11 }; // "gave up"

describe("the pair of a translation (generalise-lingua-translation-routes-by-pair D2)", () => {
  it("is formed from the document's language the page asked in and the reader's native language", () => {
    expect(pairOf("en", "fr")).toBe("en-fr");
    expect(pairOf("es", "fr")).toBe("es-fr");
    expect(pairOf("en", "es")).toBe("en-es");
  });
});

describe("relayTranslation", () => {
  it("asks both translations through the pair it is given (model-state D5, routes-by-pair D2)", async () => {
    const { access, pairs } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence, selection, language: "en" }, "en-fr");
    expect(pairs).toEqual(["en-fr", "en-fr"]);
  });

  it("Every reader today: a reader of French on an English page goes through en-fr, marked as before", async () => {
    const { access, seen, pairs } = engine((markup) => ({
      ok: true,
      html: markup === "gave up" ? "A abandonné" : "Elle <b>a abandonné</b> après la troisième tentative.",
    }));
    const request = { sentence, selection, language: "en" }; // the page asks in the document's language
    const result = await relayTranslation(access, request, pairOf(request.language, "fr"));
    expect(pairs).toEqual(["en-fr", "en-fr"]);
    expect(seen).toEqual(["She <b>gave up</b> after the third attempt.", "gave up"]);
    if (result.kind !== "translated") throw new Error("not translated");
    const { sentence: fr, marks } = result.translation;
    expect(marks.map((m) => fr.slice(m.start, m.end))).toEqual(["a abandonné"]);
  });

  it("The same page for another native language: the same request goes through en-es, and nothing of en-fr is asked", async () => {
    const { access, seen, pairs } = engine(() => ({ ok: true, html: "Se rindió tras el tercer intento." }));
    const request = { sentence, selection, language: "en" }; // the same wire message as above
    const result = await relayTranslation(access, request, pairOf(request.language, "es"));
    expect(pairs).toEqual(["en-es"]);
    expect(pairs).not.toContain("en-fr");
    // en-es's marks are not measured: the sentence goes untagged, once, and comes back without a mark.
    expect(seen).toEqual([sentence]);
    expect(result).toEqual({
      kind: "translated",
      translation: { sentence: "Se rindió tras el tercer intento.", marks: [] },
    });
  });

  it("marks a Spanish selection through es-fr, its marks measured (release-lingua-spanish-translation)", async () => {
    const { access, seen, pairs } = engine((markup) => ({
      ok: true,
      html: markup === "casa" ? "maison" : "Ma grand-mère vivait dans une petite <b>maison</b> près de la mer.",
    }));
    const result = await relayTranslation(
      access,
      {
        sentence: "Mi abuela vivía en una casa pequeña cerca del mar.",
        selection: { start: 23, end: 27 },
        language: "es",
      },
      "es-fr",
    );
    expect(seen).toEqual(["Mi abuela vivía en una <b>casa</b> pequeña cerca del mar.", "casa"]);
    expect(pairs).toEqual(["es-fr", "es-fr"]);
    if (result.kind !== "translated") throw new Error("not translated");
    const { sentence: fr, marks } = result.translation;
    expect(marks.map((m) => fr.slice(m.start, m.end))).toEqual(["maison"]);
  });

  it("The shipped pairs today: en-fr and es-fr are the pairs whose selection is marked (routes-by-pair D4)", () => {
    expect(MARKED_PAIRS).toEqual(["en-fr", "es-fr"]);
    // Keyed by pair, never by studied language: a language alone is not in the list.
    expect(MARKED_PAIRS).not.toContain("en");
    expect(MARKED_PAIRS).not.toContain("es");
  });

  it("A pair measured in another native language: es-en is translated without a mark, although es-fr's marks are measured", async () => {
    const { access, seen, pairs } = engine(() => ({
      ok: true,
      html: "My grandmother lived in a small house by the sea.",
    }));
    const request = {
      sentence: "Mi abuela vivía en una casa pequeña cerca del mar.",
      selection: { start: 23, end: 27 },
      language: "es",
    };
    expect(MARKED_PAIRS).toContain("es-fr");
    expect(MARKED_PAIRS).not.toContain("es-en");
    const result = await relayTranslation(access, request, pairOf(request.language, "en"));
    expect(pairs).toEqual(["es-en"]);
    expect(seen).toEqual([request.sentence]); // untagged, once
    expect(result).toEqual({
      kind: "translated",
      translation: { sentence: "My grandmother lived in a small house by the sea.", marks: [] },
    });
  });

  it("translates a sentence through an unmeasured pair untagged, once, without a mark (pivot D3)", async () => {
    // add-lingua-spanish-translation-pivot: a pair whose marks are not measured is translated
    // without one — Spanish below the first tier would have stayed so. A German route would be.
    const { access, seen, pairs } = engine(() => ({
      ok: true,
      html: "Elle a abandonné après la &lt;troisième&gt; tentative.",
    }));
    const result = await relayTranslation(
      access,
      {
        sentence: "Se rindió tras el <tercer> intento.",
        selection: { start: 3, end: 10 },
        language: "de",
      },
      "de-fr",
    );

    expect(seen).toEqual(["Se rindió tras el &lt;tercer&gt; intento."]); // escaped, and no tag
    expect(pairs).toEqual(["de-fr"]);
    expect(result).toEqual({
      kind: "translated",
      translation: { sentence: "Elle a abandonné après la <troisième> tentative.", marks: [] },
    });
  });

  it("answers an unmeasured pair's failure as unavailable", async () => {
    const { access } = engine(() => ({ ok: false, reason: "the model is not on this device" }));
    const log = vi.fn();
    const result = await relayTranslation(access, { sentence, selection, language: "de" }, "de-fr", log);
    expect(result.kind).not.toBe("translated");
    expect(log).toHaveBeenCalledWith("no translation:", "the model is not on this device");

    const throwing: Pick<EngineAccess, "translate"> = {
      translate: async () => {
        throw new Error("the worker died");
      },
    };
    expect((await relayTranslation(throwing, { sentence, selection, language: "de" }, "de-fr", log)).kind).not.toBe(
      "translated",
    );
    expect(log).toHaveBeenCalledWith("translation failed:", expect.any(Error));
  });

  it("marks the selection in its sentence and reads the translation back", async () => {
    const { access, seen } = engine((markup) => ({
      ok: true,
      html: markup === "gave up" ? "A abandonné" : "Elle <b>a abandonné</b> après la troisième tentative.",
    }));
    const result = await relayTranslation(access, { sentence, selection, language: "en" }, "en-fr");

    // The sentence with the selection tagged, then the selection alone — to check the tag.
    expect(seen).toEqual(["She <b>gave up</b> after the third attempt.", "gave up"]);
    expect(result.kind).toBe("translated");
    if (result.kind !== "translated") return;
    const { sentence: fr, marks } = result.translation;
    expect(fr).toBe("Elle a abandonné après la troisième tentative.");
    expect(marks.map((m) => fr.slice(m.start, m.end))).toEqual(["a abandonné"]);
  });

  it("sends the sentence without its footnote calls, the selection moved with its text", async () => {
    const { access, seen } = engine((markup) => ({ ok: true, html: markup }));
    const text = "Rusia contaba[7][8] con 23 millones de gatos.[9]";
    const start = text.indexOf("millones");
    await relayTranslation(access, { sentence: text, selection: { start, end: start + 8 }, language: "es" }, "es-fr");
    expect(seen).toEqual(["Rusia contaba con 23 <b>millones</b> de gatos.", "millones"]);
  });

  it("never turns page text into markup, there and back (the card then renders text nodes only)", async () => {
    // An engine that echoes what it is given: the page's tags come back as the text they were.
    const echo = engine((markup) => ({ ok: true, html: markup }));
    const page = 'Usa <img src=x onerror="alert(1)"> y <script>alert(2)</script> aquí.';
    const start = page.indexOf("aquí");
    const result = await relayTranslation(
      echo.access,
      { sentence: page, selection: { start, end: start + 4 }, language: "es" },
      "es-fr",
    );
    expect(echo.seen[0]).not.toMatch(/<img|<script/); // escaped before the engine
    if (result.kind !== "translated") throw new Error("not translated");
    expect(result.translation.sentence).toBe(page); // back as plain text, the tags as characters
    expect(result.translation.marks.map((m) => page.slice(m.start, m.end))).toEqual(["aquí"]);

    // An engine that answered raw tags of its own: none survives as markup, only our mark is read.
    const raw = engine(() => ({ ok: true, html: 'Utilisez <img src=x onerror="alert(1)"><b>ici</b>.' }));
    const read = await relayTranslation(
      raw.access,
      { sentence: page, selection: { start, end: start + 4 }, language: "en" },
      "en-fr",
    );
    if (read.kind !== "translated") throw new Error("not translated");
    expect(read.translation.sentence).toBe("Utilisez ici.");
  });

  it("escapes page text before it reaches the engine", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence: "a <b> b", selection: { start: 0, end: 1 }, language: "en" }, "en-fr");
    expect(seen).toEqual(["<b>a</b> &lt;b&gt; b", "a"]);
  });

  it("escapes the selection it sends alone, too", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await relayTranslation(access, { sentence: "a <b> b", selection: { start: 2, end: 5 }, language: "en" }, "en-fr");
    expect(seen).toEqual(["a <b>&lt;b&gt;</b> b", "&lt;b&gt;"]);
  });

  it("translates the sentence unmarked when the selection's place is unknown", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "Elle a abandonné." }));
    const result = await relayTranslation(
      access,
      { sentence: "She gave up.", selection: null, language: "en" },
      "en-fr",
    );
    expect(seen).toEqual(["She gave up."]);
    expect(result).toEqual({ kind: "translated", translation: { sentence: "Elle a abandonné.", marks: [] } });
  });

  describe("the check against the selection alone", () => {
    const friday = "They seldom ship on Friday, even when the customer asks nicely.";
    const seldom = { start: 5, end: 11 };
    const tagged = "Ils <b>expédient</b> rarement le vendredi, même lorsque le client demande bien.";

    const marked = async (alone: (markup: string) => EngineReply | Promise<EngineReply>) => {
      const { access } = engine((markup) => (markup === "seldom" ? alone(markup) : { ok: true, html: tagged }));
      const result = await relayTranslation(access, { sentence: friday, selection: seldom, language: "en" }, "en-fr");
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
    await expect(relayTranslation(access, { sentence, selection, language: "en" }, "en-fr", log)).resolves.toEqual({
      kind: "unavailable",
    });
    expect(log).toHaveBeenCalledWith("no translation:", "the engine did not start");
  });

  it("answers unavailable when reaching the engine throws", async () => {
    const log = vi.fn();
    const access: Pick<EngineAccess, "translate"> = { translate: () => Promise.reject(new Error("gone")) };
    await expect(relayTranslation(access, { sentence, selection, language: "en" }, "en-fr", log)).resolves.toEqual({
      kind: "unavailable",
    });
    expect(log).toHaveBeenCalledOnce();
  });

  it("does not wake the engine for an empty sentence", async () => {
    const { access, seen } = engine(() => ({ ok: true, html: "x" }));
    await expect(
      relayTranslation(access, { sentence: "   ", selection: null, language: "en" }, "en-fr"),
    ).resolves.toEqual({ kind: "unavailable" });
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
    await expect(relayWarm(async () => false, engine, "en-fr", onFailed)).resolves.toBe(false);
    expect(engine.warm).not.toHaveBeenCalled();
    expect(onFailed).not.toHaveBeenCalled();
  });

  it("warms the engine when the model is ready", async () => {
    const engine = warmEngine(true);
    await expect(relayWarm(async () => true, engine, "en-fr")).resolves.toBe(true);
    expect(engine.warm).toHaveBeenCalledOnce();
  });

  it("asks whether the warm's pair is ready, and warms that pair's route", async () => {
    const engine = warmEngine(true);
    const ready = vi.fn(async (pair: string) => pair === "en-fr");
    await expect(relayWarm(ready, engine, "es-fr")).resolves.toBe(false);
    expect(engine.warm).not.toHaveBeenCalled();
    await expect(relayWarm(ready, engine, "en-fr")).resolves.toBe(true);
    expect(engine.warm).toHaveBeenCalledWith("en-fr");
  });

  it("A pair without a route: a reader of Spanish on an English page with no en-es recorded — the engine is not started", async () => {
    // The device records only the pairs whose whole route is on it (D3); a pair the catalogue has no
    // route for is never among them, so the gate answers false before the engine's host is reached.
    const engine = warmEngine(true);
    const recorded = ["en-fr"];
    const ready = vi.fn(async (pair: string) => recorded.includes(pair));
    await expect(relayWarm(ready, engine, pairOf("en", "es"))).resolves.toBe(false);
    expect(ready).toHaveBeenCalledWith("en-es");
    expect(engine.warm).not.toHaveBeenCalled();
  });

  it("reports a ready model the engine could not load, and a warm that threw", async () => {
    const onFailed = vi.fn();
    const log = vi.fn();
    await expect(relayWarm(async () => true, warmEngine(false), "en-fr", onFailed, log)).resolves.toBe(false);
    await expect(relayWarm(async () => true, warmEngine(new Error("gone")), "en-fr", onFailed, log)).resolves.toBe(
      false,
    );
    expect(onFailed).toHaveBeenCalledTimes(2);
    expect(log).toHaveBeenCalledOnce();
  });

  it("an unreadable setting counts as not ready", async () => {
    const engine = warmEngine(true);
    await expect(relayWarm(async () => Promise.reject(new Error("storage")), engine, "en-fr")).resolves.toBe(false);
    expect(engine.warm).not.toHaveBeenCalled();
  });
});
