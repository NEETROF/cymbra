import { describe, expect, it } from "vitest";
import {
  acceptedLanguages,
  DEFAULT_LANGUAGE,
  packPath,
  pairFor,
  readingLanguage,
  SHIPPED_PAIRS,
} from "@/analyzer/pairs.ts";
import type { StudiedLanguage } from "@/analyzer/types.ts";
import { packFile, shippedPairs } from "../tool/packs.mjs";
import { makeFakePort } from "./helpers.ts";

describe("the pair that serves a language", () => {
  it("is the first listed pair that studies it", () => {
    expect(pairFor("en", ["en-fr", "es-fr"])).toBe("en-fr");
    expect(pairFor("es", ["en-fr", "es-fr"])).toBe("es-fr");
    expect(pairFor("en", ["en-de", "en-fr"])).toBe("en-de");
  });

  it("is none for a language no listed pair studies", () => {
    expect(pairFor("es", ["en-fr"])).toBeNull();
    expect(pairFor("fr", ["en-fr"])).toBeNull(); // the native side is not studied
    expect(pairFor("", ["en-fr"])).toBeNull();
  });

  it("is read from the bundle's list by default", () => {
    expect(pairFor("en")).toBe("en-fr");
  });
});

describe("the language a surface reads in", () => {
  const reader = async (studied: StudiedLanguage[]) => {
    const { port } = makeFakePort();
    await port.setStudiedLanguages(studied);
    return port;
  };

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

  it("only reads the reader's studied languages", async () => {
    const port = await reader(["es", "en"]);
    await readingLanguage(port, ["en-fr"]);
    expect(await port.studiedLanguages()).toEqual(["es", "en"]);
  });
});

describe("the languages a device accepts from the sync", () => {
  const reader = async (studied: StudiedLanguage[]) => {
    const { port } = makeFakePort();
    await port.setStudiedLanguages(studied);
    return port;
  };

  it("are the reader's shipped languages, in the reader's order", async () => {
    expect(await acceptedLanguages(await reader(["es", "en"]), ["en-fr", "es-fr"])).toEqual(["es", "en"]);
    expect(await acceptedLanguages(await reader(["es", "en"]), ["en-fr"])).toEqual(["en"]);
  });

  it("are the default pair's alone when none is shipped, and English for every reader today", async () => {
    expect(await acceptedLanguages(await reader(["es"]), ["en-fr"])).toEqual(["en"]);
    expect(await acceptedLanguages(await reader(["en"]))).toEqual(["en"]);
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
