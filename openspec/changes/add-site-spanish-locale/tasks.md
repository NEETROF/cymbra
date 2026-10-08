# Tasks

## 1. The locale (apps/site)

- [ ] 1.1 `astro.config.mjs` `es`; `localised404()` for every non-default locale; `test/post-build/pinned-routes.spec.ts` expects `es/404.html` (D1).
- [ ] 1.2 `Base.astro` and `Legal.astro` take `alternates`; every page passes its own; `hreflang` with `x-default`; the nav and footer per language (D2). Tests: the switch and `hreflang` of a legal page in each language.

## 2. The pages

- [ ] 2.1 `src/pages/es/{privacidad,terminos,soporte,eliminar-cuenta,404}`: translations of the French pages, drafts (D1).
- [ ] 2.2 `src/lib/i18n.ts` `es` (94 keys), `formatDate` `es-ES`; `packages/web-auth` Apple locale `es_ES`, `appleReturnUrl` and its tests (D3).

## 3. Pins and the extension's link

- [ ] 3.1 `pinned-routes.ts`: `/suppression-compte/`, `/en/delete-account/`, `/es/eliminar-cuenta/` (D4).
- [ ] 3.2 `apps/lingua-extension/src/account/flow.ts` `deleteAccountUrl`: Spanish → `/es/eliminar-cuenta/`; its test (D4).

## 4. Gates, review and docs

- [ ] 4.1 In `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; in `packages/web-auth`: its tests; in `apps/lingua-extension`: `yarn lint`, `yarn typecheck`, `yarn test`.
- [ ] 4.2 [manual] The owner reviews the Spanish pages (M9) — the legal texts with counsel if he wishes — and settles M11 on this pull request.
- [ ] 4.3 [manual] The owner deploys the site (`site-deploy`) before a release that links a Spanish page, and registers `/es/eliminar-cuenta` with Apple's Services ID if its sign-in needs it (D5).
- [ ] 4.4 `apps/site/README.md` (three locales, `alternates`, the deploy order); `openspec validate add-site-spanish-locale --strict` passes, and `python3 scripts/openspec_archive_order.py add-site-spanish-locale` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 29 is marked done in `docs/lingua/language-matrix-programme.md`.
