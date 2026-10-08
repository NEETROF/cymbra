import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import { nativeOf, readerPairs, SHIPPED_PAIRS, studiedOf } from "@/analyzer/pairs.ts";
import {
  fileUrl,
  loadBundledCatalogue,
  MANIFEST_PATH,
  type ModelCatalogue,
  modelsById,
  modelsFor,
  parseCatalogue,
  routeOf,
  totalSize,
  unpackedSize,
} from "@/translate/host/model-manifest.ts";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const committed = JSON.parse(readFileSync(join(root, "model-manifest.json"), "utf8"));

// The sha256 of each file's DECOMPRESSED bytes, as TRANSLATION.md pinned them when the engine was
// measured (add-lingua-translation-engine). The download is checked against these, nothing else.
const PINNED = {
  model: "6322e296d4fecfe395a8d5723da4ec37ecbe6d7613bb1dfcf4b28e2a47498b68",
  lex: "2585ed98d3af0bc949865aedeb390493d591f56870814376e73e4144c41ed059",
  vocab: "783abf3abe075afdf8d85d233994bef2c3a064e935ab1bed946820aff6ac002a",
};

const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";
const EN_ES = "en-es/base-memory/2.1";

// en-es 2.1 as Mozilla publishes it (add-lingua-translation-matrix-models D1): the sha256 of each
// file's decompressed bytes, read from the files downloaded from the registry and equal to Firefox's
// Remote Settings for en→es 2.1; the model's is the registry's `uncompressedHash`.
const EN_ES_PINNED = {
  model: "3b1c399511c01c84c36fae5c0524df44096288efdc8236e182b5c97d7ad2244c",
  lex: "7d51237c0a07027dcd61643cfbbb0f8c48597d19907ef53d2cae9d6bec2cf25c",
  vocab: "5ae254fa9b15aa182e70fd2a6186b1333c63a29a48043a9224c6aa4fcac058ad",
};

describe("the committed catalogue", () => {
  it("Every reader today: three models, en-fr's and es-fr's routes as before, es-en and en-es one model each, and nothing else is listed (matrix-models D2)", () => {
    const c = parseCatalogue(committed);
    // In the order they were pinned.
    expect(Object.keys(c.models)).toEqual([EN_FR, ES_EN, EN_ES]);
    expect(c.routes).toEqual({
      "en-fr": [EN_FR],
      "es-fr": [ES_EN, EN_FR],
      "es-en": [ES_EN],
      "en-es": [EN_ES],
    });
    // The same models in the same order as the routes keyed by studied language gave.
    expect(routeOf(c, "en-fr").map((m) => m.version)).toEqual([EN_FR]);
    expect(routeOf(c, "es-fr").map((m) => m.version)).toEqual([ES_EN, EN_FR]);
  });

  it("routes English through the en→fr base-memory model the engine was measured on", () => {
    const [model, ...rest] = routeOf(parseCatalogue(committed), "en-fr");
    expect(rest).toEqual([]);
    // The id is the version a device recorded before the catalogue: a stored model stays complete.
    expect(model.version).toBe(EN_FR);
    expect([model.from, model.to]).toEqual(["en", "fr"]);
    expect(Object.fromEntries(Object.entries(model.files).map(([role, f]) => [role, f.sha256]))).toEqual(PINNED);
  });

  it("downloads 25 752 472 bytes — Mozilla's own gzip files — and keeps 36 749 127 on the device", () => {
    const [model] = routeOf(parseCatalogue(committed), "en-fr");
    expect(totalSize(model)).toBe(25_752_472);
    expect(unpackedSize(model)).toBe(36_749_127);
  });

  it("serves each file under a content-addressed path, from Cymbra over https", () => {
    const [model] = routeOf(parseCatalogue(committed), "en-fr");
    expect(model.base).toMatch(/^https:\/\/[a-z.]+cymbra\.app\/$/);
    for (const file of Object.values(model.files)) {
      expect(file.path).toContain(`/${file.sha256}/`);
      expect(file.path).toMatch(/\.gz$/);
      expect(fileUrl(model, file)).toBe(`${model.base}${file.path}`);
    }
  });

  it("routes es-fr through English: es-en, then en-fr (add-lingua-spanish-translation-pivot D1)", () => {
    const route = routeOf(parseCatalogue(committed), "es-fr");
    expect(route.map((m) => [m.version, m.from, m.to])).toEqual([
      [ES_EN, "es", "en"],
      [EN_FR, "en", "fr"],
    ]);
    // Pinned as Mozilla's registry lists it: the decompressed model's sha256 is its uncompressedHash.
    expect(route[0].files.model.sha256).toBe("4aed7734152ae0045d1a69ae49c86cfda18f53c61f90e95e1d1de1c7c7c3b033");
    expect(totalSize(route[0])).toBe(26_241_052);
    expect(totalSize(route[0]) + totalSize(route[1])).toBe(51_993_524);
  });

  it("translates English and Spanish into each other directly: es-en is the es-en model alone, en-es the en-es model alone (matrix-models D2)", () => {
    const c = parseCatalogue(committed);
    expect(routeOf(c, "es-en").map((m) => [m.version, m.from, m.to])).toEqual([[ES_EN, "es", "en"]]);
    expect(routeOf(c, "en-es").map((m) => [m.version, m.from, m.to])).toEqual([[EN_ES, "en", "es"]]);
    // es-en's route alone: its own model, nothing of en-fr.
    expect(totalSize(routeOf(c, "es-en")[0]!)).toBe(26_241_052);
  });

  it("Pinned against Mozilla's publications: en-es 2.1's files, their digests, and 25 373 354 bytes to download (matrix-models D1)", () => {
    const [model, ...rest] = routeOf(parseCatalogue(committed), "en-es");
    expect(rest).toEqual([]);
    expect(Object.fromEntries(Object.entries(model!.files).map(([role, f]) => [role, f.sha256]))).toEqual(EN_ES_PINNED);
    expect(Object.fromEntries(Object.entries(model!.files).map(([role, f]) => [role, [f.size, f.unpacked]]))).toEqual({
      model: [22_698_792, 31_561_787],
      lex: [2_265_250, 4_198_436],
      vocab: [409_312, 816_054],
    });
    expect(totalSize(model!)).toBe(25_373_354);
    expect(unpackedSize(model!)).toBe(36_576_277);
    // Named as the other models' files, under the id and the decompressed sha256.
    expect(Object.values(model!.files).map((f) => f.path)).toEqual([
      `${EN_ES}/${EN_ES_PINNED.model}/model.bin.gz`,
      `${EN_ES}/${EN_ES_PINNED.lex}/lex.bin.gz`,
      `${EN_ES}/${EN_ES_PINNED.vocab}/vocab.spm.gz`,
    ]);
    // Where Mozilla's registry publishes them, with its mirror release and its licence.
    const raw = committed.models[EN_ES];
    expect(raw.licence).toBe("MPL-2.0");
    expect(raw.mirror).toBe("https://github.com/NEETROF/cymbra/releases/download/lingua-model-en-es-base-memory-2.1/");
    expect(Object.values(raw.files).map((f) => (f as { source: { path: string } }).source.path)).toEqual([
      "models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/model.enes.intgemm.alphas.bin.gz",
      "models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/lex.50.50.enes.s2t.bin.gz",
      "models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/vocab.enes.spm.gz",
    ]);
  });

  it("shares a file between es-en and en-es: the same vocabulary once decompressed, from two gzip files (matrix-models D3)", () => {
    const c = parseCatalogue(committed);
    const [esEn] = routeOf(c, "es-en");
    const [enEs] = routeOf(c, "en-es");
    // The device stores a file by its decompressed sha256: one held for es-en is not fetched again for en-es.
    expect(enEs!.files.vocab.sha256).toBe(esEn!.files.vocab.sha256);
    expect(committed.models[EN_ES].files.vocab.source.sha256).not.toBe(
      committed.models[ES_EN].files.vocab.source.sha256,
    );
    // The cost sums the files of the models needed, the shared one in each (the setting's cost requirement).
    expect(modelsFor(c, ["es-en", "en-es"]).reduce((sum, m) => sum + totalSize(m), 0)).toBe(26_241_052 + 25_373_354);
  });

  it("has no route for a pair nothing translates yet — nor for a studied language asked alone", () => {
    expect(routeOf(parseCatalogue(committed), "de-fr")).toEqual([]);
    expect(routeOf(parseCatalogue(committed), "fr-en")).toEqual([]);
    expect(routeOf(parseCatalogue(committed), "fr-es")).toEqual([]);
    expect(routeOf(parseCatalogue(committed), "en")).toEqual([]);
  });

  it("A route of a pair not shipped: no reader's pairs need es-en or en-es, and the es-en model is held only as es-fr's first model", () => {
    const c = parseCatalogue(committed);
    expect(SHIPPED_PAIRS).toEqual(["en-fr", "es-fr"]);
    // A reader of French: the needs, the downloads and the routes loaded are those of before.
    expect(readerPairs(["en", "es"], "fr")).toEqual(["en-fr", "es-fr"]);
    expect(modelsFor(c, readerPairs(["en", "es"], "fr")).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(c, readerPairs(["en"], "fr")).map((m) => m.version)).toEqual([EN_FR]);
    expect(modelsFor(c, readerPairs(["es"], "fr")).map((m) => m.version)).toEqual([ES_EN, EN_FR]);
    // A reader of another native language has no shipped pair: nothing is needed for them.
    expect(readerPairs(["en"], "es")).toEqual([]);
    expect(readerPairs(["es"], "en")).toEqual([]);
    expect(modelsFor(c, readerPairs(["en"], "es"))).toEqual([]);
    expect(modelsFor(c, readerPairs(["es"], "en"))).toEqual([]);
    // Whatever a reader studies, en-es is never needed, and es-en only through es-fr.
    for (const native of ["fr", "en", "es"]) {
      for (const languages of [["en"], ["es"], ["en", "es"], ["es", "en"]]) {
        const pairs = readerPairs(languages, native);
        expect(pairs.every((pair) => pair === "en-fr" || pair === "es-fr")).toBe(true);
        const needed = modelsFor(c, pairs).map((m) => m.version);
        expect(needed).not.toContain(EN_ES);
        if (needed.includes(ES_EN)) expect(pairs).toContain("es-fr");
      }
    }
  });
});

describe("parseCatalogue", () => {
  const valid = (): ModelCatalogue => parseCatalogue(committed);
  /** The committed catalogue with one model more, and `routes` as given. */
  const withRoutes = (routes: Record<string, string[]>) => {
    const c = valid();
    const esEn = { ...c.models[EN_FR]!, from: "es", to: "en" };
    return { ...c, models: { ...c.models, "es-en/base-memory/2.0": esEn }, routes };
  };

  it("refuses a catalogue without a base, models or routes", () => {
    expect(() => parseCatalogue(null)).toThrow(/required/);
    expect(() => parseCatalogue({ ...valid(), base: 2 })).toThrow(/required/);
    expect(() => parseCatalogue({ ...valid(), models: undefined })).toThrow(/required/);
    expect(() => parseCatalogue({ ...valid(), routes: undefined })).toThrow(/required/);
  });

  it("refuses a base that is not an http(s) address", () => {
    expect(() => parseCatalogue({ ...valid(), base: "file:///tmp/" })).toThrow(/base/);
  });

  it("refuses a file without its pins", () => {
    const c = valid();
    const model = c.models[EN_FR]!;
    const files = (over: object) => ({ ...c, models: { [EN_FR]: { ...model, files: { ...model.files, ...over } } } });
    expect(() => parseCatalogue(files({ lex: { ...model.files.lex, sha256: "abc" } }))).toThrow(/lex/);
    expect(() => parseCatalogue(files({ lex: { ...model.files.lex, unpacked: undefined } }))).toThrow(/unpacked/);
    expect(() => parseCatalogue(files({ vocab: undefined }))).toThrow(/vocab/);
  });

  it("refuses a model without its languages", () => {
    const c = valid();
    const model = { ...c.models[EN_FR]!, to: undefined };
    expect(() => parseCatalogue({ ...c, models: { [EN_FR]: model } })).toThrow(/languages/);
  });

  it("refuses a route that names a model the catalogue does not list", () => {
    expect(() => parseCatalogue(withRoutes({ "en-fr": ["en-fr/tiny/1.0"] }))).toThrow(/not in the catalogue/);
  });

  it("A route that does not start from its studied language: en-es starting with the es→en model is refused", () => {
    expect(() => parseCatalogue(withRoutes({ "en-es": [ES_EN] }))).toThrow(/translates from es, with en/);
    expect(() => parseCatalogue(withRoutes({ "es-fr": [EN_FR] }))).toThrow(/translates from en, with es/);
  });

  it("refuses a route that breaks the chain", () => {
    expect(() => parseCatalogue(withRoutes({ "es-fr": [ES_EN, ES_EN] }))).toThrow(/translates from es, with en/);
  });

  it("A route that does not reach French: es-fr ending in English is refused, and nothing is fetched", () => {
    expect(() => parseCatalogue(withRoutes({ "es-fr": [ES_EN] }))).toThrow(/ends in en, not fr/);
    expect(() => parseCatalogue(withRoutes({ "es-fr": [] }))).toThrow(/names no model/);
  });

  it("A route that does not reach its native language: en-es ending in French is refused", () => {
    // The parser no longer knows French: the native language is read from the key (routes-by-pair D1).
    expect(() => parseCatalogue(withRoutes({ "en-es": [EN_FR] }))).toThrow(/ends in fr, not es/);
  });

  it("A route whose key is no pair: `en`, `en-` and `-fr` are refused", () => {
    expect(() => parseCatalogue(withRoutes({ en: [EN_FR] }))).toThrow(/en is no pair/);
    expect(() => parseCatalogue(withRoutes({ "en-": [EN_FR] }))).toThrow(/en- is no pair/);
    expect(() => parseCatalogue(withRoutes({ "-fr": [EN_FR] }))).toThrow(/-fr is no pair/);
  });

  it("splits a key on its first `-`, as a pack's name is: `en-fr-x` is the pair en / fr-x, which no route ends in", () => {
    expect(() => parseCatalogue(withRoutes({ "en-fr-x": [EN_FR] }))).toThrow(/ends in fr, not fr-x/);
    // The same reading as analyzer/pairs.ts's: a model into `fr-x` would make the route whole.
    expect(nativeOf("en-fr-x")).toBe("fr-x");
    expect(studiedOf("en-fr-x")).toBe("en");
    const base = withRoutes({ "en-fr-x": ["en-frx/base-memory/2.0"] });
    const models: ModelCatalogue["models"] = base.models;
    const raw = {
      ...base,
      models: { ...models, "en-frx/base-memory/2.0": { ...models[EN_FR]!, from: "en", to: "fr-x" } },
    };
    expect(routeOf(parseCatalogue(raw), "en-fr-x").map((m) => m.to)).toEqual(["fr-x"]);
  });

  it("accepts a route through English, in order", () => {
    const c = parseCatalogue(withRoutes({ "en-fr": [EN_FR], "es-fr": [ES_EN, EN_FR] }));
    expect(routeOf(c, "es-fr").map((m) => m.version)).toEqual([ES_EN, EN_FR]);
  });

  it("accepts a pair glossed in another native language, with its own route", () => {
    const c = parseCatalogue(withRoutes({ "en-fr": [EN_FR], "es-en": [ES_EN] }));
    expect(routeOf(c, "es-en").map((m) => m.version)).toEqual([ES_EN]);
  });

  it("keeps only what the runtime needs", () => {
    const c = parseCatalogue({ ...committed, extra: true });
    expect(Object.keys(c).sort()).toEqual(["base", "models", "routes"]);
    expect(Object.keys(c.models[EN_FR]!).sort()).toEqual(["files", "from", "to"]);
    expect(Object.keys(c.models[EN_FR]!.files.model).sort()).toEqual(["path", "sha256", "size", "unpacked"]);
  });
});

describe("loading the package's catalogue", () => {
  it("reads the package's own copy — a file of the extension", async () => {
    const fetchFn = vi.fn(async () => new Response(JSON.stringify(committed)));
    const c = await loadBundledCatalogue(fetchFn as unknown as typeof fetch);
    expect(fetchFn).toHaveBeenCalledWith(MANIFEST_PATH);
    expect(Object.keys(c.models)).toEqual([EN_FR, ES_EN, EN_ES]);
  });

  it("fails on a missing file rather than guess", async () => {
    const fetchFn = vi.fn(async () => new Response("", { status: 404 }));
    await expect(loadBundledCatalogue(fetchFn as unknown as typeof fetch)).rejects.toThrow(/404/);
  });
});

describe("the models a device needs (generalise-lingua-translation-model-state D2)", () => {
  /** The committed catalogue with es-en and a Spanish route through English. */
  const pivot = () =>
    parseCatalogue({
      ...committed,
      models: { ...committed.models, "es-en/base-memory/2.0": { ...committed.models[EN_FR], from: "es", to: "en" } },
      routes: { ...committed.routes, "es-fr": ["es-en/base-memory/2.0", EN_FR] },
    });

  it("is the union of the pairs' routes, each model once, in order (routes-by-pair D5)", () => {
    expect(modelsFor(pivot(), ["en-fr", "es-fr"]).map((m) => m.version)).toEqual([EN_FR, "es-en/base-memory/2.0"]);
    expect(modelsFor(pivot(), ["es-fr", "en-fr"]).map((m) => m.version)).toEqual(["es-en/base-memory/2.0", EN_FR]);
  });

  it("needs nothing for a pair without a route", () => {
    expect(modelsFor(parseCatalogue(committed), ["de-fr"])).toEqual([]);
    expect(modelsFor(parseCatalogue(committed), ["fr-es"])).toEqual([]);
    expect(modelsFor(parseCatalogue(committed), ["en-fr", "de-fr"]).map((m) => m.version)).toEqual([EN_FR]);
  });

  it("needs the one model of es-en's and en-es's routes, for the pairs that will ship them (changes 34, 35)", () => {
    expect(modelsFor(parseCatalogue(committed), ["es-en"]).map((m) => m.version)).toEqual([ES_EN]);
    expect(modelsFor(parseCatalogue(committed), ["en-es"]).map((m) => m.version)).toEqual([EN_ES]);
  });

  it("needs both models of the committed es-fr route, en-fr once", () => {
    expect(modelsFor(parseCatalogue(committed), ["en-fr", "es-fr"]).map((m) => m.version)).toEqual([EN_FR, ES_EN]);
    expect(modelsFor(parseCatalogue(committed), ["es-fr"]).map((m) => m.version)).toEqual([ES_EN, EN_FR]);
  });

  it("finds models by id, leaving out an id the catalogue does not list", () => {
    expect(modelsById(pivot(), ["es-en/base-memory/2.0", "de-en/tiny/1.0"]).map((m) => [m.version, m.from])).toEqual([
      ["es-en/base-memory/2.0", "es"],
    ]);
  });
});
