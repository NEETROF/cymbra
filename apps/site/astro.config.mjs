// @ts-check
import { copyFileSync, existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'astro/config';
import vue from '@astrojs/vue';
import { sitemapEntries, sitemapXml } from './src/lib/sitemap.ts';

// French at the root, English under `/en/`, Spanish under `/es/` (change:
// add-site-spanish-locale) — each language with its own slugs, and each page naming its
// translations through `alternates` (see `src/layouts/Base.astro`).
const DEFAULT_LOCALE = 'fr';
const LOCALES = ['fr', 'en', 'es'];

// Astro gives `src/pages/404.astro` its special treatment only at the root: it writes
// `dist/404.html` there, but `src/pages/en/404.astro` is an ordinary page and lands on
// `dist/en/404/index.html`. Cloudflare Pages serves the nearest `404.html` walking up
// the requested path, so without this an unmatched `/en/...` would answer with the
// FRENCH not-found page. Copy it into place, for every non-default locale (change:
// pin-music-site-url-contract; `es` by add-site-spanish-locale).
function localised404() {
  return {
    name: 'cymbra:localised-404',
    hooks: {
      /** @param {{ dir: URL }} ctx */
      'astro:build:done': ({ dir }) => {
        const root = fileURLToPath(dir);
        for (const locale of LOCALES.filter((l) => l !== DEFAULT_LOCALE)) {
          const built = `${root}${locale}/404/index.html`;
          if (!existsSync(built)) throw new Error(`${locale}/404 page is missing from the build`);
          copyFileSync(built, `${root}${locale}/404.html`);
        }
      },
    },
  };
}

// `sitemap.xml` from the built pages (change: add-site-lingua-showcase, D13): every page without
// `noindex`, at its canonical address, with the translations it names (`src/lib/sitemap.ts`). It
// runs after `localised404`, whose copies are `noindex` like their originals. `public/robots.txt`
// names it.
function sitemap() {
  return {
    name: 'cymbra:sitemap',
    hooks: {
      /** @param {{ dir: URL }} ctx */
      'astro:build:done': ({ dir }) => {
        const root = fileURLToPath(dir);
        /** @param {string} at @returns {string[]} */
        const pages = (at) =>
          readdirSync(at, { withFileTypes: true }).flatMap((entry) => {
            const path = join(at, entry.name);
            if (entry.isDirectory()) return pages(path);
            return entry.name.endsWith('.html') ? [path] : [];
          });
        const entries = sitemapEntries(pages(root).map((file) => ({ file: relative(root, file), html: readFileSync(file, 'utf8') })));
        writeFileSync(join(root, 'sitemap.xml'), sitemapXml(entries));
      },
    },
  };
}

// https://astro.build/config
export default defineConfig({
  site: 'https://cymbra.app',
  // Interactive islands (sign-in, code redemption, account, checkout — change:
  // add-site-account-pages) are Vue components; the rest stays static.
  integrations: [vue(), localised404(), sitemap()],
  image: {
    // The one SVG the build rasterises is the site's own share image, the name « Cymbra » drawn as
    // paths (`src/assets/brand-share.svg`, change: add-site-lingua-showcase, D12) — never a file
    // from elsewhere.
    dangerouslyProcessSVG: true,
  },
  vite: {
    resolve: {
      // `@cymbra/web-auth` is a source-only portal package whose composables import
      // `vue`: force ONE Vue instance from this app (a second copy breaks reactivity).
      dedupe: ['vue'],
    },
  },
  i18n: {
    defaultLocale: DEFAULT_LOCALE,
    locales: LOCALES,
    routing: { prefixDefaultLocale: false },
  },
});
