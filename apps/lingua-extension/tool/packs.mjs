// The language pairs the extension ships (generalise-lingua-pack-build): one list, `packs.json`,
// read by every tool through this module — build.mjs, check_variants.mjs, gen_pack.sh and
// make_source_archive.sh. The first pair gives the default studied language: the engine's first
// pack. A pair being built is not a pair being shipped (Spanish tables exist long before Spanish
// ships), so the list is its own file rather than the folders of scripts/lingua-data/tables/.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const APP_DIR = join(dirname(fileURLToPath(import.meta.url)), "..");

/** The pairs every package ships, the default language's first. */
export function shippedPairs(appDir = APP_DIR) {
  const { pairs } = JSON.parse(readFileSync(join(appDir, "packs.json"), "utf8"));
  if (!Array.isArray(pairs) || pairs.length === 0 || !pairs.every((p) => /^[a-z]{2}-[a-z]{2}$/.test(p))) {
    throw new Error(`packs.json: "pairs" must list "<studied>-<native>" pairs, got ${JSON.stringify(pairs)}`);
  }
  if (new Set(pairs).size !== pairs.length) throw new Error(`packs.json: a pair is listed twice in ${pairs}`);
  return pairs;
}

/** Where a pair's pack lives, in the source tree and inside a package alike. */
export function packFile(pair) {
  return `assets/packs/${pair}.lingua`;
}

/** The studied side of a pair: "en-fr" → "en". */
export function studiedOf(pair) {
  return pair.split("-")[0];
}

/**
 * The lingua-core constant that holds each studied language's analyser version
 * (generalise-lingua-analysis-by-language). test/packs.spec.ts holds it to
 * crates/lingua-core/src/analysis/language.rs, so a language added there fails here first.
 */
export const ANALYZER_CONSTANTS = Object.freeze({ en: "ANALYZER_VERSION", es: "SPANISH_ANALYZER_VERSION" });

/** lingua-core's analyser version for `language`, read from the source of analysis/mod.rs. */
export function coreAnalyzerVersion(language, modRs) {
  const name = ANALYZER_CONSTANTS[language];
  if (!name) return null;
  // `\b`: SPANISH_ANALYZER_VERSION must never be read as ANALYZER_VERSION.
  return modRs.match(new RegExp(`\\b${name}:\\s*&str\\s*=\\s*"([^"]+)"`))?.[1] ?? null;
}

/** A pack's studied language and analyser version, scanned from its container's ASCII metadata. */
export function packMeta(bytes) {
  // latin1 keeps the binary intact while the ASCII meta JSON stays matchable.
  const text = Buffer.from(bytes).toString("latin1");
  return {
    studied: text.match(/"studied"\s*:\s*"([^"]+)"/)?.[1] ?? null,
    analyzerVersion: text.match(/"analyzer_version"\s*:\s*"([^"]+)"/)?.[1] ?? null,
  };
}

// For the shell scripts: `node tool/packs.mjs pairs` prints one shipped pair per line.
if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  if (process.argv[2] === "pairs") {
    console.log(shippedPairs().join("\n"));
  } else {
    console.error("usage: node tool/packs.mjs pairs");
    process.exit(2);
  }
}
