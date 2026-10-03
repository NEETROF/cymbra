import { describe, expect, it } from "vitest";
import { packPath, pairFor, SHIPPED_PAIRS } from "@/analyzer/pairs.ts";
import { packFile, shippedPairs } from "../tool/packs.mjs";

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

describe("the bundle's packs", () => {
  it("lie where the build puts them", () => {
    for (const pair of ["en-fr", "es-fr"]) expect(packPath(pair)).toBe(packFile(pair));
  });

  it("are the pairs packs.json lists", () => {
    expect(SHIPPED_PAIRS).toEqual(shippedPairs());
  });
});
