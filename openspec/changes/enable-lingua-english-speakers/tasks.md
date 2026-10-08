# Tasks

## 1. The list

- [x] 1.1 `packs.json` `["en-fr", "es-fr", "es-en"]` and `check_variants`'s `SHIPPED_PAIRS` (D1); `python3 scripts/lingua-data/gloss_coverage.py --write`, committing `apps/site/src/data/lingua-coverage.json` with es-en's figures (change 21 D6; `test_gloss_coverage.py` holds the file to `packs.json`, and change 30's pages read their pairs from it); `test/pairs.spec.ts` *for every reader today* rewritten for a list with a second native, and every test that asserts a French-only build from the default list (change 20's *Every reader today*) passing its list explicitly.
- [x] 1.2 `apps/site` `/en/lingua/`: the card's three buttons quoted from the English catalogue (`apps/lingua-extension/src/i18n/en/card.ts`) instead of « Je connais », « + Deck », « Ignorer » (handed over by change 30).
  The same card's "its French translation" goes too (the translation is in the reader's language); `apps/site/test/lingua-text.spec.ts` fails while either is left with es-en listed.
- [x] 1.3 If the owner settles under M15 not to offer es-en's translation, the `es-en` route is removed from `model-manifest.json` (the model stays, as es-fr's pivot), so the pair is unavailable as *A pair without a route* says (D5).
  - M15 settled on 2026-10-09: translation opens with each pair (the engine hardening, change 9, merged), es-en in `MARKED_PAIRS` — so the `es-en` route stays and nothing is removed.

## 2. Checks

- [x] 2.1 In `apps/lingua-extension`: `yarn gen:pack` and `yarn gen:pack:real` build every listed pack; `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the built manifests carry `_locales/fr` and `_locales/en` and no other, `default_locale` English (change 27, M13); `cargo test -p lingua-wasm` (the baselines unchanged) (D1, D2). `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`. In `apps/site`: `yarn test`, `yarn build`, `yarn check:routes`. The `lingua-apple-build` workflow green.
  - The workflow's steps were run locally (`swift test`, the release tools' tests, `test_app_localizations.sh`, the unsigned iOS simulator and macOS builds, its verification step: both apps declare `fr` and `en`); the workflow itself runs on the pull request.
- [x] 2.2 `openspec validate enable-lingua-english-speakers --strict` passes, and `python3 scripts/openspec_archive_order.py enable-lingua-english-speakers` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived).

## 3. Dogfood (owner, with Claude where a session can drive the browser)

- [ ] 3.1 Chrome (macOS), Firefox (macOS), Firefox for Android, Safari (macOS), Safari (iOS): the pass of D3, on test accounts.

## 4. Release (owner)

- [ ] 4.1 The prerequisites of proposal.md (Impact) met, and the owner's go-ahead before merge.
- [ ] 4.2 TestFlight first. The first store submission carrying `_locales` carries change 36's listings (M13): on the Chrome Web Store that package is uploaded without publishing, the French listing re-entered under `fr` and the English filled under `en`, then submitted; the owner checks both dashboards after it and records the result in `STORE-LISTING.md` (change 27's risk). The site deployed after the merge, with es-en's figures, before the listings are pasted; each store submission is the owner's (D4).
- [ ] 4.3 Change 34 is marked done in `docs/lingua/language-matrix-programme.md`.
