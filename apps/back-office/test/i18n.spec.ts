import { describe, expect, it } from "vitest";
import en from "@/i18n/locales/en.json";
import fr from "@/i18n/locales/fr.json";

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
