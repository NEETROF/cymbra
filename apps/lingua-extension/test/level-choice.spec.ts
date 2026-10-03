import { describe, expect, it } from "vitest";
import type { DeclaredLevelOp, LanguagePort } from "@/analyzer/port.ts";
import type { CefrLevel, StudiedLanguage } from "@/analyzer/types.ts";
import { needsLevelChoice } from "@/state/level-choice.ts";

/** The languages a test port's view was asked in. */
const asked: StudiedLanguage[] = [];

function port(opts: { hasLevels: boolean; declared: CefrLevel | null; decisions: DeclaredLevelOp[] }) {
  const view = { hasLevels: async () => opts.hasLevels, declaredLevel: async () => opts.declared };
  return {
    for: (language: StudiedLanguage) => {
      asked.push(language);
      return view as unknown as LanguagePort;
    },
    exportDeclaredLevels: async () => opts.decisions,
  };
}

describe("needsLevelChoice", () => {
  it("asks when the language has CEFR data and no decision was ever made", async () => {
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions: [] }), "en")).toBe(true);
  });

  it("does not ask again once « Débutant » was chosen (no level, but a stamped decision)", async () => {
    const decisions = [{ language: "en", level: "", updated_at: 1_726_300_000_000 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions }), "en")).toBe(false);
  });

  it("does not ask when a level is declared", async () => {
    const decisions = [{ language: "en", level: "B2", updated_at: 1_726_300_000_000 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: "B2", decisions }), "en")).toBe(false);
  });

  it("does not ask for a language without CEFR data (the frequency slider applies)", async () => {
    expect(await needsLevelChoice(port({ hasLevels: false, declared: null, decisions: [] }), "en")).toBe(false);
  });

  it("asks the language view in the language it is given", async () => {
    asked.length = 0;
    await needsLevelChoice(port({ hasLevels: true, declared: null, decisions: [] }), "es");
    expect(asked).toEqual(["es"]);
  });

  it("ignores an unstamped row, which records no decision", async () => {
    const decisions = [{ language: "en", level: "", updated_at: 0 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions }), "en")).toBe(true);
  });
});

describe("needsLevelChoice, per language (add-lingua-language-choice D3)", () => {
  it("does not take a Spanish decision for an English one", async () => {
    const decisions = [{ language: "es", level: "A2", updated_at: 1_726_300_000_000 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions }), "en")).toBe(true);
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions }), "es")).toBe(false);
  });
});
