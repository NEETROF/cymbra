/// <reference types="vite/client" />
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
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
// of Réglages would hold. The catalogue itself, in its three languages, is the one place — its
// `settings` module alone, in each language, holds a title. And every host hands the view the
// interface language (localise-lingua-settings D1): a host that forgot would show Réglages in French
// to every reader, and no French assertion would notice.

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
/** Every `mountSettings(…)` call in `code`, from its name to its closing parenthesis. */
function mountSettingsCalls(code: string): string[] {
  return [...code.matchAll(/\bmountSettings\(/g)].map((m) => {
    let depth = 0;
    for (let i = m.index + m[0].length - 1; i < code.length; i++) {
      if (code[i] === "(") depth++;
      else if (code[i] === ")" && --depth === 0) return code.slice(m.index, i + 1);
    }
    return code.slice(m.index);
  });
}
/** Whether a `mountSettings(…)` call hands the view the interface language. */
const handsLanguage = (call: string): boolean => /\binterfaceLanguage\b\s*[:,}]/.test(call);
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

  it("A host that mounts Réglages without the interface language: caught", () => {
    const [bare] = mountSettingsCalls('settings ??= mountSettings($("view"), port, area, { persist, store: f(x) });');
    expect(bare).toBe('mountSettings($("view"), port, area, { persist, store: f(x) })');
    expect(handsLanguage(bare)).toBe(false);
    expect(handsLanguage("mountSettings(c, port, area, {\n  persist,\n  interfaceLanguage: language,\n})")).toBe(true);
    expect(handsLanguage("mountSettings(c, port, area, { persist, interfaceLanguage })")).toBe(true);
  });

  for (const host of HOSTS) {
    it(`${host} renders mountSettings, in the interface language`, () => {
      const calls = mountSettingsCalls(readFileSync(join(root, host), "utf8"));
      expect(calls.length, `${host} renders no mountSettings`).toBeGreaterThan(0);
      for (const call of calls) {
        expect(handsLanguage(call), `${host}: hand mountSettings the interfaceLanguage — ${call}`).toBe(true);
      }
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
      const code = readFileSync(module, "utf8");
      const copied = copiedIn(code);
      expect(copied, `${rel(module)} rebuilds Réglages — mount the shared view instead`).toEqual([]);
      // A block and its title are built in one place: the builder's `settingBlock`.
      if (module === BUILDER) return;
      expect(code, `${rel(module)} builds a Réglages block`).not.toMatch(/\bsettingBlock\(|["'`]set-label["'`]/);
    });
  }
});

// — In the catalogue, a title lives in its language's `settings` module alone —

type Catalogue = Record<string, unknown>;
const modules = import.meta.glob<Catalogue>("../src/i18n/{fr,en,es}/*.ts", { eager: true });

/** The plain texts of a module, nested objects included (a message's output is its parts'). */
function plainTexts(value: unknown): string[] {
  if (typeof value === "string") return [value];
  if (value && typeof value === "object") return Object.values(value).flatMap(plainTexts);
  return [];
}

describe("the block titles, once per language in the catalogue", () => {
  for (const language of ["fr", "en", "es"] as const satisfies readonly InterfaceLanguage[]) {
    const own = (name: string): Catalogue =>
      Object.values(modules[`../src/i18n/${language}/${name}.ts`]!)[0] as Catalogue;
    const languageTitles = TITLE_KEYS.map((key) => own("settings")[key] as string);

    it(`${language}: the eleven titles are its settings module's, and no other module of ${language} holds one`, () => {
      expect(new Set(languageTitles).size).toBe(TITLE_KEYS.length);
      const elsewhere = Object.entries(modules)
        .filter(([path]) => path.startsWith(`../src/i18n/${language}/`) && !path.endsWith("/settings.ts"))
        .flatMap(([path, module]) =>
          plainTexts(module)
            .filter((text) => languageTitles.includes(text))
            .map((text) => `${path}: « ${text} »`),
        );
      expect(elsewhere, "a title belongs to the settings module alone — the view titles its blocks from it").toEqual(
        [],
      );
    });
  }
});
