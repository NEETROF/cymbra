// The language pairs the extension ships (generalise-lingua-pack-build): one list, `packs.json`,
// read by every tool through this module — build.mjs, check_variants.mjs, gen_pack.sh and
// make_source_archive.sh. The first pair gives the default studied language: the engine's first
// pack. A pair being built is not a pair being shipped (Spanish tables exist long before Spanish
// ships), so the list is its own file rather than the folders of scripts/lingua-data/tables/.
//
// A package carries each listed pair's pack as two parts (ship-lingua-pack-bases-once D1): the
// pair's layer at the pair's path, `assets/packs/<pair>.lingua`, and its studied language's base,
// once per language, at `assets/bases/<studied>.lingua`; the engine assembles the two at load.
import { existsSync, readFileSync, realpathSync } from "node:fs";
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

/** Where a pair's layer lives — the pair's path —, in the source tree and inside a package alike. */
export function packFile(pair) {
  return `assets/packs/${pair}.lingua`;
}

/** Where a studied language's base lives, in the source tree and inside a package alike. */
export function baseFile(studied) {
  return `assets/bases/${studied}.lingua`;
}

/** The listed pairs' studied languages, once each, in listed order: the bases a package carries. */
export function shippedStudied(pairs = shippedPairs()) {
  return [...new Set(pairs.map(studiedOf))];
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
 * Italian is named since add-lingua-italian-baseline (D4), though no listed pair studies it yet.
 */
export const ANALYZER_CONSTANTS = Object.freeze({
  en: "ANALYZER_VERSION",
  es: "SPANISH_ANALYZER_VERSION",
  fr: "FRENCH_ANALYZER_VERSION",
  it: "ITALIAN_ANALYZER_VERSION",
});

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
 * A container's studied language, native language, analyser version, part and base version, read
 * exactly from it: the JSON at offset 14, whose length is the little-endian u32 at offset 10
 * (generalise-lingua-native-language D8). `part` is "base" or "layer" for a pack's part
 * (split-lingua-pack-format), null for a whole pack; a base names no native language, a layer no
 * analyser version, a whole pack no base version. Each is null when the bytes are not a container
 * whose metadata reads.
 */
export function packMeta(bytes) {
  const none = { studied: null, native: null, analyzerVersion: null, part: null, baseVersion: null };
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
  return {
    studied: text(meta?.studied),
    native: text(meta?.native),
    analyzerVersion: text(meta?.analyzer_version),
    part: text(meta?.part),
    baseVersion: text(meta?.base_version),
  };
}

/** How to rebuild the parts, as every refusal below says it. */
const REBUILD = "`yarn gen:pack:real` (the committed tables) or `yarn gen:pack` (testdata)";

/**
 * Guard: every listed pair's layer is its pair's and was built against the base packaged for its
 * language, and every base shares its language's analyser version with the engine
 * (ship-lingua-pack-bases-once D4). The engine refuses a mismatched part at RUNTIME ("pack built for
 * analyzer X but this core is Y", or a layer of another base), which reads as "the extension is
 * broken" during dogfooding. A bump of a language's analyser version in lingua-core leaves the
 * gitignored real parts (gen:pack:real) stale, so the build fails early with an actionable message
 * instead. lingua-core's constants are the source of truth; each part's container metadata is read,
 * and a part whose metadata does not read, or a whole pack where a part belongs, is refused. The
 * check reads the built parts rather than the tables, because the parts are what ships. CI runs
 * gen:pack (testdata, current versions) before build, so it always matches.
 */
export function assertPacksMatchEngine({
  appDir = APP_DIR,
  pairs = shippedPairs(appDir),
  modRs = readFileSync(join(appDir, "../../crates/lingua-core/src/analysis/mod.rs"), "utf8"),
} = {}) {
  /** A part's metadata, refused when the file is missing, does not read, or is not a `kind` part. */
  const read = (file, kind, fields) => {
    const path = join(appDir, file);
    if (!existsSync(path)) {
      throw new Error(`${file} is missing — run ${REBUILD} before building.`);
    }
    const meta = packMeta(readFileSync(path));
    if (meta.studied && !meta.part) {
      throw new Error(
        `${file} is a whole pack: a package carries each pair's layer under assets/packs/ and each studied language's base under assets/bases/. Rebuild them with ${REBUILD}.`,
      );
    }
    if ((meta.part === "base" || meta.part === "layer") && meta.part !== kind) {
      throw new Error(`${file} is a ${meta.part}, not a ${kind}: rebuild it with ${REBUILD}.`);
    }
    // Every part lingua-core accepts names these (BaseMeta, LayerMeta): one that does not — not a
    // container, a truncated one, metadata that does not parse — would only be refused at runtime.
    if (meta.part !== kind || !fields.every((field) => meta[field])) {
      throw new Error(
        `${file} has no metadata this build can read (its ${fields.join(", ")}): rebuild it with ${REBUILD}.`,
      );
    }
    return meta;
  };
  const bases = new Map();
  for (const language of shippedStudied(pairs)) {
    const file = baseFile(language);
    const base = read(file, "base", ["studied", "analyzerVersion", "baseVersion"]);
    if (base.studied !== language) {
      throw new Error(`${file} studies "${base.studied}", not "${language}": rebuild it with ${REBUILD}.`);
    }
    const core = coreAnalyzerVersion(language, modRs);
    if (!core) {
      throw new Error(
        `lingua-core has no analyser version for "${language}" (${file}): tool/packs.mjs names one per language.`,
      );
    }
    if (base.analyzerVersion !== core) {
      throw new Error(
        `Analyzer version mismatch: ${file} is ${base.analyzerVersion} but lingua-core's ${language} analyser is ${core}. ` +
          `The engine refuses a mismatched pack at runtime. Rebuild it: ${REBUILD}.`,
      );
    }
    bases.set(language, base);
  }
  for (const pair of pairs) {
    const file = packFile(pair);
    const layer = read(file, "layer", ["studied", "native", "baseVersion"]);
    const language = studiedOf(pair);
    if (layer.studied !== language) {
      throw new Error(`${file} studies "${layer.studied}", not "${language}": rebuild it with ${REBUILD}.`);
    }
    // A layer glossed in another language than its pair's would be served to readers of the wrong
    // native language (generalise-lingua-native-language D8).
    if (layer.native !== nativeOf(pair)) {
      throw new Error(`${file} is glossed in "${layer.native}", not "${nativeOf(pair)}": rebuild it with ${REBUILD}.`);
    }
    // A layer and a base out of step: the engine would refuse the pair at load.
    const base = bases.get(language);
    if (layer.baseVersion !== base.baseVersion) {
      throw new Error(
        `${pair}: ${file} was built against the base ${layer.baseVersion}, but ${baseFile(language)} is ${base.baseVersion}: ` +
          `the layer and the base packaged for "${language}" are out of step. Rebuild them with ${REBUILD}.`,
      );
    }
  }
}

// For the shell scripts: `node tool/packs.mjs pairs` prints one shipped pair per line, and
// `node tool/packs.mjs studied` one shipped studied language — one base — per line. Run through a
// symlinked path, Node loads the module by its real path: the script's path is compared resolved.
if (process.argv[1] && fileURLToPath(import.meta.url) === realpathSync(process.argv[1])) {
  if (process.argv[2] === "pairs") {
    console.log(shippedPairs().join("\n"));
  } else if (process.argv[2] === "studied") {
    console.log(shippedStudied().join("\n"));
  } else {
    console.error("usage: node tool/packs.mjs pairs|studied");
    process.exit(2);
  }
}
