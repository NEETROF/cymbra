import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";
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

describe("the committed catalogue", () => {
  it("routes English through the en→fr base-memory model the engine was measured on", () => {
    const [model, ...rest] = routeOf(parseCatalogue(committed), "en");
    expect(rest).toEqual([]);
    // The id is the version a device recorded before the catalogue: a stored model stays complete.
    expect(model.version).toBe(EN_FR);
    expect([model.from, model.to]).toEqual(["en", "fr"]);
    expect(Object.fromEntries(Object.entries(model.files).map(([role, f]) => [role, f.sha256]))).toEqual(PINNED);
  });

  it("downloads 25 752 472 bytes — Mozilla's own gzip files — and keeps 36 749 127 on the device", () => {
    const [model] = routeOf(parseCatalogue(committed), "en");
    expect(totalSize(model)).toBe(25_752_472);
    expect(unpackedSize(model)).toBe(36_749_127);
  });

  it("serves each file under a content-addressed path, from Cymbra over https", () => {
    const [model] = routeOf(parseCatalogue(committed), "en");
    expect(model.base).toMatch(/^https:\/\/[a-z.]+cymbra\.app\/$/);
    for (const file of Object.values(model.files)) {
      expect(file.path).toContain(`/${file.sha256}/`);
      expect(file.path).toMatch(/\.gz$/);
      expect(fileUrl(model, file)).toBe(`${model.base}${file.path}`);
    }
  });

  it("has no route for a language nothing translates yet", () => {
    expect(routeOf(parseCatalogue(committed), "es")).toEqual([]);
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
    expect(() => parseCatalogue(withRoutes({ en: ["en-fr/tiny/1.0"] }))).toThrow(/not in the catalogue/);
  });

  it("refuses a route that does not start from its language", () => {
    expect(() => parseCatalogue(withRoutes({ es: [EN_FR] }))).toThrow(/translates from en, with es/);
  });

  it("refuses a route that breaks the chain", () => {
    expect(() => parseCatalogue(withRoutes({ es: ["es-en/base-memory/2.0", "es-en/base-memory/2.0"] }))).toThrow(
      /translates from es, with en/,
    );
  });

  it("refuses a route that does not end in French", () => {
    expect(() => parseCatalogue(withRoutes({ es: ["es-en/base-memory/2.0"] }))).toThrow(/ends in en/);
    expect(() => parseCatalogue(withRoutes({ es: [] }))).toThrow(/names no model/);
  });

  it("accepts a route through English, in order", () => {
    const c = parseCatalogue(withRoutes({ en: [EN_FR], es: ["es-en/base-memory/2.0", EN_FR] }));
    expect(routeOf(c, "es").map((m) => m.version)).toEqual(["es-en/base-memory/2.0", EN_FR]);
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
    expect(Object.keys(c.models)).toEqual([EN_FR]);
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
      routes: { ...committed.routes, es: ["es-en/base-memory/2.0", EN_FR] },
    });

  it("is the union of the languages' routes, each model once, in order", () => {
    expect(modelsFor(pivot(), ["en", "es"]).map((m) => m.version)).toEqual([EN_FR, "es-en/base-memory/2.0"]);
    expect(modelsFor(pivot(), ["es", "en"]).map((m) => m.version)).toEqual(["es-en/base-memory/2.0", EN_FR]);
  });

  it("needs nothing for a language without a route", () => {
    expect(modelsFor(parseCatalogue(committed), ["es"])).toEqual([]);
    expect(modelsFor(parseCatalogue(committed), ["en", "es"]).map((m) => m.version)).toEqual([EN_FR]);
  });

  it("finds models by id, leaving out an id the catalogue does not list", () => {
    expect(modelsById(pivot(), ["es-en/base-memory/2.0", "de-en/tiny/1.0"]).map((m) => [m.version, m.from])).toEqual([
      ["es-en/base-memory/2.0", "es"],
    ]);
  });
});
