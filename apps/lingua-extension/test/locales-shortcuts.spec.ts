import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { readLocales } from "../tool/locales.mjs";
import { settings as en } from "../src/i18n/en/settings";
import { settings as es } from "../src/i18n/es/settings";
import { settings as fr } from "../src/i18n/fr/settings";
import { settings as it_ } from "../src/i18n/it/settings";

// The browser's shortcuts page and Réglages name one command alike (localise-lingua-manifest-italian
// D1, D3): each Réglages shortcut line's text after the keys is contained, whatever its case, in the
// matching `_locales/<language>` command's message.

const messages = readLocales(join(__dirname, ".."));
const SETTINGS = { fr, en, es, it: it_ } as const;
const KEYS = "\u0000KEYS\u0000";

/** A Réglages line's words after the keys, without the separator (« — » or « : ») between them. */
function words(line: (keys: string) => string): string {
  const after = line(KEYS).split(KEYS)[1] ?? "";
  return after.replace(/^\s*[—:]\s*/u, "").trim();
}

const COMMANDS = [
  ["commandCaptureSelection", "shortcutCapture"],
  ["commandToggleDrawer", "shortcutDrawer"],
  ["commandSidePanel", "shortcutSidePanel"],
] as const;

describe("the shortcuts page and Réglages name each command alike", () => {
  it("commits a folder for every interface language this test reads", () => {
    expect(Object.keys(messages).sort()).toEqual(Object.keys(SETTINGS).sort());
  });

  for (const [language, own] of Object.entries(SETTINGS)) {
    it.each(COMMANDS)(`${language}: %s contains Réglages' %s words`, (command, line) => {
      const expected = words(own[line]);
      expect(expected.length).toBeGreaterThan(0);
      const message = messages[language]?.[command]?.message ?? "";
      expect(
        message.toLocaleLowerCase(language).includes(expected.toLocaleLowerCase(language)),
        `_locales/${language} ${command} « ${message} » does not contain Réglages' « ${expected} »`,
      ).toBe(true);
    });
  }
});
