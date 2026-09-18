// @ts-check
import { copyFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'astro/config';
import vue from '@astrojs/vue';

// Astro gives `src/pages/404.astro` its special treatment only at the root: it writes
// `dist/404.html` there, but `src/pages/en/404.astro` is an ordinary page and lands on
// `dist/en/404/index.html`. Cloudflare Pages serves the nearest `404.html` walking up
// the requested path, so without this an unmatched `/en/...` would answer with the
// FRENCH not-found page. Copy it into place (change: pin-music-site-url-contract).
function localised404() {
  return {
    name: 'cymbra:localised-404',
    hooks: {
      'astro:build:done': ({ dir }) => {
        const root = fileURLToPath(dir);
        const built = `${root}en/404/index.html`;
        if (!existsSync(built)) throw new Error('en/404 page is missing from the build');
        copyFileSync(built, `${root}en/404.html`);
      },
    },
  };
}

// https://astro.build/config
export default defineConfig({
  site: 'https://cymbra.app',
  // Interactive islands (sign-in, code redemption, account, checkout — change:
  // add-site-account-pages) are Vue components; the rest stays static.
  integrations: [vue(), localised404()],
  vite: {
    resolve: {
      // `@cymbra/web-auth` is a source-only portal package whose composables import
      // `vue`: force ONE Vue instance from this app (a second copy breaks reactivity).
      dedupe: ['vue'],
    },
  },
  i18n: {
    defaultLocale: 'fr',
    locales: ['fr', 'en'],
    routing: { prefixDefaultLocale: false },
  },
});
