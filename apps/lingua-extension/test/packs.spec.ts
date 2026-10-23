import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  ANALYZER_CONSTANTS,
  assertPacksMatchEngine,
  baseFile,
  coreAnalyzerVersion,
  nativeOf,
  packFile,
  packMeta,
  shippedPairs,
  shippedStudied,
  studiedOf,
} from "../tool/packs.mjs";

const repo = join(__dirname, "../../..");
const modRs = readFileSync(join(repo, "crates/lingua-core/src/analysis/mod.rs"), "utf8");
const languageRs = readFileSync(join(repo, "crates/lingua-core/src/analysis/language.rs"), "utf8");

function appWith(packs: unknown): string {
  const dir = mkdtempSync(join(tmpdir(), "lingua-packs-"));
  writeFileSync(join(dir, "packs.json"), JSON.stringify(packs));
  return dir;
}

describe("the shipped pairs", () => {
  it("ships en-fr, es-fr, es-en, en-es then fr-en and fr-es, each native language's default first (enable-lingua-french)", () => {
    expect(shippedPairs()).toEqual(["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"]);
  });

  it("refuses a list it could not build from", () => {
    expect(() => shippedPairs(appWith({ pairs: [] }))).toThrow(/must list/);
    expect(() => shippedPairs(appWith({ pairs: ["english"] }))).toThrow(/must list/);
    expect(() => shippedPairs(appWith({}))).toThrow(/must list/);
    expect(() => shippedPairs(appWith({ pairs: ["en-fr", "en-fr"] }))).toThrow(/twice/);
    // A pair glossed in the language it studies (generalise-lingua-native-language D8).
    expect(() => shippedPairs(appWith({ pairs: ["en-fr", "fr-fr"] }))).toThrow(
      "packs.json: fr-fr would be glossed in the language it studies",
    );
    expect(() => shippedPairs(appWith({ pairs: ["es-es"] }))).toThrow(/glossed in the language it studies/);
    expect(shippedPairs(appWith({ pairs: ["en-fr", "es-fr"] }))).toEqual(["en-fr", "es-fr"]);
  });

  it("puts each pair's layer at the pair's path and each studied language's base once, and reads its studied side", () => {
    expect(packFile("en-fr")).toBe("assets/packs/en-fr.lingua");
    expect(packFile("es-fr")).toBe("assets/packs/es-fr.lingua");
    // ship-lingua-pack-bases-once D1: a base per studied language, named without a dash.
    expect(baseFile("en")).toBe("assets/bases/en.lingua");
    expect(shippedStudied(["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"])).toEqual(["en", "es", "fr"]);
    expect(shippedStudied(["es-en", "en-fr", "es-fr"])).toEqual(["es", "en"]);
    expect(studiedOf("en-fr")).toBe("en");
    expect(studiedOf("es-fr")).toBe("es");
    expect(nativeOf("en-fr")).toBe("fr");
    expect(nativeOf("es-en")).toBe("en");
  });
});

describe("each pack against its own language's analyser", () => {
  it("reads each language's version from lingua-core, never Spanish's for English", () => {
    const spanishFirst =
      'pub const SPANISH_ANALYZER_VERSION: &str = "0.1.0";\npub const ANALYZER_VERSION: &str = "1.1.0";';
    expect(coreAnalyzerVersion("en", spanishFirst)).toBe("1.1.0");
    expect(coreAnalyzerVersion("es", spanishFirst)).toBe("0.1.0");
    expect(coreAnalyzerVersion("pt", spanishFirst)).toBeNull();
    expect(coreAnalyzerVersion("en", modRs)).toMatch(/^\d+\.\d+\.\d+$/);
    expect(coreAnalyzerVersion("es", modRs)).toMatch(/^\d+\.\d+\.\d+$/);
    // French has its own analysis (add-lingua-french-analysis): its own version, no longer one of
    // the baseline's `0.x`, read beside the two others'.
    expect(coreAnalyzerVersion("fr", modRs)).toMatch(/^[1-9]\d*\.\d+\.\d+$/);
  });

  it("reads a pack's studied language, native language and analyser version from its metadata", () => {
    const fixture = readFileSync(join(__dirname, "fixtures/en-fr.testdata.lingua"));
    expect(packMeta(fixture)).toEqual({
      studied: "en",
      native: "fr",
      analyzerVersion: coreAnalyzerVersion("en", modRs),
      part: null,
      baseVersion: null,
    });
    expect(packMeta(new Uint8Array([1, 2, 3]))).toEqual(NONE);
  });

  it("reads a part's kind and the base version it shares with its other part (ship-lingua-pack-bases-once D4)", () => {
    const base = packMeta(readFileSync(join(__dirname, "fixtures/en.testdata.base.lingua")));
    const layer = packMeta(readFileSync(join(__dirname, "fixtures/en-fr.testdata.layer.lingua")));
    expect(base).toEqual({
      studied: "en",
      native: null,
      analyzerVersion: coreAnalyzerVersion("en", modRs),
      part: "base",
      baseVersion: expect.stringMatching(/^[0-9a-f]{64}$/),
    });
    expect(layer).toEqual({
      studied: "en",
      native: "fr",
      analyzerVersion: null,
      part: "layer",
      baseVersion: base.baseVersion,
    });
  });

  it("reads the metadata exactly where the container puts it, never a lookalike past it", () => {
    // A section that spells another pair is not the metadata.
    const lookalike = container({ studied: "en", native: "fr", analyzer_version: "1.1.0" }, '"native":"es"');
    expect(packMeta(lookalike)).toEqual({ ...NONE, studied: "en", native: "fr", analyzerVersion: "1.1.0" });
    const none = NONE;
    // Not a container, a truncated one, or metadata that does not parse.
    expect(packMeta(Buffer.from('LINGUAPX\x01\x00{"studied":"en"}', "latin1"))).toEqual(none);
    expect(packMeta(container({ studied: "en" }).subarray(0, 20))).toEqual(none);
    const broken = container({ studied: "en" });
    broken[14] = 0x7b + 1; // "{" → "|"
    expect(packMeta(broken)).toEqual(none);
    // A field of another type reads as absent.
    expect(packMeta(container({ studied: "en", native: 3 }))).toEqual({ ...NONE, studied: "en" });
    expect(packMeta(container({ part: "layer", studied: "en", base_version: 7 }))).toEqual({
      ...NONE,
      studied: "en",
      part: "layer",
    });
  });

  it("maps every language lingua-core analyses to the constant that versions it", () => {
    // `StudiedLanguage::Spanish => "es"` in tag(), `StudiedLanguage::Spanish => crate::analysis::X` in
    // analyzer_version(): the map must name the same constant for every language, and miss none.
    const tags = new Map([...languageRs.matchAll(/StudiedLanguage::(\w+) => "([a-z]{2})"/g)].map((m) => [m[1], m[2]]));
    const constants = new Map(
      [...languageRs.matchAll(/StudiedLanguage::(\w+) => crate::analysis::([A-Z_]+)/g)].map((m) => [m[1], m[2]]),
    );
    expect(tags.size).toBeGreaterThan(1);
    const fromCore = Object.fromEntries([...tags].map(([variant, tag]) => [tag, constants.get(variant)]));
    expect(fromCore).toEqual({ ...ANALYZER_CONSTANTS });
  });
});

/** What packMeta reads of bytes that are no container. */
const NONE = { studied: null, native: null, analyzerVersion: null, part: null, baseVersion: null };

/** A container as lingua-core's format.rs writes it: magic, format 1, the JSON metadata, one section. */
function container(meta: unknown, section = "x"): Buffer {
  const json = Buffer.from(JSON.stringify(meta), "utf8");
  const head = Buffer.alloc(14);
  head.write("LINGUAPK", 0, "latin1");
  head.writeUInt16LE(1, 8);
  head.writeUInt32LE(json.length, 10);
  const name = Buffer.from("notice", "latin1");
  const data = Buffer.from(section, "utf8");
  const count = Buffer.alloc(2);
  count.writeUInt16LE(1);
  const dataLength = Buffer.alloc(4);
  dataLength.writeUInt32LE(data.length);
  return Buffer.concat([head, json, count, Buffer.from([name.length]), name, dataLength, data]);
}

describe("the build's check of the parts it ships (ship-lingua-pack-bases-once D4)", () => {
  const versions = 'pub const ANALYZER_VERSION: &str = "1.1.0";\npub const SPANISH_ANALYZER_VERSION: &str = "1.2.0";';
  const BASE_EN = "b".repeat(64);
  const BASE_ES = "c".repeat(64);

  type Meta = Record<string, unknown>;
  /** An app directory listing `layers`' pairs, each layer's and each base's metadata given. */
  function appWithParts(layers: Record<string, Meta>, bases: Record<string, Meta>): string {
    const dir = appWith({ pairs: Object.keys(layers) });
    mkdirSync(join(dir, "assets/packs"), { recursive: true });
    mkdirSync(join(dir, "assets/bases"), { recursive: true });
    for (const [pair, meta] of Object.entries(layers)) writeFileSync(join(dir, packFile(pair)), container(meta));
    for (const [studied, meta] of Object.entries(bases)) writeFileSync(join(dir, baseFile(studied)), container(meta));
    return dir;
  }
  const layer = (pair: string, baseVersion: string): Meta => ({
    part: "layer",
    studied: studiedOf(pair),
    native: nativeOf(pair),
    pack_version: "0.0.1",
    base_version: baseVersion,
  });
  const base = (studied: string, analyzerVersion: string, baseVersion: string): Meta => ({
    part: "base",
    studied,
    analyzer_version: analyzerVersion,
    base_version: baseVersion,
  });
  /** en-fr and es-fr, each layer of its base, each base of its language's analyser. */
  const today = (layers: Record<string, Meta> = {}, bases: Record<string, Meta> = {}) =>
    appWithParts(
      { "en-fr": layer("en-fr", BASE_EN), "es-fr": layer("es-fr", BASE_ES), ...layers },
      { en: base("en", "1.1.0", BASE_EN), es: base("es", "1.2.0", BASE_ES), ...bases },
    );
  const check = (appDir: string) => () => assertPacksMatchEngine({ appDir, modRs: versions });

  it("passes the six shipped pairs' layers and their three bases", () => {
    // Every pair packs.json lists, as gen:pack builds it for lingua-core's analyser of its language —
    // in a directory of its own: assets/ is generated, and absent from a fresh checkout until
    // `yarn gen:pack`.
    const pairs = shippedPairs();
    const version = (studied: string) => `${studied}`.repeat(32);
    const shipped = appWithParts(
      Object.fromEntries(pairs.map((pair) => [pair, layer(pair, version(studiedOf(pair)))])),
      Object.fromEntries(
        shippedStudied(pairs).map((studied) => [
          studied,
          base(studied, coreAnalyzerVersion(studied, modRs) as string, version(studied)),
        ]),
      ),
    );
    expect(() => assertPacksMatchEngine({ appDir: shipped, modRs })).not.toThrow();
    expect(check(today())).not.toThrow();
  });

  it("refuses a layer of another pair, naming the layer, the language and the rebuild", () => {
    expect(check(today({ "en-fr": { ...layer("en-fr", BASE_EN), native: "es" } }))).toThrow(
      'assets/packs/en-fr.lingua is glossed in "es", not "fr": rebuild it with `yarn gen:pack:real`',
    );
    expect(check(today({ "en-fr": { ...layer("en-fr", BASE_EN), studied: "es" } }))).toThrow(
      'assets/packs/en-fr.lingua studies "es", not "en"',
    );
  });

  it("refuses a layer of another base, naming the pair", () => {
    expect(check(today({ "es-fr": layer("es-fr", BASE_EN) }))).toThrow(
      `es-fr: assets/packs/es-fr.lingua was built against the base ${BASE_EN}, but assets/bases/es.lingua is ${BASE_ES}`,
    );
  });

  it("refuses a base of another analyser, naming the base, both versions and the rebuild", () => {
    expect(check(today({}, { es: base("es", "0.1.0", BASE_ES) }))).toThrow(
      "Analyzer version mismatch: assets/bases/es.lingua is 0.1.0 but lingua-core's es analyser is 1.2.0. " +
        "The engine refuses a mismatched pack at runtime. Rebuild it: `yarn gen:pack:real`",
    );
    expect(check(today({}, { es: base("en", "1.1.0", BASE_ES) }))).toThrow(
      'assets/bases/es.lingua studies "en", not "es"',
    );
  });

  it("refuses a whole pack where a part belongs, and one part for the other", () => {
    const whole = { studied: "en", native: "fr", analyzer_version: "1.1.0" };
    expect(check(today({ "en-fr": whole }))).toThrow(
      "assets/packs/en-fr.lingua is a whole pack: a package carries each pair's layer under assets/packs/ and each studied language's base under assets/bases/.",
    );
    expect(check(today({}, { en: whole }))).toThrow("assets/bases/en.lingua is a whole pack");
    expect(check(today({ "en-fr": base("en", "1.1.0", BASE_EN) }))).toThrow(
      "assets/packs/en-fr.lingua is a base, not a layer: rebuild it with `yarn gen:pack:real`",
    );
    expect(check(today({}, { en: { ...layer("en-fr", BASE_EN), analyzer_version: "1.1.0" } }))).toThrow(
      "assets/bases/en.lingua is a layer, not a base",
    );
  });

  it("refuses a part that is missing or whose metadata does not read", () => {
    const missingBase = appWithParts({ "en-fr": layer("en-fr", BASE_EN) }, {});
    expect(check(missingBase)).toThrow("assets/bases/en.lingua is missing — run `yarn gen:pack:real`");
    expect(check(appWith({ pairs: ["en-fr"] }))).toThrow("assets/bases/en.lingua is missing");
    const missingLayer = appWithParts({ "en-fr": layer("en-fr", BASE_EN) }, { en: base("en", "1.1.0", BASE_EN) });
    rmSync(join(missingLayer, packFile("en-fr")));
    expect(check(missingLayer)).toThrow("assets/packs/en-fr.lingua is missing");
    const garbage = today();
    writeFileSync(join(garbage, packFile("en-fr")), "not a pack");
    expect(check(garbage)).toThrow("assets/packs/en-fr.lingua has no metadata this build can read");
    const truncated = today();
    writeFileSync(join(truncated, baseFile("en")), container(base("en", "1.1.0", BASE_EN)).subarray(0, 20));
    expect(check(truncated)).toThrow(
      "assets/bases/en.lingua has no metadata this build can read (its studied, analyzerVersion, baseVersion)",
    );
    expect(check(today({ "en-fr": { ...layer("en-fr", BASE_EN), base_version: undefined } }))).toThrow(
      "assets/packs/en-fr.lingua has no metadata this build can read (its studied, native, baseVersion)",
    );
    expect(check(today({ "en-fr": { ...layer("en-fr", BASE_EN), part: "index" } }))).toThrow(
      "assets/packs/en-fr.lingua has no metadata this build can read",
    );
    expect(() =>
      assertPacksMatchEngine({ appDir: appWith({ pairs: ["en-fr"] }), pairs: [], modRs: versions }),
    ).not.toThrow();
  });

  it("refuses a language lingua-core has no analyser version for", () => {
    const portuguese = appWithParts({ "pt-fr": layer("pt-fr", BASE_EN) }, { pt: base("pt", "1.0.0", BASE_EN) });
    expect(check(portuguese)).toThrow('lingua-core has no analyser version for "pt" (assets/bases/pt.lingua)');
  });
});
