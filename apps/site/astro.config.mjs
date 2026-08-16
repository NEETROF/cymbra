// @ts-check
import { defineConfig } from 'astro/config';
import vue from '@astrojs/vue';

// https://astro.build/config
export default defineConfig({
  site: 'https://cymbra.app',
  // Interactive islands (sign-in, code redemption, account, checkout — change:
  // add-site-account-pages) are Vue components; the rest stays static.
  integrations: [vue()],
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
