import { describe, expect, it } from "vitest";
import en from "@/i18n/locales/en.json";
import { MANAGED_ROLES, roleLabel } from "@/lib/roles";

// The catalogue as vue-i18n sees it: `te` answers "is there a message for this key?",
// `t` returns it (and echoes the key back when there is none — the behaviour that put
// "SCOPE.LINGUA" on screen in production).
const labels = en.role as Record<string, string>;
const has = (key: string) => key.startsWith("role.") && key.slice("role.".length) in labels;
const translate = (key: string) => labels[key.slice("role.".length)] ?? key;

describe("roleLabel", () => {
  it("uses the catalogue label when the role has one", () => {
    expect(roleLabel("moderator", has, translate)).toBe(labels.moderator);
  });

  it("falls back to the role's own name when it has none", () => {
    // A role the backend grew without the console knowing: readable, not a raw key.
    expect(roleLabel("reviewer", has, translate)).toBe("reviewer");
  });

  it("never lets a message key reach the screen", () => {
    for (const role of [...MANAGED_ROLES, "user", "reviewer", "support_agent", ""]) {
      expect(roleLabel(role, has, translate)).not.toMatch(/^role\./);
    }
  });

  it("labels every role this console can grant", () => {
    for (const role of MANAGED_ROLES) {
      expect(roleLabel(role, has, translate)).toBe(labels[role]);
    }
  });
});
