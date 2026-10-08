import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { settings } from "@/i18n/fr/settings.ts";

// Réglages have ONE builder, `mountSettings` (src/reading/settings-view.ts), and every surface
// that shows them renders it. The toolbar popup once carried its own copy — markup in
// popup.html, wiring in popup.ts — and every block added to the shared view afterwards was
// missing from it; the read-aloud voice was found missing in dogfooding, on the surface a
// reader opens first. Nothing failed: the copy compiled, and looked finished.
//
// So: every host calls `mountSettings`, and no other page or module holds a Réglages block of
// its own — recognised by the block titles the builder uses, as a text node in HTML or as a
// string literal in code. The titles are the catalogue's (localise-lingua-settings D2): the
// builder writes `settingBlock(copy.<key>)`, and the French entries of those keys are what a copy
// of Réglages would hold. The catalogue itself, in its three languages, is the one place.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "src");
const BUILDER = join(src, "reading", "settings-view.ts");

/**
 * The blocks' titles, by their keys in `src/i18n/fr/settings.ts`. The level blocks are titled per
 * studied language (`levelTitle`, language-labels), the others here; a new block's key belongs here.
 */
const TITLE_KEYS = [
  "studiedLanguages",
  "barOnPage",
  "readAloud",
  "books",
  "display",
  "colours",
  "translation",
  "shortcuts",
  "account",
  "sync",
  "reset",
] as const satisfies readonly (keyof typeof settings)[];

/** The surfaces that show Réglages. A new one belongs here. */
const HOSTS = ["src/popup/popup.ts", "src/sidepanel/sidepanel.ts", "src/reading/drawer.ts"];

function files(dir: string, ext: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (name === "pkg" || name === "gen") return []; // generated
    // The copy catalogue (src/i18n) holds the block titles as text, not a block of its own.
    if (name === "i18n") return [];
    if (statSync(path).isDirectory()) return files(path, ext);
    return path.endsWith(ext) ? [path] : [];
  });
}

const titles: string[] = TITLE_KEYS.map((key) => settings[key]);
const escapeHtml = (s: string): string => s.replace(/&/g, "&amp;");
const rel = (path: string): string => path.slice(root.length + 1);
/** The titles a module writes as string literals. */
const copiedIn = (code: string): string[] => titles.filter((t) => code.includes(`"${t}"`) || code.includes(`'${t}'`));

describe("one Réglages, rendered by every surface", () => {
  it("reads the block titles off the catalogue, the keys the builder titles its blocks with", () => {
    expect(titles).toEqual(expect.arrayContaining(["Langues étudiées", "Lecture à voix haute", "Réinitialisation"]));
    // Every `settingBlock(copy.<key>)` of the builder is listed, and every listed key is one of them.
    const built = [...readFileSync(BUILDER, "utf8").matchAll(/settingBlock\(copy\.(\w+)\)/g)].map((m) => m[1]);
    expect([...built].sort()).toEqual([...TITLE_KEYS].sort());
  });

  it("A host that writes a block's title as a literal: caught, the title named", () => {
    expect(copiedIn('const block = settingBlock("Compte");')).toEqual(["Compte"]);
    expect(copiedIn("const block = settingBlock(copy.account);")).toEqual([]);
  });

  for (const host of HOSTS) {
    it(`${host} renders mountSettings`, () => {
      expect(readFileSync(join(root, host), "utf8")).toMatch(/\bmountSettings\(/);
    });
  }

  for (const page of files(src, ".html")) {
    it(`${rel(page)} holds no Réglages block of its own`, () => {
      const html = readFileSync(page, "utf8");
      const copied = titles.filter((t) => html.includes(`>${t}<`) || html.includes(`>${escapeHtml(t)}<`));
      expect(copied, `${rel(page)} rebuilds Réglages — mount the shared view instead`).toEqual([]);
    });
  }

  for (const module of files(src, ".ts")) {
    it(`${rel(module)} holds no Réglages block of its own`, () => {
      const copied = copiedIn(readFileSync(module, "utf8"));
      expect(copied, `${rel(module)} rebuilds Réglages — mount the shared view instead`).toEqual([]);
    });
  }
});
