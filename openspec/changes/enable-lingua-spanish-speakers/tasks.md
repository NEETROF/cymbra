# Tasks

## 1. The list

- [x] 1.1 `packs.json` `["en-fr", "es-fr", "es-en", "en-es"]` and `check_variants`'s `SHIPPED_PAIRS` (D1); `python3 scripts/lingua-data/gloss_coverage.py --write`, committing `apps/site/src/data/lingua-coverage.json` with en-es's figures (`test_gloss_coverage.py` holds the file to `packs.json`, and change 30's pages read their pairs from it; `/es/lingua/` is built only when a Spanish-glossed pair is in that file, change 30 D3).

## 2. Checks

- [x] 2.1 In `apps/lingua-extension`: `yarn gen:pack` and `yarn gen:pack:real` build every listed pack; `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the built manifests carry `_locales/fr`, `_locales/en` and `_locales/es` and no other (change 27); `cargo test -p lingua-wasm` (the baselines unchanged) (D1, D2). `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`. In `apps/site`: `yarn test`, `yarn build` (`/es/lingua/` built), `yarn check:routes`. The `lingua-apple-build` workflow green.
  - The workflow's steps were run locally (`swift test`, the release tools' tests, `test_app_localizations.sh`, the unsigned iOS simulator and macOS builds, its verification step: both apps and both extensions declare `fr`, `en` and `es`); the workflow itself runs on the pull request.
- [x] 2.2 `openspec validate enable-lingua-spanish-speakers --strict` passes, and `python3 scripts/openspec_archive_order.py enable-lingua-spanish-speakers` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived).

## 3. Dogfood (owner, with Claude where a session can drive the browser)

- [ ] 3.1 Chrome (macOS), Firefox (macOS), Firefox for Android, Safari (macOS), Safari (iOS): the pass of D3, on test accounts.

## 4. Release (owner)

- [ ] 4.1 The prerequisites of proposal.md (Impact) met, and the owner's go-ahead before merge.
- [ ] 4.2 TestFlight first; the first store submission carrying `_locales/es` carries change 37's listings (D4). The site deployed after the merge, with en-es's figures, so that `/es/lingua/` appears, before the listings are pasted; each store submission is the owner's.
- [ ] 4.3 Change 35 is marked done in `docs/lingua/language-matrix-programme.md`.
