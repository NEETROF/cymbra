# Tasks

## 1. The pages (apps/site)

- [ ] 1.1 `src/pages/es/index.astro` and `src/pages/es/music.astro`: the French pages' structure and classes, the Spanish drafts of design D2's table (tú, neutral, M10), section ids `productos` and `funciones`, `StoreButtons links={musicStores('es')} primary`, `Community lang="es"`, no island; the Music page's closing paragraph not carried (D1, D2, D4).
- [ ] 1.2 The home's Lingua card in `src/lib/lingua-text.ts`: one builder from the shipped pairs, on change 30's Spanish names, speakers and grammar and `fill`, inserted with `set:html`; the button `linguaHref('es')` (D3). Tests in `test/lingua-text.spec.ts`: today's pairs (« Lee la web en inglés o en español… Pensada para francohablantes… »), today's and es-en (« francohablantes y anglohablantes »), the matrix (« Lee la web en inglés… », no audience sentence), the matrix with fr-en and fr-es (« en inglés o en francés »), a language the Spanish table does not name refused.
- [ ] 1.3 The links (D5): `alternates` of `src/pages/index.astro` and `en/index.astro` gain `es: '/es/'`, of `music.astro` and `en/music.astro` `es: '/es/music'`; `Base.astro`'s Spanish table: brand `/es/`, navigation and footer « Music » `/es/music` (« Cuenta » and « Código de acceso » unchanged); `src/pages/es/404.astro`: « Inicio » `/es/`, « Cymbra Music » `/es/music`; `lingua-text.ts` `MUSIC_HREF.es` `/es/music`. The comments that say the site has no Spanish home or Music page (`Base.astro`, `es/404.astro`, `lingua-text.ts`) updated.

## 2. Tests

- [ ] 2.1 `test/post-build/alternates.spec.ts`: the home and Music pages in each language — `hreflang` fr, en, es and the switch to the two others; *a page without a Spanish twin* on `/account`, `/en/redeem` and `/checkout` instead of `/` and `/en/music`; on a Spanish page, the brand and « Music » open `/es/` and `/es/music`, « Cuenta » and « Código de acceso » `/en/account` and `/en/redeem` (D4, D5).
- [ ] 2.2 `test/post-build/locale.spec.ts`: `/es/` and `/es/music/` are `<html lang="es">` and mount no island.
- [ ] 2.3 `test/astro/` (Container API, the data files stood in for as `lingua-page.spec.ts` does): `/es/` with today's pairs — the audience sentence, the card's button `/en/lingua`, the brand `/es/` — and with the matrix — « Lee la web en inglés », the button `/es/lingua`; `test/lingua-text.spec.ts`'s Spanish Lingua page expects its closing to link `/es/music` (D3, D5).

## 3. Gates, review and docs

- [ ] 3.1 In `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes` (the pinned routes unchanged). Build `origin/main` and the branch from the same directory (an island's `uid` hashes its component's path) and compare `dist/`: two pages more; `/`, `/en/`, `/music/` and `/en/music/` differ by their Spanish `hreflang` and switch entry alone; the existing Spanish files by their brand and Music links and the not-found page's two buttons; every other file and `_astro/` byte for byte (D6).
- [ ] 3.2 `apps/site/README.md`: the pages table (`/es/`, `/es/music`), the Spanish paragraph (home and Music in Spanish; account, code and checkout in English, and why), the Music page's copy mirroring `apps/music/store/copy/{fr,en,es}.md`.
- [ ] 3.3 [manual] The owner reviews the Spanish of both pages (M9) — design D2's table, the Lingua card's two sentences (D3) — and answers the open questions: Music's Spanish listing URLs (a later change), the « beta » badge, the Music page's closing paragraph, « lectura » or « reproducción ».
- [ ] 3.4 [manual] The owner deploys the site (`site-deploy`, M18) only after change 10's server (`add-lingua-native-language-server`) is deployed and checked from outside (its task 5.2) — the deploy also publishes change 31's privacy annex (its 3.2) — and before change 35's release (D7).
- [ ] 3.5 `openspec validate extend-site-spanish-locale --strict` passes, and `python3 scripts/openspec_archive_order.py extend-site-spanish-locale` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 29b is marked done in `docs/lingua/language-matrix-programme.md`.
