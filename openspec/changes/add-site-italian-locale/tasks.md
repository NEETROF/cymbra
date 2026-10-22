# Tasks

## 0. Before this change's pull request

- [ ] 0.1 [macOS] A pull request of its own, a `chore(music)` merged first (D4): the Italian showcase video, `melos run showcase-video -- it`, committed under `apps/music/store/video/it/`; it passes the site's checks alone, since `music-media.spec` ignores a language not in `LANGS`.

## 1. The locale

- [ ] 1.1 `apps/site`: `astro.config.mjs`'s `LOCALES` and `src/lib/i18n.ts`'s `LANGS` gain `it`; `localised404()` writes `it/404.html`; `Base.astro`'s navigation, footer (French and English links), labels (« IT ») and brand alt; `Legal.astro`, `Community.astro`, `Downloads.astro`; the 24 French, English and Spanish page files' `alternates` gain their Italian twin (D1).
- [ ] 1.2 The types the compiler names once `LANGS` holds `it`: the islands' dictionary (107 keys) and `it-IT` dates; `lingua-leave.ts`'s `THANKS_ID` (`grazie`) and `SUPPORT_HREF`; `lingua-text.ts`'s `LINGUA_TEXT.it` (its `showcase` captions as type stubs), `MUSIC_HREF.it`, `GRAMMAR.it`, and `speakers.it` in the French, English and Spanish tables; `music-media.ts`; `stores.ts`; `AccountIsland.vue` (D5, D6).
- [ ] 1.3 `plan-view.ts`'s `appleReturnUrl` strips `/it`; `packages/web-auth/src/apple.ts`: `it_IT`; `.env.example`'s Apple Return URLs gain `/elimina-account` (D5).

## 2. The pages

- [ ] 2.1 `/it/privacy/`, `/it/termini/`, `/it/assistenza/`, `/it/elimina-account/`, translated from the French as they stand; the three legal pages carrying, under their title, « Traduzione di cortesia: fa fede la versione francese » linking their French page, and their French page's date of last update and sections; the Lingua labels from change 33's catalogue (D2).
- [ ] 2.2 `/it/lingua/arrivederci/` and its thanks `#grazie`; `/it/404` (D1).
- [ ] 2.3 `/it/` with the Italian home card's builder, and `/it/music/` with Music's Italian terms; the Italian gallery from Music's store captures; `scripts/sync-music-media.ts` and `scripts/music-share-images.ts` take a language argument, and run for Italian alone (D4).
- [ ] 2.4 `src/lib/lingua-media.ts`: a built Lingua page's demo video must exist; `test/post-build/media-budget.spec.ts` derives its Lingua pages from the build (D6).
- [ ] 2.5 `PINNED_ROUTES`: `/it/privacy/`, `/it/termini/`, `/it/elimina-account/`, `/it/lingua/arrivederci/` with D3's `pinnedBy` and sources; the file's header names the exception for a path pinned ahead of a client about to compile it (D3).

## 3. The backend

- [ ] 3.1 `backend/server/src/web_lingua.rs`: `thanks_path("it")` → `/it/lingua/arrivederci/#grazie`, with its test; the module doc's anchors and the no-site fallback gain Italian (D8).

## 4. Tests and budget

- [ ] 4.1 `test/post-build/media-budget.spec.ts`: the total at 50 MB (D7); the Italian media within every per-file budget; `apps/site/README.md` (« media ≤ 30 MB », the fr/en/es lists) and `AGENTS.md` follow.
- [ ] 4.2 The tests that name the locales: `alternates.spec` (the switch's regex with `IT`, the families), `pinned-routes.spec` (`it/404.html`), `head.spec`, `locale.spec`, `share-preview.spec` (the Music pages), `lingua.spec` (no `/it/lingua/` while no pair glossed in Italian is listed), `lingua-text.spec` (the capture tests kept at fr, en and es), `lingua-pairs.spec` (its example of a language the site does not speak, `en-it`, replaced), `lingua-leave.spec`, `stores.spec`, `plan-view.spec`, `test/astro/spanish-pages.spec.ts` (the `hreflang` lists of `/es/` and `/es/music/`, its switch regex), `packages/web-auth/test/apple.spec.ts`; `test/astro/italian-pages.spec.ts` (new, as Spanish's); a check holding each Italian legal page to its French page's `updated` date and sections (D2).
- [ ] 4.3 Every French, English and Spanish page's built text byte for byte but its head's Italian alternate and switch (D10).

## 5. Review, deploys, gates

- [ ] 5.1 [agent] The Italian reviewer agent's pass over every Italian text and the slugs, before the pins (D9); its findings applied or answered in the pull request.
- [ ] 5.2 [owner] Merge the backend's release pull request, deploy the backend, then the site, before change 45's release; register `https://cymbra.app/elimina-account` on Apple's Services ID (D8, D5).
- [ ] 5.3 In `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; in `backend`: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, the server's tests; `openspec validate add-site-italian-locale --strict`; the archive order; `docs/lingua/italian-programme.md` row 41's status.
