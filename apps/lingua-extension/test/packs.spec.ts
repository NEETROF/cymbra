import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  ANALYZER_CONSTANTS,
  assertPacksMatchEngine,
  coreAnalyzerVersion,
  nativeOf,
  packFile,
  packMeta,
  shippedPairs,
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
  it("ships en-fr then es-fr, the default language first (enable-lingua-spanish)", () => {
    expect(shippedPairs()).toEqual(["en-fr", "es-fr"]);
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

  it("puts each pair's pack at a path of its own, and reads its studied side", () => {
    expect(packFile("en-fr")).toBe("assets/packs/en-fr.lingua");
    expect(packFile("es-fr")).toBe("assets/packs/es-fr.lingua");
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
    });
    const none = { studied: null, native: null, analyzerVersion: null };
    expect(packMeta(new Uint8Array([1, 2, 3]))).toEqual(none);
  });

  it("reads the metadata exactly where the container puts it, never a lookalike past it", () => {
    // A section that spells another pair is not the metadata.
    const lookalike = container({ studied: "en", native: "fr", analyzer_version: "1.1.0" }, '"native":"es"');
    expect(packMeta(lookalike)).toEqual({ studied: "en", native: "fr", analyzerVersion: "1.1.0" });
    const none = { studied: null, native: null, analyzerVersion: null };
    // Not a container, a truncated one, or metadata that does not parse.
    expect(packMeta(Buffer.from('LINGUAPX\x01\x00{"studied":"en"}', "latin1"))).toEqual(none);
    expect(packMeta(container({ studied: "en" }).subarray(0, 20))).toEqual(none);
    const broken = container({ studied: "en" });
    broken[14] = 0x7b + 1; // "{" → "|"
    expect(packMeta(broken)).toEqual(none);
    // A field of another type reads as absent.
    expect(packMeta(container({ studied: "en", native: 3 }))).toEqual({
      studied: "en",
      native: null,
      analyzerVersion: null,
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

describe("the build's check of the packs it ships (generalise-lingua-native-language D8)", () => {
  const versions = 'pub const ANALYZER_VERSION: &str = "1.1.0";\npub const SPANISH_ANALYZER_VERSION: &str = "1.2.0";';

  /** An app directory listing `pairs`, each pack's metadata from `meta`. */
  function appWithPacks(packs: Record<string, Record<string, unknown>>): string {
    const dir = appWith({ pairs: Object.keys(packs) });
    mkdirSync(join(dir, "assets/packs"), { recursive: true });
    for (const [pair, meta] of Object.entries(packs)) writeFileSync(join(dir, packFile(pair)), container(meta));
    return dir;
  }
  const check = (appDir: string) => () => assertPacksMatchEngine({ appDir, modRs: versions });

  it("passes both shipped pairs", () => {
    // Every pair packs.json lists, each pack built as gen:pack builds it for lingua-core's analyser
    // of its language — in a directory of its own: assets/packs/ is generated, and absent from a
    // fresh checkout until `yarn gen:pack`.
    const shipped = appWithPacks(
      Object.fromEntries(
        shippedPairs().map((pair) => [
          pair,
          {
            studied: studiedOf(pair),
            native: nativeOf(pair),
            analyzer_version: coreAnalyzerVersion(studiedOf(pair), modRs),
          },
        ]),
      ),
    );
    expect(() => assertPacksMatchEngine({ appDir: shipped, modRs })).not.toThrow();
    const today = appWithPacks({
      "en-fr": { studied: "en", native: "fr", analyzer_version: "1.1.0" },
      "es-fr": { studied: "es", native: "fr", analyzer_version: "1.2.0" },
    });
    expect(check(today)).not.toThrow();
  });

  it("refuses a pack whose native language is not its pair's, naming the pack, the language and the rebuild", () => {
    const glossedInSpanish = appWithPacks({ "en-fr": { studied: "en", native: "es", analyzer_version: "1.1.0" } });
    expect(check(glossedInSpanish)).toThrow(
      'assets/packs/en-fr.lingua is glossed in "es", not "fr": rebuild it with `yarn gen:pack:real`',
    );
  });

  it("keeps refusing a pack of another language, another analyser, an unreadable one, or none at all", () => {
    expect(check(appWithPacks({ "en-fr": { studied: "es", native: "fr", analyzer_version: "1.2.0" } }))).toThrow(
      'assets/packs/en-fr.lingua studies "es", not "en"',
    );
    expect(check(appWithPacks({ "es-fr": { studied: "es", native: "fr", analyzer_version: "0.1.0" } }))).toThrow(
      "Analyzer version mismatch: assets/packs/es-fr.lingua is 0.1.0 but lingua-core's es analyser is 1.2.0.",
    );
    expect(check(appWith({ pairs: ["en-fr"] }))).toThrow("assets/packs/en-fr.lingua is missing");
    // A pack whose metadata does not read would only be refused at runtime.
    const unreadable = "assets/packs/en-fr.lingua has no metadata this build can read";
    const garbage = appWithPacks({ "en-fr": {} });
    writeFileSync(join(garbage, packFile("en-fr")), "not a pack");
    expect(check(garbage)).toThrow(unreadable);
    const truncated = appWithPacks({ "en-fr": {} });
    writeFileSync(
      join(truncated, packFile("en-fr")),
      container({ studied: "en", native: "fr", analyzer_version: "1.1.0" }).subarray(0, 20),
    );
    expect(check(truncated)).toThrow(unreadable);
    expect(check(appWithPacks({ "en-fr": { studied: "en", analyzer_version: "1.1.0" } }))).toThrow(unreadable);
    expect(check(appWithPacks({ "en-fr": { studied: "en", native: "fr" } }))).toThrow(unreadable);
    expect(check(appWithPacks({ "en-fr": { native: "fr", analyzer_version: "1.1.0" } }))).toThrow(unreadable);
    expect(() =>
      assertPacksMatchEngine({ appDir: appWith({ pairs: ["en-fr"] }), pairs: [], modRs: versions }),
    ).not.toThrow();
  });
});
