import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
import { nativeOf, studiedOf } from "@/analyzer/pairs.ts";
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

describe("the committed catalogue", () => {
  it("Every reader today: en-fr is the en-fr model alone, es-fr is es-en then en-fr, and nothing else is listed (routes-by-pair D1)", () => {
    const c = parseCatalogue(committed);
    expect(Object.keys(c.models)).toEqual([EN_FR, ES_EN]);
    expect(c.routes).toEqual({ "en-fr": [EN_FR], "es-fr": [ES_EN, EN_FR] });
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

  it("has no route for a pair nothing translates yet — nor for a studied language asked alone", () => {
    expect(routeOf(parseCatalogue(committed), "de-fr")).toEqual([]);
    expect(routeOf(parseCatalogue(committed), "es-en")).toEqual([]);
    expect(routeOf(parseCatalogue(committed), "en")).toEqual([]);
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
    expect(Object.keys(c.models)).toEqual([EN_FR, ES_EN]);
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
    expect(modelsFor(parseCatalogue(committed), ["en-es"])).toEqual([]);
    expect(modelsFor(parseCatalogue(committed), ["en-fr", "de-fr"]).map((m) => m.version)).toEqual([EN_FR]);
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
