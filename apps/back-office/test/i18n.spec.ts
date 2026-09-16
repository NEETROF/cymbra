import { describe, expect, it } from "vitest";
import en from "@/i18n/locales/en.json";
import fr from "@/i18n/locales/fr.json";
import { SCOPES } from "@/lib/jwt";
import { MANAGED_ROLES } from "@/lib/roles";

// Flatten a message catalogue into sorted dotted key paths.
function keys(obj: Record<string, unknown>, prefix = ""): string[] {
  return Object.entries(obj)
    .flatMap(([k, v]) => {
      const path = prefix ? `${prefix}.${k}` : k;
      return v && typeof v === "object" ? keys(v as Record<string, unknown>, path) : [path];
    })
    .sort();
}

// The TS schema (`fr: MessageSchema`) already fails compilation on a MISSING key;
// this also catches EXTRA/renamed keys at runtime, and covers every locale we add.
describe("i18n locales", () => {
  const base = keys(en);
  const others: Record<string, Record<string, unknown>> = { fr };

  for (const [name, msgs] of Object.entries(others)) {
    it(`"${name}" has exactly the English key set`, () => {
      expect(keys(msgs)).toEqual(base);
    });
  }
});

// A scope with no label renders its RAW KEY: the roles panel showed "SCOPE.LINGUA"
// because `lingua` was missing from BOTH locales — which the parity test above cannot
// see, since it only compares the locales with each other. The token's scope list is
// the reference: every scope it can carry needs a label in every locale.
describe("scope labels", () => {
  const catalogues: Record<string, { scope: Record<string, string> }> = {
    en: en as unknown as { scope: Record<string, string> },
    fr: fr as unknown as { scope: Record<string, string> },
  };

  for (const [name, msgs] of Object.entries(catalogues)) {
    it(`"${name}" labels every scope a token can carry`, () => {
      expect(Object.keys(msgs.scope).sort()).toEqual([...SCOPES].sort());
    });
  }
});

// Roles are NOT a closed set: the server stores whatever `grant_role` was given (it
// guards the scope, never the role), so the catalogues cannot be held to the full list
// the way scopes are — an unlabelled one falls back to its own name (role-label.spec.ts).
// What IS closed is the set this console grants, so those must all be labelled.
describe("role labels", () => {
  const catalogues: Record<string, { role: Record<string, string> }> = {
    en: en as unknown as { role: Record<string, string> },
    fr: fr as unknown as { role: Record<string, string> },
  };

  for (const [name, msgs] of Object.entries(catalogues)) {
    it(`"${name}" labels every role the console can grant`, () => {
      const labelled = Object.keys(msgs.role);
      expect(MANAGED_ROLES.filter((r) => !labelled.includes(r))).toEqual([]);
    });
  }
});
