import { mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterAll, describe, expect, it } from "vitest";
import { baselinePath, type Hit, htmlLiterals, sources as walk, tsLiterals } from "./support/literals.ts";

// The interface's copy lives in the catalogue, src/i18n (add-lingua-interface-language D5): a
// French string literal anywhere else in src/ fails, unless its file is on the BASELINE — the
// surfaces still holding their copy, which the changes moving them take off the list:
// localise-lingua-account-onboarding (17); localise-lingua-reading-surfaces (14) took the popup,
// the HUD, the drawer, the cards, the side panel's page and the reader off it,
// localise-lingua-settings (15) Réglages and its blocks, localise-lingua-review-stats (16) the review
// and the statistics. The baseline is checked the other way too: a file on it that holds no French
// literal fails, so it cannot go stale.
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
  "src/onboarding/level-row.ts",
  "src/onboarding/onboarding.html",
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

export function frenchLiterals(path: string): Hit[] {
  const text = readFileSync(path, "utf8");
  const hits = path.endsWith(".html") ? htmlLiterals(text) : tsLiterals(path, text);
  return hits.filter((h) => isFrench(h.text));
}

/** Every source and page under `src`, the catalogue and the generated code aside. */
export function sources(src: string): string[] {
  return walk(src, ["pkg", "gen", "i18n"]);
}

/** What fails on a tree: French outside the baseline, and a baseline file holding none. */
export function check(treeRoot: string, baseline: readonly string[]): string[] {
  const failures: string[] = [];
  const src = join(treeRoot, "src");
  const seen = new Set<string>();
  for (const path of sources(src)) {
    const rel = baselinePath(treeRoot, path);
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
    expect(BASELINE.filter((f) => f.endsWith(".html")).length).toBe(2);
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
    const rel = baselinePath(root, path);
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
    // From the walk above, not a second one: a baseline entry must be a file the walk reaches.
    const walked = new Set(files.map((path) => baselinePath(root, path)));
    expect(BASELINE.filter((rel) => !walked.has(rel))).toEqual([]);
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
