import { describe, expect, it } from "vitest";
import {
  acceptedLanguages,
  basePath,
  DEFAULT_LANGUAGE,
  DEFAULT_NATIVE,
  defaultPair,
  nativeOf,
  packPath,
  pairFor,
  pairOf,
  pairsOf,
  readerPairs,
  readingLanguage,
  SHIPPED_PAIRS,
  shippedNatives,
  studiedOf,
} from "@/analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import {
  baseFile,
  nativeOf as packNativeOf,
  packFile,
  shippedNatives as packShippedNatives,
  shippedPairs,
  shippedStudied,
  studiedOf as packStudiedOf,
} from "../tool/packs.mjs";
import { makeFakePort } from "./helpers.ts";

/** Two French-native pairs beside a pair glossed in English (generalise-lingua-native-language):
 *  packs.json in change 34 (enable-lingua-english-speakers), held here so these tests keep it. */
const MIXED = ["en-fr", "es-fr", "es-en"];
/** That list beside a pair glossed in Spanish: packs.json in change 35 (enable-lingua-spanish-speakers). */
const ALL = [...MIXED, "en-es"];
/** That list beside the pairs studying French, last: packs.json since change 52 (enable-lingua-french). */
const FRENCH = [...ALL, "fr-en", "fr-es"];
/** fr-es under its floor: the list ending with fr-en (enable-lingua-french D2). */
const FR_EN_ALONE = [...ALL, "fr-en"];

describe("the pair that serves a language", () => {
  it("is the first listed pair that studies it, glossed in the reader's native language", () => {
    expect(pairFor("en", "fr", ["en-fr", "es-fr"])).toBe("en-fr");
    expect(pairFor("es", "fr", ["en-fr", "es-fr"])).toBe("es-fr");
    expect(pairFor("en", "fr", ["en-de", "en-fr"])).toBe("en-fr");
  });

  it("is chosen by both sides", () => {
    expect(pairFor("es", "fr", MIXED)).toBe("es-fr");
    expect(pairFor("es", "en", MIXED)).toBe("es-en");
    expect(pairFor("es", "fr", ["es-en", "es-fr"])).toBe("es-fr"); // es-en is listed first, and never chosen
    expect(pairFor("es", "en", ["es-fr"])).toBeNull();
    expect(pairFor("en", "en", MIXED)).toBeNull(); // a reader never studies their native language
  });

  it("is none for a language no listed pair studies", () => {
    expect(pairFor("es", "fr", ["en-fr"])).toBeNull();
    expect(pairFor("fr", "fr", ["en-fr"])).toBeNull(); // the native side is not studied
    expect(pairFor("", "fr", ["en-fr"])).toBeNull();
  });

  it("is read from the bundle's list by default", () => {
    expect(pairFor("en", DEFAULT_NATIVE)).toBe("en-fr");
    expect(pairFor("es", DEFAULT_NATIVE)).toBe("es-fr");
  });
});

describe("a pair's name (generalise-lingua-translation-routes-by-pair D1, D2)", () => {
  it("is the studied language, one `-`, the native language — and is read back at the first `-`", () => {
    expect(pairOf("en", "fr")).toBe("en-fr");
    expect(pairOf("es", "fr")).toBe("es-fr");
    expect(pairOf("en", "es")).toBe("en-es");
    for (const [studied, native] of [
      ["en", "fr"],
      ["es", "en"],
    ]) {
      expect(studiedOf(pairOf(studied!, native!))).toBe(studied);
      expect(nativeOf(pairOf(studied!, native!))).toBe(native);
    }
    // A second `-` belongs to the native side: `en-fr-x` is en / fr-x, as the catalogue's parser
    // reads a route key (model-manifest.ts). No `-`: a studied language alone, with no native side.
    expect(studiedOf("en-fr-x")).toBe("en");
    expect(nativeOf("en-fr-x")).toBe("fr-x");
    expect(studiedOf("en")).toBe("en");
    expect(nativeOf("en")).toBe("");
  });

  it("is read the same by the build (tool/packs.mjs)", () => {
    for (const pair of ["en-fr", "es-en", "en-fr-x", "en", "-fr", "en-"]) {
      expect(packStudiedOf(pair), pair).toBe(studiedOf(pair));
      expect(packNativeOf(pair), pair).toBe(nativeOf(pair));
    }
  });
});

describe("the reader's pairs (routes-by-pair D5)", () => {
  it("are the listed pair of their native language that studies each accepted language, in order", () => {
    expect(readerPairs(["en"], "fr", MIXED)).toEqual(["en-fr"]);
    expect(readerPairs(["es", "en"], "fr", MIXED)).toEqual(["es-fr", "en-fr"]);
    expect(readerPairs(["es"], "en", MIXED)).toEqual(["es-en"]);
  });

  it("leave out a language no listed pair studies for that native language", () => {
    expect(readerPairs(["en", "de"], "fr", MIXED)).toEqual(["en-fr"]); // de-fr: no pack
    expect(readerPairs(["en"], "es", MIXED)).toEqual([]); // en-es: not in change 34's list
    expect(readerPairs(["en"], "es", ALL)).toEqual(["en-es"]); // en-es: shipped since change 35
    expect(readerPairs([], "fr", MIXED)).toEqual([]);
  });

  it("read the bundle's list by default", () => {
    expect(readerPairs(["en", "es"], DEFAULT_NATIVE)).toEqual(["en-fr", "es-fr"]);
  });

  it("study French through the pair of the reader's native language, never for a French speaker (enable-lingua-french)", () => {
    expect(readerPairs(["es", "fr"], "en", FRENCH)).toEqual(["es-en", "fr-en"]);
    expect(readerPairs(["fr", "es"], "en", FRENCH)).toEqual(["fr-en", "es-en"]);
    expect(readerPairs(["en", "fr"], "es", FRENCH)).toEqual(["en-es", "fr-es"]);
    expect(readerPairs(["fr"], "es", FRENCH)).toEqual(["fr-es"]);
    expect(readerPairs(["en", "es", "fr"], "fr", FRENCH)).toEqual(["en-fr", "es-fr"]); // no fr-fr
    // fr-es under its floor: a Spanish speaker's French has no pair.
    expect(readerPairs(["en", "fr"], "es", FR_EN_ALONE)).toEqual(["en-es"]);
    expect(readerPairs(["es", "fr"], "en", FR_EN_ALONE)).toEqual(["es-en", "fr-en"]);
  });
});

describe("the pairs of a native language", () => {
  it("are the listed pairs glossed in it, in listed order", () => {
    expect(nativeOf("en-fr")).toBe("fr");
    expect(nativeOf("es-en")).toBe("en");
    expect(pairsOf("fr", MIXED)).toEqual(["en-fr", "es-fr"]);
    expect(pairsOf("en", MIXED)).toEqual(["es-en"]);
    expect(pairsOf("es", MIXED)).toEqual([]);
  });

  it("start with the native language's default pair, none when no pair is glossed in it", () => {
    expect(defaultPair("fr", MIXED)).toBe("en-fr");
    expect(defaultPair("en", MIXED)).toBe("es-en"); // an English reader's
    expect(defaultPair("en", ["es-en", "en-fr"])).toBe("es-en");
    expect(defaultPair("fr", ["es-en", "en-fr"])).toBe("en-fr");
    expect(defaultPair("es", MIXED)).toBeNull();
  });

  it("are, in the bundle's list, French's two pairs, English's two and Spanish's two, French last (enable-lingua-french)", () => {
    expect(SHIPPED_PAIRS).toEqual(FRENCH);
    // A French-native reader — every installed reader (M22) — keeps their pairs and their default.
    expect(DEFAULT_NATIVE).toBe("fr");
    expect(pairsOf(DEFAULT_NATIVE)).toEqual(["en-fr", "es-fr"]);
    expect(defaultPair(DEFAULT_NATIVE)).toBe(SHIPPED_PAIRS[0]);
    expect(defaultPair(DEFAULT_NATIVE)).toBe("en-fr");
    // An English-native reader studies Spanish through es-en (change 34), then French through fr-en:
    // Spanish stays their default (D1).
    expect(pairsOf("en")).toEqual(["es-en", "fr-en"]);
    expect(defaultPair("en")).toBe("es-en");
    // A Spanish-native reader studies English through en-es (change 35), then French through fr-es:
    // English stays their default.
    expect(pairsOf("es")).toEqual(["en-es", "fr-es"]);
    expect(defaultPair("es")).toBe("en-es");
  });

  it("are English's alone for a Spanish speaker when fr-es is under its floor (enable-lingua-french D2)", () => {
    expect(pairsOf("es", FR_EN_ALONE)).toEqual(["en-es"]);
    expect(pairsOf("en", FR_EN_ALONE)).toEqual(["es-en", "fr-en"]);
    expect(pairsOf("fr", FR_EN_ALONE)).toEqual(["en-fr", "es-fr"]);
  });
});

describe("the shipped natives (add-lingua-native-language-choice D1)", () => {
  it("are the native languages with a shipped pair, in listed order, once each", () => {
    expect(shippedNatives(["en-fr", "es-fr"])).toEqual(["fr"]);
    expect(shippedNatives(MIXED)).toEqual(["fr", "en"]);
    expect(shippedNatives(ALL)).toEqual(["fr", "en", "es"]);
    expect(shippedNatives(FRENCH)).toEqual(["fr", "en", "es"]); // the French pairs add no native language
    expect(shippedNatives(["es-en", "en-fr", "en-es"])).toEqual(["en", "fr", "es"]);
    expect(shippedNatives([])).toEqual([]);
  });

  it("count three in the bundle's list: French first (M22), English, then Spanish (enable-lingua-spanish-speakers; enable-lingua-french adds none)", () => {
    expect(shippedNatives()).toEqual([DEFAULT_NATIVE, "en", "es"]);
  });

  it("are read the same by the build (tool/packs.mjs)", () => {
    for (const pairs of [["en-fr", "es-fr"], MIXED, ALL, FRENCH, ["es-en", "en-fr", "en-es"], ["en-fr-x", "en"]]) {
      expect(packShippedNatives(pairs), pairs.join(",")).toEqual(shippedNatives(pairs));
    }
    expect(packShippedNatives()).toEqual(shippedNatives());
  });
});

/** A reader of `native` studying `studied`. */
const reader = async (studied: StudiedLanguage[], native: NativeLanguage = "fr") => {
  const { port } = makeFakePort();
  port.nativeLanguage = async () => native;
  await port.setStudiedLanguages(studied);
  return port;
};

describe("the language a surface reads in", () => {
  it("is the reader's first studied language a shipped pair studies", async () => {
    expect(await readingLanguage(await reader(["es", "en"]), ["en-fr", "es-fr"])).toBe("es");
    expect(await readingLanguage(await reader(["es", "en"]), ["en-fr"])).toBe("en");
    expect(await readingLanguage(await reader(["en", "es"]), ["en-fr", "es-fr"])).toBe("en");
  });

  it("is the default pair's when none of them is shipped", async () => {
    expect(await readingLanguage(await reader(["es"]), ["en-fr"])).toBe("en");
    expect(await readingLanguage(await reader(["en"]), ["es-fr", "en-fr"])).toBe("en");
    expect(await readingLanguage(await reader(["es"]), ["es-fr"])).toBe("es");
  });

  it("reads the bundle's list by default, which starts in English and ships Spanish (enable-lingua-spanish)", async () => {
    expect(DEFAULT_LANGUAGE).toBe("en");
    // Spanish ships: a reader who put it first reads in it.
    expect(await readingLanguage(await reader(["es", "en"]))).toBe("es");
    // A reader with no language of their own starts in the default.
    expect(await readingLanguage(await reader([]))).toBe("en");
  });

  it("is never a language a French reader is served in English only", async () => {
    expect(await readingLanguage(await reader(["es", "en"]), MIXED)).toBe("es");
    expect(await readingLanguage(await reader(["es"]), ["en-fr", "es-en"])).toBe("en");
  });

  it("is, for an English reader, a language glossed in English", async () => {
    expect(await readingLanguage(await reader(["es"], "en"), MIXED)).toBe("es");
    // English is not served to them: their default pair's language instead.
    expect(await readingLanguage(await reader(["en"], "en"), MIXED)).toBe("es");
    expect(await readingLanguage(await reader([], "en"), MIXED)).toBe("es");
  });

  it("only reads the reader's studied languages", async () => {
    const port = await reader(["es", "en"]);
    await readingLanguage(port, ["en-fr"]);
    expect(await port.studiedLanguages()).toEqual(["es", "en"]);
  });
});

describe("the languages a device accepts from the sync", () => {
  it("are the reader's shipped languages, in the reader's order", async () => {
    expect(await acceptedLanguages(await reader(["es", "en"]), ["en-fr", "es-fr"])).toEqual(["es", "en"]);
    expect(await acceptedLanguages(await reader(["es", "en"]), ["en-fr"])).toEqual(["en"]);
  });

  it("are the default pair's alone when none is shipped, and English for every reader today", async () => {
    expect(await acceptedLanguages(await reader(["es"]), ["en-fr"])).toEqual(["en"]);
    expect(await acceptedLanguages(await reader(["en"]))).toEqual(["en"]);
  });

  it("are the languages of the reader's native language's pairs", async () => {
    expect(await acceptedLanguages(await reader(["es", "en"]), MIXED)).toEqual(["es", "en"]);
    expect(await acceptedLanguages(await reader(["en", "es"], "en"), MIXED)).toEqual(["es"]);
    expect(await acceptedLanguages(await reader(["en"], "en"), MIXED)).toEqual(["es"]);
  });

  it("do not change for any reader of French with the bundle's list, es-en and en-es listed (D2)", async () => {
    for (const studied of [[], ["en"], ["es"], ["en", "es"], ["es", "en"]] as StudiedLanguage[][]) {
      const french = await reader(studied);
      const expected = studied.length > 0 ? studied : ["en"];
      expect(await acceptedLanguages(french)).toEqual(expected);
      expect(await readingLanguage(french)).toBe(expected[0]);
    }
  });

  it("leave French out for any reader of French with the bundle's list, French listed (enable-lingua-french)", async () => {
    for (const studied of [["fr"], ["en", "fr"], ["fr", "es"], ["es", "fr", "en"]] as StudiedLanguage[][]) {
      const french = await reader(studied);
      const expected = studied.filter((language) => language !== "fr");
      expect(await acceptedLanguages(french)).toEqual(expected.length > 0 ? expected : ["en"]);
      expect(await readingLanguage(french)).not.toBe("fr");
    }
  });

  it("are Spanish and French, in the reader's order, for a reader of English who studies them (enable-lingua-french)", async () => {
    const cases: [StudiedLanguage[], StudiedLanguage[]][] = [
      [["fr"], ["fr"]],
      [
        ["es", "fr"],
        ["es", "fr"],
      ],
      [
        ["fr", "es"],
        ["fr", "es"],
      ],
      [["en", "fr"], ["fr"]],
    ];
    for (const [studied, accepted] of cases) {
      const english = await reader(studied, "en");
      expect(await acceptedLanguages(english)).toEqual(accepted);
      expect(await readingLanguage(english)).toBe(accepted[0]);
      expect(readerPairs(await acceptedLanguages(english), "en")).toEqual(accepted.map((l) => `${l}-en`));
    }
  });

  it("are English and French, in the reader's order, for a reader of Spanish who studies them (enable-lingua-french)", async () => {
    const cases: [StudiedLanguage[], StudiedLanguage[]][] = [
      [["fr"], ["fr"]],
      [
        ["en", "fr"],
        ["en", "fr"],
      ],
      [
        ["fr", "en"],
        ["fr", "en"],
      ],
      [["es", "fr"], ["fr"]],
    ];
    for (const [studied, accepted] of cases) {
      const spanish = await reader(studied, "es");
      expect(await acceptedLanguages(spanish)).toEqual(accepted);
      expect(await readingLanguage(spanish)).toBe(accepted[0]);
      expect(readerPairs(await acceptedLanguages(spanish), "es")).toEqual(accepted.map((l) => `${l}-es`));
    }
    // fr-es under its floor: French is not theirs, English their default.
    expect(await acceptedLanguages(await reader(["fr"], "es"), FR_EN_ALONE)).toEqual(["en"]);
  });

  it("are Spanish for any reader of English with the bundle's list (English speakers learning Spanish)", async () => {
    for (const studied of [[], ["en"], ["es"], ["en", "es"], ["es", "en"]] as StudiedLanguage[][]) {
      const english = await reader(studied, "en");
      expect(await acceptedLanguages(english)).toEqual(["es"]);
      expect(await readingLanguage(english)).toBe("es");
      expect(readerPairs(await acceptedLanguages(english), "en")).toEqual(["es-en"]);
    }
  });

  it("are English for any reader of Spanish with the bundle's list (Spanish speakers learning English)", async () => {
    for (const studied of [[], ["en"], ["es"], ["en", "es"], ["es", "en"]] as StudiedLanguage[][]) {
      const spanish = await reader(studied, "es");
      expect(await acceptedLanguages(spanish)).toEqual(["en"]);
      expect(await readingLanguage(spanish)).toBe("en");
      expect(readerPairs(await acceptedLanguages(spanish), "es")).toEqual(["en-es"]);
    }
  });
});

describe("the bundle's packs", () => {
  it("lie where the build puts them", () => {
    for (const pair of FRENCH) expect(packPath(pair)).toBe(packFile(pair));
  });

  it("are each pair's layer at the pair's path and each studied language's base once (ship-lingua-pack-bases-once D1)", () => {
    expect(packPath("es-en")).toBe("assets/packs/es-en.lingua");
    expect(basePath("es")).toBe("assets/bases/es.lingua");
    for (const studied of shippedStudied(SHIPPED_PAIRS)) expect(basePath(studied)).toBe(baseFile(studied));
    // One base per studied language the list names, whatever native languages gloss it.
    expect(shippedStudied(SHIPPED_PAIRS)).toEqual(["en", "es", "fr"]);
    expect(new Set(SHIPPED_PAIRS.map(studiedOf))).toEqual(new Set(shippedStudied(SHIPPED_PAIRS)));
  });

  it("are the pairs packs.json lists", () => {
    expect(SHIPPED_PAIRS).toEqual(shippedPairs());
  });
});
