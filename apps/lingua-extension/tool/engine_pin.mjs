// The translation engine's pin (add-lingua-translation-delivery D9): which upstream commit it is
// built from, and the sha256 of every file the extension packages. One check, used by everyone
// who touches the engine — build.mjs before packaging it, tool/fetch_engine.sh after downloading
// it, tool/build_engine.sh after compiling it, and check_variants.mjs on what was packaged.
//
// The pin holds because the build is reproducible: three runs of lingua-engine-build a day apart
// produced byte-identical files. An engine that does not match was built from something else.
//
// CLI: node tool/engine_pin.mjs <dir>          exits 1, naming each problem, unless <dir> matches.
//      node tool/engine_pin.mjs --release-tag  prints the GitHub Release that keeps the pinned engine.
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

/** engine-pin.json: `{ translationsCommit, files: { name: sha256 } }`. */
export function enginePin() {
  return JSON.parse(readFileSync(join(appRoot, "engine-pin.json"), "utf8"));
}

/** The files the extension packages, in the order they are copied. */
export function engineFiles() {
  return Object.keys(enginePin().files);
}

/**
 * The GitHub Release that keeps the pinned engine for good (lingua-engine-build publishes it):
 * artefacts expire after 90 days, a release asset does not, and the packages must still build the
 * day upstream can no longer be rebuilt.
 */
export function engineReleaseTag() {
  return `lingua-engine-${enginePin().translationsCommit.slice(0, 12)}`;
}

export function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

/**
 * What is wrong with the engine in `dir`, or nothing. A `TRANSLATIONS_COMMIT` file, when present
 * (the workflow's artefact carries one), must name the pinned commit.
 */
export function engineProblems(dir) {
  const pin = enginePin();
  const problems = [];
  for (const [name, want] of Object.entries(pin.files)) {
    const path = join(dir, name);
    if (!existsSync(path)) {
      problems.push(`${name} is missing`);
      continue;
    }
    // The actual hash is named, so moving the pin to a new upstream commit is one build away.
    const got = sha256(path);
    if (got !== want) problems.push(`${name} is not the pinned build (sha256 ${got}, pinned ${want})`);
  }
  const commitFile = join(dir, "TRANSLATIONS_COMMIT");
  if (existsSync(commitFile)) {
    const commit = readFileSync(commitFile, "utf8").trim();
    if (commit !== pin.translationsCommit) {
      problems.push(`built from ${commit}, not the pinned ${pin.translationsCommit}`);
    }
  }
  return problems;
}

if (process.argv[1] === fileURLToPath(import.meta.url) && process.argv[2] === "--release-tag") {
  console.log(engineReleaseTag());
} else if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const dir = process.argv[2];
  if (!dir) {
    console.error("usage: node tool/engine_pin.mjs <engine-dir>");
    process.exit(2);
  }
  const problems = engineProblems(dir);
  if (problems.length > 0) {
    console.error(`${dir}: ${problems.join("; ")}`);
    process.exit(1);
  }
  console.log(`${dir}: the pinned engine (mozilla/translations ${enginePin().translationsCommit}).`);
}
