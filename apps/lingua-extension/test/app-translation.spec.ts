import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import { SHIPPED_PAIRS, studiedOf } from "@/analyzer/pairs.ts";
import type { StudiedLanguage } from "@/analyzer/types.ts";
import {
  APP_MODELS_KEY,
  type AppModelView,
  type AppTranslationDeps,
  appPairs,
  openAppTranslation,
} from "@/app/translation.ts";
import type { DownloadEvent } from "@/translate/host/downloads.ts";
import { type ModelCatalogue, type ModelManifest, modelsFor, parseCatalogue } from "@/translate/host/model-manifest.ts";

// « Traduction étendue » in the Cymbra Lingua app (add-lingua-review-translation-fallback D7): the
// app's own copy of the models, offered once with its size, nothing downloaded before the reader
// accepts, deleted when the extension's setting is off, offered again once evicted, and a download
// that fails its checksum recorded as nothing. The workers are test doubles: the engine's channel and
// the download host are measured in their own specs.

const file = (size: number) => ({ path: "x", size, unpacked: size * 2, sha256: "s" });
const model = (size: number) => ({ from: "en", to: "fr", files: { model: file(size), lex: file(0), vocab: file(0) } });
const CATALOGUE: ModelCatalogue = {
  base: "https://models.cymbra.app/",
  models: { "en-fr/m": model(26_000_000), "es-en/m": model(20_000_000), "en-fr/x": model(1) },
  routes: { "en-fr": ["en-fr/m"], "es-fr": ["es-en/m", "en-fr/m"] },
};

function storage(seed: Record<string, string> = {}) {
  const items = { ...seed };
  return {
    items,
    getItem: (key: string) => items[key] ?? null,
    setItem: (key: string, value: string) => void (items[key] = value),
  };
}

function setup(over: Partial<AppTranslationDeps> & { complete?: string[] } = {}) {
  const complete = new Set(over.complete ?? []);
  const downloads: { models: string[]; report: (event: DownloadEvent) => void }[] = [];
  const views: AppModelView[] = [];
  const erase = vi.fn(async () => {});
  const engine = {
    translate: vi.fn(async () => ({ ok: true as const, html: "Il <b>se relâche</b>." })),
    warm: vi.fn(async () => true),
  };
  const local = storage();
  const deps: AppTranslationDeps = {
    enabled: true,
    pairs: async () => ["en-fr"],
    native: async () => "fr",
    catalogue: async () => CATALOGUE,
    db: { complete: async (m: ModelManifest) => complete.has(m.version), erase },
    engine,
    download: (models, report) => void downloads.push({ models, report }),
    local,
    onChange: (view) => void views.push(view),
    ...over,
  };
  return { deps, complete, downloads, views, erase, engine, local };
}

describe("the app's models (add-lingua-review-translation-fallback D7)", () => {
  it("offers once what a pair of the deck lacks, with its size, and downloads nothing before the reader accepts", async () => {
    const s = setup();
    const t = await openAppTranslation(s.deps);
    expect(t.view()).toEqual({ phase: "offer", pairs: ["en-fr"], bytes: 26_000_000 });
    expect(s.downloads).toEqual([]);
    expect(t.source("en")).toBeNull();

    t.decline();
    expect(t.view()).toEqual({ phase: "none" });
    expect(JSON.parse(s.local.items[APP_MODELS_KEY]!)).toEqual({ declined: ["en-fr"], downloaded: [] });
    // The next opening does not ask again.
    expect((await openAppTranslation({ ...s.deps, local: s.local })).view()).toEqual({ phase: "none" });
  });

  it("counts only the models a route still lacks: a pivot sharing a model already there", async () => {
    const s = setup({ pairs: async () => ["en-fr", "es-fr"], complete: ["en-fr/m"] });
    expect((await openAppTranslation(s.deps)).view()).toEqual({ phase: "offer", pairs: ["es-fr"], bytes: 20_000_000 });
  });

  it("downloads on acceptance, shows its progress, and translates once the models are there", async () => {
    const s = setup();
    const t = await openAppTranslation(s.deps);
    t.accept();
    expect(s.downloads.map((d) => d.models)).toEqual([["en-fr/m"]]);
    s.downloads[0]!.report({ kind: "progress", received: 13_000_000, total: 26_000_000 });
    expect(t.view()).toEqual({ phase: "downloading", received: 13_000_000, total: 26_000_000 });
    s.complete.add("en-fr/m");
    s.downloads[0]!.report({ kind: "done" });
    await vi.waitFor(() => expect(t.view()).toEqual({ phase: "none" }));

    const translator = t.source("en");
    expect(translator).not.toBeNull();
    expect(t.source("es")).toBeNull();
    const result = await translator!.translate({
      sentence: "It slackens.",
      selection: { start: 3, end: 11 },
      language: "en",
    });
    expect(result.kind).toBe("translated");
    expect(s.engine.translate).toHaveBeenCalledWith(expect.stringContaining("<b>slackens</b>"), "en-fr");
    translator!.warm?.("en");
    await vi.waitFor(() => expect(s.engine.warm).toHaveBeenCalledWith("en-fr"));
    expect(JSON.parse(s.local.items[APP_MODELS_KEY]!)).toEqual({ declined: [], downloaded: ["en-fr"] });
  });

  it("records nothing of a download that fails its checksum, and offers to try again", async () => {
    const s = setup();
    const t = await openAppTranslation(s.deps);
    t.accept();
    s.downloads[0]!.report({ kind: "failed", reason: "not-the-model" });
    expect(t.view()).toEqual({ phase: "failed", reason: "not-the-model" });
    expect(s.local.items[APP_MODELS_KEY]).toBeUndefined();
    expect(t.source("en")).toBeNull();
    t.accept();
    expect(s.downloads).toHaveLength(2);
  });

  it("offers again a pair downloaded once and evicted since, even after a « Plus tard »", async () => {
    const s = setup();
    s.local.items[APP_MODELS_KEY] = JSON.stringify({ declined: ["en-fr"], downloaded: ["en-fr"] });
    expect((await openAppTranslation(s.deps)).view()).toMatchObject({ phase: "offer", pairs: ["en-fr"] });
  });

  it("reads again what is on the device when a ready route gives no answer", async () => {
    const s = setup({ complete: ["en-fr/m"] });
    const t = await openAppTranslation(s.deps);
    expect(t.view()).toEqual({ phase: "none" });
    s.complete.clear(); // evicted under the page
    s.engine.translate.mockResolvedValue({ ok: false, reason: "the model is not on this device" } as never);
    await t.source("en")!.translate({ sentence: "It slackens.", selection: null, language: "en" });
    await vi.waitFor(() => expect(t.view()).toMatchObject({ phase: "offer" }));
  });

  it("deletes the app's copy when the setting is off in the extension, and offers nothing", async () => {
    const s = setup({ enabled: false, complete: ["en-fr/m"] });
    s.local.items[APP_MODELS_KEY] = JSON.stringify({ declined: [], downloaded: ["en-fr"] });
    const t = await openAppTranslation(s.deps);
    expect(s.erase).toHaveBeenCalledOnce();
    expect(t.view()).toEqual({ phase: "none" });
    expect(t.source("en")).toBeNull();
    t.accept();
    t.decline();
    expect(s.downloads).toEqual([]);
    expect(JSON.parse(s.local.items[APP_MODELS_KEY]!)).toEqual({ declined: [], downloaded: [] });
  });

  it("offers nothing for a pair the catalogue has no route for, nor when the models cannot be read", async () => {
    const cases: Partial<AppTranslationDeps>[] = [
      { pairs: async () => ["fr-de"] },
      { catalogue: async () => Promise.reject(new Error("offline")) },
    ];
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    for (const over of cases) {
      const t = await openAppTranslation(setup(over).deps);
      expect(t.view()).toEqual({ phase: "none" });
      expect(t.source("en")).toBeNull();
    }
    warn.mockRestore();
  });

  it("reads a remembered offer it cannot parse as none, and survives a storage that refuses", async () => {
    const s = setup({
      local: {
        getItem: () => "{not json",
        setItem: () => {
          throw new Error("full");
        },
      },
    });
    const t = await openAppTranslation(s.deps);
    expect(t.view()).toMatchObject({ phase: "offer" });
    expect(() => t.decline()).not.toThrow();
    expect((await openAppTranslation({ ...s.deps, local: null })).view()).toMatchObject({ phase: "offer" });
  });
});

describe("the review's hint in the app (add-lingua-review-translation-fallback D12)", () => {
  // (case, deps, the hint for an English card)
  const HINTS: [string, Partial<AppTranslationDeps> & { complete?: string[] }, string | null][] = [
    ["« Traduction étendue » off in the extension: turn it on there", { enabled: false }, "extension"],
    ["on, the models absent: the download", {}, "download"],
    ["on, the models there: nothing to say", { complete: ["en-fr/m"] }, null],
    ["a pair without a route: nothing would help", { pairs: async () => ["fr-de"] }, null],
  ];

  it("says how the app gets a card translated, per case", async () => {
    for (const [name, over, kind] of HINTS) {
      const t = await openAppTranslation(setup(over).deps);
      expect(t.hint("en")?.kind ?? null, name).toBe(kind);
    }
  });

  it("re-offers the download after a « Plus tard », through the same offer", async () => {
    const s = setup();
    const t = await openAppTranslation(s.deps);
    t.decline();
    expect(t.view()).toEqual({ phase: "none" });
    const hint = t.hint("en");
    expect(hint?.kind).toBe("download");
    if (hint?.kind === "download") hint.download();
    await vi.waitFor(() => expect(t.view()).toEqual({ phase: "offer", pairs: ["en-fr"], bytes: 26_000_000 }));
    expect(JSON.parse(s.local.items[APP_MODELS_KEY]!)).toEqual({ declined: [], downloaded: [] });
    expect(s.downloads).toEqual([]); // offered, never fetched before the reader accepts
    t.accept();
    expect(s.downloads).toHaveLength(1);
    expect(t.hint("en")).toBeNull(); // a download running: its progress says enough
    t.reoffer(); // and nothing is re-offered under it
    expect(t.view()).toMatchObject({ phase: "downloading" });
  });

  it("the setting off: the hint points to the extension and the re-offer does nothing", async () => {
    const s = setup({ enabled: false });
    const t = await openAppTranslation(s.deps);
    t.reoffer();
    expect(t.view()).toEqual({ phase: "none" });
    expect(s.downloads).toEqual([]);
  });
});

describe("the app's pairs (add-lingua-italian-baseline, task 7.1)", () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const committed = parseCatalogue(JSON.parse(readFileSync(join(root, "model-manifest.json"), "utf8")));
  const IT_EN = "it-en/base-memory/2.0";
  const EN_IT = "en-it/base-memory/2.1";
  // A profile as the engine reports it: once the core knows Italian, a backup may name it, whatever
  // the extension's own type says.
  const port = (studied: string[], native: string) => ({
    studiedLanguages: async () => studied as StudiedLanguage[],
    nativeLanguage: async () => native as "fr" | "en" | "es",
  });

  it("are the listed pairs of the reader's native language, so over today's shipped list no Italian model is needed", async () => {
    // The catalogue routes Italian's pairs (italian-translation-models); no listed pair studies Italian.
    expect(Object.keys(committed.routes)).toEqual(expect.arrayContaining(["it-fr", "it-en", "it-es"]));
    expect(SHIPPED_PAIRS.some((pair) => studiedOf(pair) === "it")).toBe(false);
    // Studied × native would have named it-fr, whose route fetches the it-en model.
    expect(modelsFor(committed, ["it-fr"]).map((m) => m.version)).toContain(IT_EN);
    for (const native of ["fr", "en", "es"]) {
      for (const studied of [["it"], ["it", "en"], ["es", "it"], ["en", "es", "fr", "it"]]) {
        const pairs = await appPairs(port(studied, native));
        const where = `${native}: ${studied.join(",")}`;
        expect(
          pairs.every((pair) => SHIPPED_PAIRS.includes(pair)),
          where,
        ).toBe(true);
        expect(
          pairs.some((pair) => studiedOf(pair) === "it"),
          where,
        ).toBe(false);
        const needed = modelsFor(committed, pairs).map((m) => m.version);
        expect(needed, where).not.toContain(IT_EN);
        expect(needed, where).not.toContain(EN_IT);
      }
    }
    // A reader of English and Spanish keeps the pairs of before; one of Italian alone has none.
    expect(await appPairs(port(["en", "es"], "fr"))).toEqual(["en-fr", "es-fr"]);
    expect(await appPairs(port(["it"], "fr"))).toEqual([]);
  });

  it("offer a reader of Italian and English the English pair's models alone", async () => {
    const s = setup({ pairs: () => appPairs(port(["it", "en"], "fr")), catalogue: async () => committed });
    const view = (await openAppTranslation(s.deps)).view();
    expect(view).toMatchObject({ phase: "offer", pairs: ["en-fr"] });
  });
});
