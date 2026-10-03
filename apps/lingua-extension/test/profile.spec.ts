import { describe, expect, it } from "vitest";
import { studiedLanguagesOf } from "@/state/profile.ts";

// The studied languages read from a stored backup without an engine
// (generalise-lingua-translation-model-state D2). The layout is the engine's, pinned on its side by
// lingua-core's decks/backup.rs.

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
