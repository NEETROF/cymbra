import { describe, expect, it } from "vitest";
import {
  acceptedLanguages,
  DEFAULT_LANGUAGE,
  DEFAULT_NATIVE,
  defaultPair,
  nativeOf,
  packPath,
  pairFor,
  pairsOf,
  readingLanguage,
  SHIPPED_PAIRS,
} from "@/analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import { packFile, shippedPairs } from "../tool/packs.mjs";
import { makeFakePort } from "./helpers.ts";

/** Today's list beside a pair glossed in English (generalise-lingua-native-language). */
const MIXED = ["en-fr", "es-fr", "es-en"];

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

  it("are, for every reader today, the bundle's whole list", () => {
    expect(DEFAULT_NATIVE).toBe("fr");
    expect(pairsOf(DEFAULT_NATIVE)).toEqual([...SHIPPED_PAIRS]);
    expect(defaultPair(DEFAULT_NATIVE)).toBe(SHIPPED_PAIRS[0]);
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

  it("do not change for any reader of French with today's list", async () => {
    for (const studied of [[], ["en"], ["es"], ["en", "es"], ["es", "en"]] as StudiedLanguage[][]) {
      const french = await reader(studied);
      const expected = studied.length > 0 ? studied : ["en"];
      expect(await acceptedLanguages(french)).toEqual(expected);
      expect(await readingLanguage(french)).toBe(expected[0]);
    }
  });
});

describe("the bundle's packs", () => {
  it("lie where the build puts them", () => {
    for (const pair of ["en-fr", "es-fr"]) expect(packPath(pair)).toBe(packFile(pair));
  });

  it("are the pairs packs.json lists", () => {
    expect(SHIPPED_PAIRS).toEqual(shippedPairs());
  });
});
