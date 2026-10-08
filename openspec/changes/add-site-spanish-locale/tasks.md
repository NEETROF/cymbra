# Tasks

## 1. The locale (apps/site)

- [ ] 1.1 `astro.config.mjs` `es`; `localised404()` for every non-default locale; `test/post-build/pinned-routes.spec.ts` expects `es/404.html` (D1).
- [ ] 1.2 `Base.astro` and `Legal.astro` take `alternates`; every page passes its own; `hreflang` with `x-default`; the nav and footer per language (D2). Tests: the switch and `hreflang` of a legal page in each language.

## 2. The pages

- [ ] 2.1 `src/pages/es/{privacidad,terminos,soporte,eliminar-cuenta,404}`: translations of the French pages, drafts (D1).
- [ ] 2.2 `src/lib/i18n.ts` `es` (94 keys), `formatDate` `es-ES`; `packages/web-auth/src/apple.ts` `SDK_LOCALE.es = 'es_ES'`; `apps/site/src/lib/plan-view.ts` `appleReturnUrl` strips `/es`, and `apps/site/test/plan-view.spec.ts` (D3).

## 3. Pins and the extension's link

- [ ] 3.1 `pinned-routes.ts`: `/suppression-compte/`, `/lingua/`, `/es/eliminar-cuenta/`; `/en/delete-account/`'s `pinnedBy` names Lingua; the README's stale sentence removed (D4).
- [ ] 3.2 After change 17's `deleteAccountUrl(interfaceLanguage)` has merged: Spanish → `/es/eliminar-cuenta/` (in `account/flow.ts`, or `account/locale.ts` if change 17 moved it); its test (D4).

## 4. Gates, review and docs

- [ ] 4.1 In `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; in `packages/web-auth`: its tests; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the Spanish pages (M9) — the legal texts with counsel if he wishes, and a prevailing-language clause, absent from the English terms too — and settles M11 on this pull request: `/es/terminos/` goes beyond M11's minimum and change 32's footer needs it; the slugs (`privacidad`, `terminos`, `soporte`, `eliminar-cuenta`) become permanent once pinned; the support page is Music's today, so its translation serves no Lingua reader until it covers Lingua.
- [ ] 4.3 [manual] The owner deploys the site before a release that links a Spanish page, and registers `https://cymbra.app/eliminar-cuenta` (what `appleReturnUrl` sends from `/es/eliminar-cuenta/`) on the Services ID, checking `https://cymbra.app/suppression-compte` and `https://cymbra.app/delete-account` are registered too; `.env.example` and the README list every Return URL (D3, D5).
- [ ] 4.4 `apps/site/README.md` (three locales, `alternates`, the deploy order); `openspec validate add-site-spanish-locale --strict` passes, and `python3 scripts/openspec_archive_order.py add-site-spanish-locale` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 29 is marked done in `docs/lingua/language-matrix-programme.md`.
