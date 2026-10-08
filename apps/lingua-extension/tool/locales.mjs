// The committed `_locales` (localise-lingua-manifest D1): one messages.json per native language a
// shipped pair may be glossed in — the extension's description and its commands' descriptions, which
// the browser reads in its own language once a pair glossed in that language ships. Read here for
// the build (build.mjs), the version check (check_version.mjs) and the variant check
// (check_variants.mjs); tool/manifests.mjs decides what a package carries and stays pure.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

/** The folder the browsers read: `_locales/<language>/messages.json`, in the source and in a package alike. */
export const LOCALES_DIR = "_locales";

/** Where a language's messages live, relative to the source tree or to a package. */
export function messagesFile(language) {
  return `${LOCALES_DIR}/${language}/messages.json`;
}

/**
 * Every language under `dir`'s `_locales`, its messages parsed, by language in name order — `{}` when
 * the folder does not exist (a package built while only French-native pairs ship carries none).
 * @param {string} dir the source tree, or a package
 * @returns {Record<string, Record<string, { message: string, description?: string }>>}
 */
export function readLocales(dir) {
  const root = join(dir, LOCALES_DIR);
  if (!existsSync(root)) return {};
  const languages = readdirSync(root, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
  return Object.fromEntries(
    languages.map((language) => [language, JSON.parse(readFileSync(join(dir, messagesFile(language)), "utf8"))]),
  );
}
