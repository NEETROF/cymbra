# Tasks

## 1. The list

- [ ] 1.1 `packs.json` `["en-fr", "es-fr", "es-en", "en-es"]` and `check_variants`'s `SHIPPED_PAIRS` (D1).

## 2. Checks

- [ ] 2.1 In `apps/lingua-extension`: `yarn gen:pack` and `yarn gen:pack:real` build every listed pack; `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; the built manifests carry `_locales/es` beside `fr` and `en` (change 27); `cargo test -p lingua-wasm` (the baselines unchanged) (D1, D2).
- [ ] 2.2 `openspec validate enable-lingua-spanish-speakers --strict` passes, and `python3 scripts/openspec_archive_order.py enable-lingua-spanish-speakers` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived).

## 3. Dogfood (owner, with Claude where a session can drive the browser)

- [ ] 3.1 Chrome (macOS), Firefox (macOS), Firefox for Android, Safari (macOS), Safari (iOS): the pass of D3.

## 4. Release (owner)

- [ ] 4.1 The prerequisites of proposal.md (Impact) met, and the owner's go-ahead before merge.
- [ ] 4.2 The packages go to the beta channels with the Spanish listings; each store submission is the owner's.
- [ ] 4.3 Change 35 is marked done in `docs/lingua/language-matrix-programme.md`.
