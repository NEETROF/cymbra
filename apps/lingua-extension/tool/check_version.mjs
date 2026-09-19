// The extension's version is one value living in two files: package.json, which
// release-please bumps, and manifest.json, which the browser reads and which build.mjs
// folds into all three variants. release-please writes both (`extra-files` in
// release-please-config.json), so a disagreement means one of them was edited by hand —
// and the failure it causes is silent: a package whose version the changelog never mentions.
//
// The shape matters too. A browser's manifest version is dot-separated integers; SemVer's
// `1.2.0-rc.1` is refused at upload, after the release has been tagged and built. Better to
// refuse it here, where the answer is "retag", than in a store dialog.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const MAX_PART = 65535;

/** What is wrong with a version on its own, or null. */
function shapeProblem(version) {
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

/** Everything wrong with the pair, as sentences. Empty means the release can proceed. */
export function versionProblems(packageVersion, manifestVersion) {
  const problems = [];
  if (packageVersion !== manifestVersion) {
    problems.push(
      `package.json says "${packageVersion}" and manifest.json says "${manifestVersion}". ` +
        `release-please writes both, so one of them was edited by hand.`,
    );
  }
  // A set, so an identical bad version is reported once rather than twice.
  for (const version of new Set([packageVersion, manifestVersion])) {
    const problem = shapeProblem(version);
    if (problem) problems.push(problem);
  }
  return problems;
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const read = (file) => JSON.parse(readFileSync(join(root, file), "utf8")).version;
  const problems = versionProblems(read("package.json"), read("manifest.json"));
  if (problems.length > 0) {
    for (const problem of problems) console.error(`error: ${problem}`);
    process.exit(1);
  }
  console.log(`Version ${read("package.json")} is consistent and publishable.`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
