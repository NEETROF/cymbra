import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  ANALYZER_CONSTANTS,
  coreAnalyzerVersion,
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
    expect(shippedPairs(appWith({ pairs: ["en-fr", "es-fr"] }))).toEqual(["en-fr", "es-fr"]);
  });

  it("puts each pair's pack at a path of its own, and reads its studied side", () => {
    expect(packFile("en-fr")).toBe("assets/packs/en-fr.lingua");
    expect(packFile("es-fr")).toBe("assets/packs/es-fr.lingua");
    expect(studiedOf("en-fr")).toBe("en");
    expect(studiedOf("es-fr")).toBe("es");
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
  });

  it("scans a pack's studied language and analyser version from its metadata", () => {
    const fixture = readFileSync(join(__dirname, "fixtures/en-fr.testdata.lingua"));
    expect(packMeta(fixture)).toEqual({ studied: "en", analyzerVersion: coreAnalyzerVersion("en", modRs) });
    expect(packMeta(new Uint8Array([1, 2, 3]))).toEqual({ studied: null, analyzerVersion: null });
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
