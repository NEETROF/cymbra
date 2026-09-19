// The extension's version has one home: package.json, which release-please bumps. build.mjs
// stamps it onto each built manifest, so there is no second file to disagree with.
//
// What is still worth checking is its shape. A browser's manifest version is dot-separated
// integers; SemVer's `1.2.0-rc.1` is refused at upload, after the release has been tagged and
// built. Better to refuse it here, where the answer is "retag", than in a store dialog.
//
// (The mirror this used to compare against was an `extra-files` rule. release-please's JSON
// updater rewrote the whole manifest rather than the one value — it expanded every array —
// and the Prettier gate refused the result. The mirror is gone; so is that failure.)
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const MAX_PART = 65535;

/** Why a store would refuse this version, or null. */
export function versionProblem(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    return `"${version}" is not three plain integers. A browser refuses a pre-release or build suffix in an extension version, so release-please must not produce one for this component.`;
  }
  const parts = version.split(".");
  if (parts.some((part) => Number(part) > MAX_PART)) {
    return `"${version}" has a part above ${MAX_PART}, which a manifest version may not exceed.`;
  }
  if (parts.some((part) => part.length > 1 && part.startsWith("0"))) {
    return `"${version}" has a leading zero, which a manifest version may not have.`;
  }
  return null;
}

/** The source manifest must not carry a version: build.mjs stamps it, and a stale copy would win. */
export function manifestProblem(manifest) {
  return "version" in manifest
    ? `manifest.json carries a version ("${manifest.version}"). It must not: build.mjs stamps package.json's version onto every variant, and a copy here would be a second source that drifts.`
    : null;
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const read = (file) => JSON.parse(readFileSync(join(root, file), "utf8"));
  const version = read("package.json").version;
  const problems = [versionProblem(version), manifestProblem(read("manifest.json"))].filter(Boolean);
  if (problems.length > 0) {
    for (const problem of problems) console.error(`error: ${problem}`);
    process.exit(1);
  }
  console.log(`Version ${version} is publishable, and manifest.json defers to it.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
