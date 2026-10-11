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
import { messagesFile, readLocales } from "./locales.mjs";
import { COMMAND_MESSAGES, DESCRIPTION_MESSAGE, LITERAL_NATIVE } from "./manifests.mjs";
import { nativeOf, shippedNatives, shippedPairs, studiedOf } from "./packs.mjs";

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

// Apple validates the bundled Safari extension's manifest when the archive is UPLOADED, so a
// description one character too long costs a full signed build, an upload, and a failed
// release — which is how this limit was found (lingua-apple-v1.1.0). Apple's 112 is stricter
// than Chrome's 132, so 112 is the one that binds.
const MAX_DESCRIPTION = 112;

/** Everything wrong with the source manifest, as sentences. */
export function manifestProblems(manifest) {
  const problems = [];
  if ("version" in manifest) {
    problems.push(
      `manifest.json carries a version ("${manifest.version}"). It must not: build.mjs stamps package.json's version onto every variant, and a copy here would be a second source that drifts.`,
    );
  }
  const { description } = manifest;
  if (typeof description !== "string" || description.length === 0) {
    problems.push("manifest.json has no description. Apple refuses the upload without one, and both stores show it.");
  } else if (description.length > MAX_DESCRIPTION) {
    problems.push(
      `the description is ${description.length} characters; Apple refuses more than ${MAX_DESCRIPTION} when the archive is uploaded (Chrome allows 132, so Apple's is the limit that binds).`,
    );
  }
  return problems;
}

/**
 * Everything wrong with the committed `_locales` (localise-lingua-manifest D3), as sentences. Every
 * language's description is what a browser in that language shows and both stores list once a pair
 * glossed in it ships, so each is held to Apple's limit, named by its language. The French has two
 * homes — manifest.json's literals, which every package built today carries, and `_locales/fr`,
 * which a localised package reads — and they are held equal, key by key.
 */
export function localeProblems(manifest, locales) {
  const problems = [];
  for (const [language, messages] of Object.entries(locales)) {
    const description = messages[DESCRIPTION_MESSAGE]?.message;
    if (typeof description !== "string" || description.length === 0) {
      problems.push(
        `${messagesFile(language)} has no "${DESCRIPTION_MESSAGE}" message: it is what a browser in ${language} shows, and both stores list.`,
      );
    } else if (description.length > MAX_DESCRIPTION) {
      problems.push(
        `the ${language} description (${messagesFile(language)}) is ${description.length} characters; Apple refuses more than ${MAX_DESCRIPTION} when the archive is uploaded (Chrome allows 132, so Apple's is the limit that binds).`,
      );
    }
  }
  const french = locales[LITERAL_NATIVE];
  if (!french) {
    problems.push(
      `${messagesFile(LITERAL_NATIVE)} is missing: a localised package reads the French from it, so it must hold manifest.json's literal text.`,
    );
    return problems;
  }
  const literals = [
    [DESCRIPTION_MESSAGE, manifest.description],
    ...Object.entries(manifest.commands ?? {}).map(([id, command]) => [COMMAND_MESSAGES[id], command?.description]),
  ];
  for (const [key, literal] of literals) {
    // A command with no message of its own is refused by the build (tool/manifests.mjs), not here.
    if (!key) continue;
    const message = french[key]?.message;
    if (message !== literal) {
      problems.push(
        `${messagesFile(LITERAL_NATIVE)}'s "${key}" reads ${JSON.stringify(message)} and manifest.json's literal reads ${JSON.stringify(literal)}: the French has two homes, held equal.`,
      );
    }
  }
  return problems;
}

/**
 * How each native language names the languages a pair may study, as the interface's catalogue
 * names them (`name` in src/i18n/{fr,en,es}/languages.ts; test/check-version.spec.ts holds the two
 * equal). The description is read in its own language: « anglais » is English in French, "English"
 * in English, « inglés » in Spanish.
 */
export const LANGUAGE_NAMES = Object.freeze({
  fr: Object.freeze({ en: "Anglais", es: "Espagnol", fr: "Français" }),
  en: Object.freeze({ en: "English", es: "Spanish", fr: "French" }),
  es: Object.freeze({ en: "Inglés", es: "Español", fr: "Francés" }),
});

/**
 * The languages `text` names in `native`'s words, as codes in the table's order: each name matched
 * whole — no letter on either side, so « l'anglais » names English and "Spanishness" names nothing —
 * and whatever its case.
 */
export function namedLanguages(native, text) {
  const names = LANGUAGE_NAMES[native] ?? {};
  return Object.entries(names)
    .filter(([, name]) => new RegExp(`(?<!\\p{L})${name}(?!\\p{L})`, "iu").test(text))
    .map(([code]) => code);
}

/**
 * Everything wrong with what the stores' summaries name (add-lingua-french-listings D6), as
 * sentences. For each native language a shipped pair is glossed in, its description — the one a
 * browser in that language shows and both stores list as the summary: `_locales/<native>`, and for
 * French manifest.json's literal too — names every language its shipped pairs study, and no other.
 * A package that lists fr-en under "Read Spanish on the web" fails here, and so does a summary that
 * names French before a pair glossed in its language studies it. A language no shipped pair is glossed
 * in is not read: its readers are not offered the package's languages yet.
 */
export function summaryProblems(manifest, locales, pairs) {
  const problems = [];
  for (const native of shippedNatives(pairs)) {
    const names = LANGUAGE_NAMES[native];
    const glossed = pairs.filter((pair) => nativeOf(pair) === native);
    if (!names) {
      problems.push(
        `${glossed.join(", ")} ship glossed in "${native}", whose names for the studied languages tool/check_version.mjs does not hold (LANGUAGE_NAMES): add them, as the interface's catalogue names them.`,
      );
      continue;
    }
    const studied = [...new Set(glossed.map(studiedOf))];
    const listed = (codes) =>
      codes.length === 0 ? "no language" : joinNames(codes.map((code) => names[code] ?? `"${code}"`));
    const descriptions = [[messagesFile(native), locales[native]?.[DESCRIPTION_MESSAGE]?.message]];
    if (native === LITERAL_NATIVE) descriptions.push(["manifest.json", manifest.description]);
    for (const [file, description] of descriptions) {
      if (typeof description !== "string") {
        problems.push(
          `the ${native} description (${file}) is missing, yet ${glossed.join(", ")} ship glossed in ${native}: it is the summary both stores list, and must name ${listed(studied)}.`,
        );
        continue;
      }
      const found = namedLanguages(native, description);
      const missing = studied.filter((code) => !found.includes(code));
      const extra = found.filter((code) => !studied.includes(code));
      if (missing.length === 0 && extra.length === 0) continue;
      const why = [
        ...(missing.length ? [`it does not name ${listed(missing)}`] : []),
        ...(extra.length ? [`it names ${listed(extra)}, which no shipped pair glossed in ${native} studies`] : []),
      ].join("; ");
      problems.push(
        `the ${native} description (${file}) names ${listed(found)} while the shipped pairs glossed in ${native} (${glossed.join(", ")}) study ${listed(studied)}: ${why}. It is the summary both stores list; change it with the pairs (add-lingua-french-listings D6).`,
      );
    }
  }
  return problems;
}

/** "A", "A and B", "A, B and C". */
function joinNames(names) {
  return names.length <= 1 ? (names[0] ?? "") : `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
}

function main() {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const read = (file) => JSON.parse(readFileSync(join(root, file), "utf8"));
  const version = read("package.json").version;
  const manifest = read("manifest.json");
  const locales = readLocales(root);
  const pairs = shippedPairs(root);
  const problems = [
    versionProblem(version),
    ...manifestProblems(manifest),
    ...localeProblems(manifest, locales),
    ...summaryProblems(manifest, locales, pairs),
  ].filter(Boolean);
  if (problems.length > 0) {
    for (const problem of problems) console.error(`error: ${problem}`);
    process.exit(1);
  }
  console.log(
    `Version ${version} is publishable, and manifest.json is one both stores accept, in ${Object.keys(locales).join(", ")}; ` +
      `the summary of each shipped native language (${shippedNatives(pairs).join(", ")}) names what its pairs study.`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
