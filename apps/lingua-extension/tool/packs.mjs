// The language pairs the extension ships (generalise-lingua-pack-build): one list, `packs.json`,
// read by every tool through this module — build.mjs, check_variants.mjs, gen_pack.sh and
// make_source_archive.sh. The first pair gives the default studied language: the engine's first
// pack. A pair being built is not a pair being shipped (Spanish tables exist long before Spanish
// ships), so the list is its own file rather than the folders of scripts/lingua-data/tables/.
import { existsSync, readFileSync } from "node:fs";
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
  // A reader never studies their native language (generalise-lingua-native-language D8).
  const same = pairs.find((pair) => studiedOf(pair) === nativeOf(pair));
  if (same) throw new Error(`packs.json: ${same} would be glossed in the language it studies`);
  return pairs;
}

/** Where a pair's pack lives, in the source tree and inside a package alike. */
export function packFile(pair) {
  return `assets/packs/${pair}.lingua`;
}

// A pair's name splits at its first `-`, as src/analyzer/pairs.ts splits it: test/pairs.spec.ts
// holds the two readings equal.

/** The studied side of a pair: "en-fr" → "en". */
export function studiedOf(pair) {
  const dash = pair.indexOf("-");
  return dash < 0 ? pair : pair.slice(0, dash);
}

/** The native side of a pair, the language its glosses are written in: "en-fr" → "fr"; "" without one. */
export function nativeOf(pair) {
  const dash = pair.indexOf("-");
  return dash < 0 ? "" : pair.slice(dash + 1);
}

/**
 * The native languages with at least one shipped pair, in listed order, once each
 * (add-lingua-native-language-choice D1): the languages a package's manifest speaks
 * (tool/manifests.mjs). src/analyzer/pairs.ts holds the same helper for the bundle; test/pairs.spec.ts
 * holds the two equal.
 */
export function shippedNatives(pairs = shippedPairs()) {
  return [...new Set(pairs.map(nativeOf))];
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

/** The container's magic (crates/lingua-core/src/packs/format.rs). */
const MAGIC = "LINGUAPK";
/** Where the metadata's length sits: after the magic and the u16 format version. */
const META_LENGTH_AT = 10;
/** Where the metadata's JSON starts: after its u32 length. */
const META_AT = 14;

/**
 * A pack's studied language, native language and analyser version, read exactly from its
 * container: the JSON at offset 14, whose length is the little-endian u32 at offset 10
 * (generalise-lingua-native-language D8). Each is null when the bytes are not a container whose
 * metadata reads.
 */
export function packMeta(bytes) {
  const none = { studied: null, native: null, analyzerVersion: null };
  const buf = Buffer.from(bytes);
  if (buf.length < META_AT || buf.toString("latin1", 0, MAGIC.length) !== MAGIC) return none;
  const end = META_AT + buf.readUInt32LE(META_LENGTH_AT);
  if (end > buf.length) return none;
  let meta;
  try {
    meta = JSON.parse(buf.toString("utf8", META_AT, end));
  } catch {
    return none;
  }
  const text = (value) => (typeof value === "string" ? value : null);
  return { studied: text(meta?.studied), native: text(meta?.native), analyzerVersion: text(meta?.analyzer_version) };
}

/**
 * Guard: every listed pack is its pair's, and shares its language's analyzer version with the
 * engine. The engine refuses a mismatched pack at RUNTIME ("pack built for analyzer X but this core
 * is Y"), which reads as "the extension is broken" during dogfooding. A bump of a language's
 * analyser version in lingua-core leaves the gitignored real pack (gen:pack:real) stale, so the
 * build fails early with an actionable message instead. lingua-core's constants are the source of
 * truth; each pack's container metadata is read for its studied language, its native language and
 * its analyser version, and a pack whose metadata does not read is refused. The check reads the
 * built pack rather than the tables, because the pack is what ships. CI runs gen:pack (testdata, current versions) before build, so it always matches.
 */
export function assertPacksMatchEngine({
  appDir = APP_DIR,
  pairs = shippedPairs(appDir),
  modRs = readFileSync(join(appDir, "../../crates/lingua-core/src/analysis/mod.rs"), "utf8"),
} = {}) {
  for (const pair of pairs) {
    const file = packFile(pair);
    const path = join(appDir, file);
    if (!existsSync(path)) {
      throw new Error(
        `${file} is missing — run \`yarn gen:pack\` (testdata) or \`yarn gen:pack:real\` (the committed tables) before building.`,
      );
    }
    const language = studiedOf(pair);
    const { studied, native, analyzerVersion } = packMeta(readFileSync(path));
    // Every pack lingua-core accepts names all three (PackMeta): one that does not — not a
    // container, a truncated one, metadata that does not parse — would only be refused at runtime.
    if (!studied || !native || !analyzerVersion) {
      throw new Error(
        `${file} has no metadata this build can read (its studied language, native language and analyser version): rebuild it with \`yarn gen:pack:real\` (the committed tables) or \`yarn gen:pack\` (testdata).`,
      );
    }
    if (studied !== language) {
      throw new Error(`${file} studies "${studied}", not "${language}": rebuild it with \`yarn gen:pack:real\`.`);
    }
    // A pack glossed in another language than its pair's would be served to readers of the wrong
    // native language (generalise-lingua-native-language D8).
    if (native !== nativeOf(pair)) {
      throw new Error(
        `${file} is glossed in "${native}", not "${nativeOf(pair)}": rebuild it with \`yarn gen:pack:real\` (the committed tables) or \`yarn gen:pack\` (testdata).`,
      );
    }
    const core = coreAnalyzerVersion(language, modRs);
    if (!core) {
      throw new Error(
        `lingua-core has no analyser version for "${language}" (${pair}): tool/packs.mjs names one per language.`,
      );
    }
    if (analyzerVersion !== core) {
      throw new Error(
        `Analyzer version mismatch: ${file} is ${analyzerVersion} but lingua-core's ${language} analyser is ${core}. ` +
          "The engine refuses a mismatched pack at runtime. Rebuild it: " +
          "`yarn gen:pack:real` (the committed tables) or `yarn gen:pack` (testdata).",
      );
    }
  }
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
