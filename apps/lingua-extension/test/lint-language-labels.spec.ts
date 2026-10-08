import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { afterAll, describe, expect, it } from "vitest";

// A language is named in one place per interface language, src/i18n/<language>/languages.ts
// (add-lingua-language-choice D1, add-lingua-native-language-labels D4): every other source and
// page asks analyzer/language-labels.ts, which names it in the interface language. « anglais »,
// "Spanish" or « español » written anywhere else — the other catalogue modules included — is a
// label that will say one language to a reader of another. What counts is read from the
// TypeScript syntax tree (string and template literals, so a comment is not a hit) and from the
// HTML pages' text nodes and attribute values, as test/lint-copy.spec.ts reads French. A file on
// the BASELINE still holds names, until the change moving its copy takes it off — and a file on it
// that holds none fails, so the baseline cannot go stale. The engine's enum names are data, not
// copy: EXCEPTIONS lets them through, text for text, in their one file.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(root, "src");

/** The catalogue's languages modules: the one place a language's name is written, per interface language. */
const LANGUAGES_MODULE = /^src\/i18n\/[a-z]+\/languages\.ts$/;

/** The files that still hold a language's name, until the change moving each surface removes it. */
export const BASELINE: readonly string[] = [];

/** Literals that name a language but are data, by file: the engine's enum names (state/profile.ts). */
export const EXCEPTIONS: Readonly<Record<string, readonly string[]>> = {
  "src/state/profile.ts": ["French", "English", "Spanish"],
};

/**
 * Every name of a studied or native language in every interface language, with its inflections —
 * « anglaise », « espagnoles », « ingleses », « francesas » — as a whole word, whatever the case.
 */
export const NAMES =
  /(?<!\p{L})(?:anglais|espagnol|français|english|spanish|french|ingl[eé]s|español|franc[eé]s)(?:e|es|a|as|s)?(?!\p{L})/iu;

/** A file's path as the baseline writes it: relative, with `/` whatever the platform's separator. */
function baselinePath(treeRoot: string, path: string): string {
  return relative(treeRoot, path).split(sep).join("/");
}

export interface Hit {
  line: number;
  text: string;
}

/** The string and template literals of a TypeScript source, by line. */
export function tsLiterals(path: string, text: string): Hit[] {
  const source = ts.createSourceFile(path, text, ts.ScriptTarget.ES2022, true);
  const hits: Hit[] = [];
  const line = (node: ts.Node): number => source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1;
  const visit = (node: ts.Node): void => {
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
      hits.push({ line: line(node), text: node.text });
    } else if (ts.isTemplateExpression(node)) {
      const chunks = [node.head.text, ...node.templateSpans.map((span) => span.literal.text)];
      hits.push({ line: line(node), text: chunks.join("\u0000") });
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
  return hits;
}

/** The text nodes and attribute values of an HTML page, by line; comments do not count. */
export function htmlLiterals(text: string): Hit[] {
  const stripped = text.replace(/<!--[\s\S]*?-->/g, (c) => c.replace(/[^\n]/g, " "));
  const hits: Hit[] = [];
  const line = (index: number): number => stripped.slice(0, index).split("\n").length;
  for (const m of stripped.matchAll(/>([^<>]+)</g)) {
    const t = m[1].replace(/\s+/g, " ").trim();
    if (t) hits.push({ line: line(m.index), text: t });
  }
  for (const m of stripped.matchAll(/\s[a-zA-Z-]+="([^"]*)"/g)) {
    if (m[1]) hits.push({ line: line(m.index), text: m[1] });
  }
  return hits;
}

/** The literals of a file that name a language, the file's exceptions left out. */
export function nameLiterals(path: string, rel: string, exceptions = EXCEPTIONS): Hit[] {
  const text = readFileSync(path, "utf8");
  const hits = path.endsWith(".html") ? htmlLiterals(text) : tsLiterals(path, text);
  const allowed = exceptions[rel] ?? [];
  return hits.filter((h) => NAMES.test(h.text) && !allowed.includes(h.text));
}

/** Every source and page under `src`, the languages modules and the generated code aside. */
export function sources(src: string): string[] {
  const walk = (dir: string): string[] =>
    [...readdirSync(dir)].sort().flatMap((name) => {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) return ["pkg", "gen"].includes(name) ? [] : walk(path);
      return /\.(ts|html)$/.test(name) && !name.endsWith(".d.ts") ? [path] : [];
    });
  return walk(src).filter((path) => !LANGUAGES_MODULE.test(baselinePath(join(src, ".."), path)));
}

/** What fails on a tree: a name outside the languages modules and the baseline, and a baseline file holding none. */
export function check(treeRoot: string, baseline: readonly string[], exceptions = EXCEPTIONS): string[] {
  const failures: string[] = [];
  const seen = new Set<string>();
  for (const path of sources(join(treeRoot, "src"))) {
    const rel = baselinePath(treeRoot, path);
    seen.add(rel);
    const hits = nameLiterals(path, rel, exceptions);
    if (baseline.includes(rel)) {
      if (hits.length === 0) failures.push(`${rel}: names no language — remove it from the baseline`);
    } else {
      for (const h of hits)
        failures.push(`${rel}:${h.line}: a language named outside the catalogue: ${JSON.stringify(h.text)}`);
    }
  }
  for (const rel of baseline) if (!seen.has(rel)) failures.push(`${rel}: on the baseline, but no such file`);
  return failures;
}

describe("languages are named in one place per interface language", () => {
  const files = sources(SRC);
  const rels = files.map((path) => baselinePath(root, path));

  it("reads the sources and the pages, the labels module among them, the languages modules apart", () => {
    expect(files.length).toBeGreaterThan(100);
    expect(rels).toContain("src/analyzer/language-labels.ts");
    expect(rels).toContain("src/state/profile.ts");
    expect(rels.some((rel) => rel.startsWith("src/i18n/"))).toBe(true);
    for (const language of ["fr", "en", "es"]) {
      expect(statSync(join(SRC, "i18n", language, "languages.ts")).isFile()).toBe(true);
      expect(rels).not.toContain(`src/i18n/${language}/languages.ts`);
    }
    for (const rel of BASELINE) expect(statSync(join(root, rel)).isFile(), rel).toBe(true);
    for (const rel of Object.keys(EXCEPTIONS)) expect(statSync(join(root, rel)).isFile(), rel).toBe(true);
  });

  it("knows every language's name in every interface language, as a whole word, whatever the case", () => {
    for (const word of [
      "anglais",
      "Anglaise",
      "anglaises",
      "espagnol",
      "espagnole",
      "espagnols",
      "français",
      "Française",
      "English",
      "SPANISH",
      "french",
      "inglés",
      "inglesa",
      "ingleses",
      "español",
      "Española",
      "españolas",
      "francés",
      "francesas",
    ]) {
      expect(NAMES.test(`un mot ${word} ici`), word).toBe(true);
    }
    for (const text of ["en", "es", "fr", "en-US", "Englishman", "Spanishness", "frenchify", "angla", "espa"]) {
      expect(NAMES.test(text), text).toBe(false);
    }
  });

  for (const [path, rel] of files.map((path): [string, string] => [path, baselinePath(root, path)])) {
    if (BASELINE.includes(rel)) {
      it(`${rel} still names a language (or leaves the baseline)`, () => {
        expect(nameLiterals(path, rel).length, `${rel}: names none — remove it from the baseline`).toBeGreaterThan(0);
      });
    } else {
      it(`${rel} names no language`, () => {
        const hits = nameLiterals(path, rel).map((h) => `${rel}:${h.line}: ${JSON.stringify(h.text)}`);
        expect(hits, `${rel}: name the language through analyzer/language-labels.ts`).toEqual([]);
      });
    }
  }

  it("names no file that no longer exists", () => {
    const walked = new Set(rels);
    expect(BASELINE.filter((rel) => !walked.has(rel))).toEqual([]);
  });

  it("the exceptions are the engine's enum names, as data, and nothing else in their file", () => {
    const rel = "src/state/profile.ts";
    const text = readFileSync(join(root, rel), "utf8");
    const named = tsLiterals(rel, text).filter((h) => NAMES.test(h.text));
    expect(named.map((h) => h.text).sort()).toEqual([...EXCEPTIONS[rel]].sort());
  });
});

describe("Checked by lint: the lint on a scratch tree", () => {
  const scratch = mkdtempSync(join(tmpdir(), "lingua-lint-languages-"));
  mkdirSync(join(scratch, "src", "popup"), { recursive: true });
  mkdirSync(join(scratch, "src", "i18n", "en"), { recursive: true });
  mkdirSync(join(scratch, "src", "state"), { recursive: true });
  // A module outside the catalogue that writes "Spanish" as a name…
  writeFileSync(
    join(scratch, "src", "popup", "popup.ts"),
    ["// English in a comment does not count", "const label = `No Spanish text on ${page}`;", "export { label };"].join(
      "\n",
    ),
  );
  // …a page that writes « español » in its text…
  writeFileSync(join(scratch, "src", "popup", "popup.html"), '<!-- español -->\n<p lang="es">Texto en español</p>\n');
  // …a catalogue module that is not the languages module, naming one…
  writeFileSync(
    join(scratch, "src", "i18n", "en", "settings.ts"),
    'export const settings = { title: "English level" };\n',
  );
  // …the languages module itself, which may…
  writeFileSync(join(scratch, "src", "i18n", "en", "languages.ts"), 'export const languages = { name: "English" };\n');
  // …the engine's enum names, which are data, beside a text that is not…
  writeFileSync(
    join(scratch, "src", "state", "profile.ts"),
    [
      'const NAMES = new Map([["French", "fr"], ["English", "en"]]);',
      'const copy = "Spanish text";',
      "export { NAMES, copy };",
    ].join("\n"),
  );
  afterAll(() => rmSync(scratch, { recursive: true, force: true }));

  it("A name outside the catalogue: fails, naming the file and the line", () => {
    expect(check(scratch, [], { "src/state/profile.ts": ["French", "English"] })).toEqual([
      'src/i18n/en/settings.ts:1: a language named outside the catalogue: "English level"',
      'src/popup/popup.html:2: a language named outside the catalogue: "Texto en español"',
      'src/popup/popup.ts:2: a language named outside the catalogue: "No Spanish text on \\u0000"',
      'src/state/profile.ts:2: a language named outside the catalogue: "Spanish text"',
    ]);
  });

  it("A baseline file that names a language passes; one that names none went stale and fails", () => {
    expect(check(scratch, ["src/popup/popup.ts", "src/popup/popup.html", "src/i18n/en/settings.ts"], {})).toEqual([
      'src/state/profile.ts:1: a language named outside the catalogue: "French"',
      'src/state/profile.ts:1: a language named outside the catalogue: "English"',
      'src/state/profile.ts:2: a language named outside the catalogue: "Spanish text"',
    ]);
    mkdirSync(join(scratch, "src", "stats"), { recursive: true });
    writeFileSync(join(scratch, "src", "stats", "view.ts"), 'import { x } from "./y.ts";\nexport { x };\n');
    expect(check(scratch, ["src/stats/view.ts", "src/stats/gone.ts"], {})).toEqual(
      expect.arrayContaining([
        "src/stats/view.ts: names no language — remove it from the baseline",
        "src/stats/gone.ts: on the baseline, but no such file",
      ]),
    );
  });
});
