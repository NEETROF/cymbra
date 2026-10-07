import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { afterAll, describe, expect, it } from "vitest";

// The interface's copy lives in the catalogue, src/i18n (add-lingua-interface-language D5): a
// French string literal anywhere else in src/ fails, unless its file is on the BASELINE — the
// surfaces still holding their copy, which the changes moving them (add-lingua-popup-language and
// the following) take off the list. The baseline is checked the other way too: a file on it that
// holds no French literal fails, so it cannot go stale.
//
// What counts is read from the TypeScript syntax tree — string and template literals only, so a
// comment or a regular expression in French is not a hit — and from the HTML pages' text nodes and
// attribute values. A literal is French when it holds an accented letter, « », ’, or one of the
// inventory's unaccented French words as a whole word. ASCII French with none of those is beyond
// the lint; the baseline, shrinking by whole files whose literals were moved by reading them, is
// the real guard.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(root, "src");

/** The files that hold French literals today, until the change moving each surface removes it. */
export const BASELINE = [
  "src/account/account.html",
  "src/account/copy.ts",
  "src/account/flow.ts",
  "src/account/view.ts",
  "src/analyzer/language-labels.ts",
  "src/onboarding/level-row.ts",
  "src/onboarding/onboarding.html",
  "src/popup/popup.html",
  "src/popup/popup.ts",
  "src/reader/copy.ts",
  "src/reader/library.ts",
  "src/reader/reader.html",
  "src/reading/account-setting.ts",
  "src/reading/book-display-view.ts",
  "src/reading/colour-settings-view.ts",
  "src/reading/drawer.ts",
  "src/reading/grammar-labels.ts",
  "src/reading/hud.ts",
  "src/reading/selection-card.ts",
  "src/reading/settings-view.ts",
  "src/reading/studied-languages-view.ts",
  "src/reading/translation-setting.ts",
  "src/reading/wordpopup.ts",
  "src/review/review-page.ts",
  "src/review/view.ts",
  "src/sidepanel/sidepanel.html",
  "src/stats/ladder.ts",
  "src/stats/stats.html",
  "src/stats/view.ts",
  "src/sync/status.ts",
];

/** Unaccented words of the inventory's copy, matched whole and case-sensitively. */
export const FRENCH_WORDS = [
  "Annuler",
  "Fermer",
  "Retour",
  "Continuer",
  "Valider",
  "Retirer",
  "Supprimer",
  "Sauvegarder",
  "Restaurer",
  "Modifier",
  "Ignorer",
  "Sommaire",
  "Statistiques",
  "Mots",
  "mots",
  "inconnus",
  "connus",
  "suivis",
  "Livre",
  "Livres",
  "titre",
  "Niveau",
  "Langue",
  "Compte",
  "Couleurs",
  "Traduction",
  "Affichage",
  "Apparence",
  "Synchronisation",
  "Papier",
  "Sombre",
  "Fond",
  "Texte",
  "Soulignement",
  "Tirets",
  "Aucun",
  "Choisie",
  "Ordre",
  "Nombre",
  "courants",
  "rares",
  "connu",
  "Perso",
  "Plein",
  "Directe",
  "Facile",
  "Difficile",
  "Pseudo",
  "Lier",
  "Partielle",
  "Recommencer",
  "appareil",
  "cette",
  "pour",
  "dans",
  "avec",
  "les",
  "des",
  "ton",
  "tes",
  "sur",
];

const ACCENTED = /[àâäçéèêëîïôöùûüÿœæÀÂÄÇÉÈÊËÎÏÔÖÙÛÜŸŒÆ«»’]/;
const WORDS = new RegExp(`(?<!\\p{L})(?:${FRENCH_WORDS.join("|")})(?!\\p{L})`, "u");

export function isFrench(text: string): boolean {
  return ACCENTED.test(text) || WORDS.test(text);
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

export function frenchLiterals(path: string): Hit[] {
  const text = readFileSync(path, "utf8");
  const hits = path.endsWith(".html") ? htmlLiterals(text) : tsLiterals(path, text);
  return hits.filter((h) => isFrench(h.text));
}

/** Every source and page under `src`, the catalogue and the generated code aside. */
export function sources(src: string): string[] {
  return readdirSync(src).flatMap((name) => {
    const path = join(src, name);
    if (statSync(path).isDirectory()) return ["pkg", "gen", "i18n"].includes(name) ? [] : sources(path);
    return /\.(ts|html)$/.test(name) && !name.endsWith(".d.ts") ? [path] : [];
  });
}

/** What fails on a tree: French outside the baseline, and a baseline file holding none. */
export function check(treeRoot: string, baseline: readonly string[]): string[] {
  const failures: string[] = [];
  const src = join(treeRoot, "src");
  const seen = new Set<string>();
  for (const path of sources(src)) {
    const rel = relative(treeRoot, path);
    seen.add(rel);
    const hits = frenchLiterals(path);
    if (baseline.includes(rel)) {
      if (hits.length === 0) failures.push(`${rel}: holds no French literal — remove it from the baseline`);
    } else {
      for (const h of hits) failures.push(`${rel}:${h.line}: French outside the catalogue: ${JSON.stringify(h.text)}`);
    }
  }
  for (const rel of baseline) if (!seen.has(rel)) failures.push(`${rel}: on the baseline, but no such file`);
  return failures;
}

describe("no French literal outside the catalogue", () => {
  const files = sources(SRC);

  it("reads the sources and the pages, and knows the baseline's files", () => {
    expect(files.length).toBeGreaterThan(100);
    expect(BASELINE.filter((f) => f.endsWith(".html")).length).toBe(6);
    for (const rel of BASELINE) expect(statSync(join(root, rel)).isFile(), rel).toBe(true);
  });

  it("counts literals, not comments or regular expressions", () => {
    const hits = tsLiterals(
      "x.ts",
      [
        "// « Réglages » in a comment does not count",
        "const re = /déjà/;",
        'const a = "Annuler";',
        "const b = `${n} carte(s) à revoir`;",
        "const c = `plain`;",
      ].join("\n"),
    );
    expect(hits.map((h) => [h.line, isFrench(h.text)])).toEqual([
      [3, true],
      [4, true],
      [5, false],
    ]);
    expect(isFrench("Annulers")).toBe(false); // a whole word only
    expect(isFrench("stats")).toBe(false); // case matters: « Stats » is a label, "stats" a view id
    expect(isFrench("Mots inconnus")).toBe(true);
  });

  it("reads a page's text and attributes, not its comments", () => {
    const hits = htmlLiterals(
      [
        "<!doctype html>",
        '<html lang="fr">',
        "<!-- Réglages › Données live in mountSettings -->",
        '<button aria-label="Réglages">⚙</button>',
        "<p>Mots inconnus</p>",
      ].join("\n"),
    );
    expect(hits.filter((h) => isFrench(h.text))).toEqual([
      { line: 5, text: "Mots inconnus" },
      { line: 4, text: "Réglages" },
    ]);
  });

  for (const path of files) {
    const rel = relative(root, path);
    if (BASELINE.includes(rel)) {
      it(`${rel} still holds French literals (or leaves the baseline)`, () => {
        expect(frenchLiterals(path).length, `${rel}: holds none — remove it from the baseline`).toBeGreaterThan(0);
      });
    } else {
      it(`${rel} holds no French literal`, () => {
        const hits = frenchLiterals(path).map((h) => `${rel}:${h.line}: ${JSON.stringify(h.text)}`);
        expect(hits, `${rel}: move the copy to src/i18n, or name the file in the baseline`).toEqual([]);
      });
    }
  }

  it("names no file that no longer exists", () => {
    expect(check(root, BASELINE).filter((f) => f.includes("no such file"))).toEqual([]);
  });
});

describe("the lint on a scratch tree", () => {
  const scratch = mkdtempSync(join(tmpdir(), "lingua-lint-copy-"));
  mkdirSync(join(scratch, "src", "popup"), { recursive: true });
  mkdirSync(join(scratch, "src", "stats"), { recursive: true });
  // A surface's module with a literal the baseline does not name…
  writeFileSync(
    join(scratch, "src", "popup", "popup.ts"),
    ['const label = "Réglages";', "export { label };"].join("\n"),
  );
  // …a module moved to the catalogue, still on the baseline…
  writeFileSync(
    join(scratch, "src", "stats", "view.ts"),
    'import { stats } from "../i18n/fr/stats.ts";\nexport { stats };\n',
  );
  // …and a page whose only French is a comment.
  writeFileSync(join(scratch, "src", "stats", "stats.html"), '<!-- Statistiques -->\n<h1 id="stats-root"></h1>\n');
  afterAll(() => rmSync(scratch, { recursive: true, force: true }));

  it("A French literal outside the catalogue: fails, naming the file and the line", () => {
    expect(check(scratch, ["src/stats/view.ts"])).toEqual([
      'src/popup/popup.ts:1: French outside the catalogue: "Réglages"',
      "src/stats/view.ts: holds no French literal — remove it from the baseline",
    ]);
  });

  it("A baseline that went stale: a page with only a comment fails the baseline too", () => {
    expect(check(scratch, ["src/popup/popup.ts", "src/stats/stats.html"])).toEqual([
      "src/stats/stats.html: holds no French literal — remove it from the baseline",
    ]);
  });
});
