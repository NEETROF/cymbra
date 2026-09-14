import { describe, expect, it } from "vitest";
import type { DeclaredLevelOp } from "@/analyzer/port.ts";
import type { CefrLevel } from "@/analyzer/types.ts";
import { needsLevelChoice } from "@/state/level-choice.ts";

function port(opts: { hasLevels: boolean; declared: CefrLevel | null; decisions: DeclaredLevelOp[] }) {
  return {
    hasLevels: async () => opts.hasLevels,
    declaredLevel: async () => opts.declared,
    exportDeclaredLevels: async () => opts.decisions,
  };
}

describe("needsLevelChoice", () => {
  it("asks when the language has CEFR data and no decision was ever made", async () => {
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions: [] }))).toBe(true);
  });

  it("does not ask again once « Débutant » was chosen (no level, but a stamped decision)", async () => {
    const decisions = [{ language: "en", level: "", updated_at: 1_726_300_000_000 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions }))).toBe(false);
  });

  it("does not ask when a level is declared", async () => {
    const decisions = [{ language: "en", level: "B2", updated_at: 1_726_300_000_000 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: "B2", decisions }))).toBe(false);
  });

  it("does not ask for a language without CEFR data (the frequency slider applies)", async () => {
    expect(await needsLevelChoice(port({ hasLevels: false, declared: null, decisions: [] }))).toBe(false);
  });

  it("ignores an unstamped row, which records no decision", async () => {
    const decisions = [{ language: "en", level: "", updated_at: 0 }];
    expect(await needsLevelChoice(port({ hasLevels: true, declared: null, decisions }))).toBe(true);
  });
});
