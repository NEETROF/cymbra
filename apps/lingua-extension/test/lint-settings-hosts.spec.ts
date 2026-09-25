import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Réglages have ONE builder, `mountSettings` (src/reading/settings-view.ts), and every surface
// that shows them renders it. The toolbar popup once carried its own copy — markup in
// popup.html, wiring in popup.ts — and every block added to the shared view afterwards was
// missing from it; the read-aloud voice was found missing in dogfooding, on the surface a
// reader opens first. Nothing failed: the copy compiled, and looked finished.
//
// So: every host calls `mountSettings`, and no other page or module holds a Réglages block of
// its own — recognised by the block titles the builder uses, as a text node in HTML or as a
// string literal in code.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "src");
const BUILDER = join(src, "reading", "settings-view.ts");

/** The surfaces that show Réglages. A new one belongs here. */
const HOSTS = ["src/popup/popup.ts", "src/sidepanel/sidepanel.ts", "src/reading/drawer.ts"];

function files(dir: string, ext: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (name === "pkg" || name === "gen") return []; // generated
    if (statSync(path).isDirectory()) return files(path, ext);
    return path.endsWith(ext) ? [path] : [];
  });
}

const titles = [...readFileSync(BUILDER, "utf8").matchAll(/settingBlock\("([^"]+)"\)/g)].map((m) => m[1]);
const escapeHtml = (s: string): string => s.replace(/&/g, "&amp;");
const rel = (path: string): string => path.slice(root.length + 1);

describe("one Réglages, rendered by every surface", () => {
  it("reads the block titles off the builder", () => {
    expect(titles).toEqual(expect.arrayContaining(["Niveau d'anglais", "Lecture à voix haute", "Réinitialisation"]));
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

  for (const module of files(src, ".ts").filter((f) => f !== BUILDER)) {
    it(`${rel(module)} holds no Réglages block of its own`, () => {
      const code = readFileSync(module, "utf8");
      const copied = titles.filter((t) => code.includes(`"${t}"`) || code.includes(`'${t}'`));
      expect(copied, `${rel(module)} rebuilds Réglages — mount the shared view instead`).toEqual([]);
    });
  }
});
