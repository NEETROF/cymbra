import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { LANGS, type Lang } from "../../src/lib/i18n";
import { outputFileFor } from "../../src/lib/pinned-routes";

// Run after `yarn build` (see `vitest.build.config.ts`): each locale's deletion page and
// not-found page declare their language on `<html>`, and the deletion page mounts its
// island with that same language in its props — the island's dictionary (`i18n.ts`) is
// keyed on it, so a page saying `es` around an island told `en` would greet a Spanish
// reader in English (change: add-site-spanish-locale). Asserted on the built HTML: the
// layouts are `.astro`, which only the build renders.

const dist = resolve(__dirname, "../../dist");

const DELETION: Record<Lang, string> = {
  fr: "/suppression-compte",
  en: "/en/delete-account",
  es: "/es/eliminar-cuenta",
};
// Astro gives the root `404.astro` its special filename; `localised404()` copies the
// others to `dist/<locale>/404.html`, outside `outputFileFor`'s one-directory-per-page rule.
const NOT_FOUND: Record<Lang, string> = { fr: "404.html", en: "en/404.html", es: "es/404.html" };

const read = (file: string) => readFileSync(resolve(dist, file), "utf8");

describe("each locale's pages say their language", () => {
  for (const lang of LANGS) {
    it(`${lang}: the deletion page is <html lang="${lang}"> and mounts its island in ${lang}`, () => {
      const html = read(outputFileFor(DELETION[lang]));
      expect(html).toContain(`<html lang="${lang}"`);
      // One island on the page, the deletion one, with `lang` serialised in its props
      // (`[0, value]` is Astro's encoding of a plain string).
      const islands = [...html.matchAll(/<astro-island [^>]*>/g)].map((m) => m[0]);
      expect(islands, `${DELETION[lang]} islands`).toHaveLength(1);
      expect(islands[0]).toContain("DeleteAccountIsland");
      expect(islands[0]).toContain(`props="{&quot;lang&quot;:[0,&quot;${lang}&quot;]}"`);
    });

    it(`${lang}: the not-found page is <html lang="${lang}">`, () => {
      expect(read(NOT_FOUND[lang])).toContain(`<html lang="${lang}"`);
    });
  }

  // The Spanish home and Music pages (change: extend-site-spanish-locale, D1): static
  // pages, in Spanish, mounting no island.
  for (const path of ["/es/", "/es/music"]) {
    it(`es: ${path} is <html lang="es"> and mounts no island`, () => {
      const html = read(outputFileFor(path));
      expect(html).toContain('<html lang="es"');
      expect(html).not.toContain("<astro-island");
    });
  }
});
