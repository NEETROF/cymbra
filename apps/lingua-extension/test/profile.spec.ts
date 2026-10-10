import { describe, expect, it } from "vitest";
import { nativeLanguageOf, studiedLanguagesOf } from "@/state/profile.ts";

// The studied languages (generalise-lingua-translation-model-state D2) and the native language
// (generalise-lingua-native-language D7) read from a stored backup without an engine. The layout is
// the engine's, pinned on its side by lingua-core's decks/backup.rs.

/** A version 2 backup as the engine writes it, cut to what matters here. */
const version2 = (studied: unknown) =>
  JSON.stringify({ schema_version: 2, profile: { native_language: "French", studied_languages: studied } }, null, 2);

describe("the studied languages of a stored backup", () => {
  it("reads the engine's names, in the reader's order", () => {
    expect(studiedLanguagesOf(version2(["Spanish", "English"]))).toEqual(["es", "en"]);
  });

  it("reads a backup without a profile as English — every reader of English alone", () => {
    expect(studiedLanguagesOf(JSON.stringify({ schema_version: 1, knowledge: {} }))).toEqual(["en"]);
  });

  it("leaves out a language this build does not know, and falls back on English", () => {
    expect(studiedLanguagesOf(version2(["Portuguese", "Spanish"]))).toEqual(["es"]);
    expect(studiedLanguagesOf(version2(["Portuguese"]))).toEqual(["en"]);
    expect(studiedLanguagesOf(version2("Spanish"))).toEqual(["en"]);
  });

  it("reads anything unreadable as English", () => {
    expect(studiedLanguagesOf("not json")).toEqual(["en"]);
    expect(studiedLanguagesOf("null")).toEqual(["en"]);
  });
});

describe("the native language of a stored backup", () => {
  /** A version 2 backup naming `native`, as the engine writes it, cut to what matters here. */
  const native = (name: unknown) =>
    JSON.stringify({ schema_version: 2, profile: { native_language: name, studied_languages: ["Spanish"] } }, null, 2);
  /** The French-native pairs beside pairs glossed in English and in Spanish: the bundle's list since change 35. */
  const MIXED = ["en-fr", "es-fr", "es-en", "en-es"];

  it("reads the engine's names", () => {
    expect(nativeLanguageOf(native("English"), MIXED)).toBe("en");
    expect(nativeLanguageOf(native("Spanish"), MIXED)).toBe("es");
    expect(nativeLanguageOf(native("French"), MIXED)).toBe("fr");
  });

  it("is French for a backup without a profile — every installed reader", () => {
    expect(nativeLanguageOf(JSON.stringify({ schema_version: 1, knowledge: {} }), MIXED)).toBe("fr");
    expect(nativeLanguageOf(JSON.stringify({ schema_version: 2, profile: {} }), MIXED)).toBe("fr");
  });

  it("is French for a name this build does not know, or no name at all", () => {
    for (const name of ["Italian", "Portuguese", "english", "en", "toString", "", 3, null, ["English"]]) {
      expect(nativeLanguageOf(native(name), MIXED)).toBe("fr");
    }
  });

  it("is French for anything unreadable", () => {
    expect(nativeLanguageOf("not json", MIXED)).toBe("fr");
    expect(nativeLanguageOf("null", MIXED)).toBe("fr");
  });

  it("is French for a native language no listed pair is glossed in", () => {
    expect(nativeLanguageOf(native("English"), ["en-fr", "es-fr"])).toBe("fr");
    expect(nativeLanguageOf(native("Spanish"), ["en-fr", "es-en"])).toBe("fr");
    // Change 34's list (enable-lingua-english-speakers): es-en is glossed in English, no pair in Spanish.
    expect(nativeLanguageOf(native("English"), ["en-fr", "es-fr", "es-en"])).toBe("en");
    expect(nativeLanguageOf(native("Spanish"), ["en-fr", "es-fr", "es-en"])).toBe("fr");
  });

  it("is each stored native language with the bundle's list, which glosses in all three (enable-lingua-spanish-speakers)", () => {
    expect(nativeLanguageOf(native("French"))).toBe("fr");
    expect(nativeLanguageOf(native("English"))).toBe("en");
    expect(nativeLanguageOf(native("Spanish"))).toBe("es");
  });
});
