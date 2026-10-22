# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main`: changes 30, 31 and 50 (change 50 released, recommended), 51, 52 and 53.

## 1. The list

- [ ] 1.1 `packs.json` gains `"es-it"` at the end, en-it ahead of it; `check_variants`'s `SHIPPED_PAIRS` the same, its comment naming this change (D1).
- [ ] 1.2 `gloss_coverage.py --write` with es-it's figures; `test_gloss_coverage.py`: es-it « published once listed », `ListedFloorsTest` naming it; `committed_tables.rs`: es-it listed (D1, D2).
- [ ] 1.3 The tests that read the list (D9), each keeping the earlier lists as explicit arguments.

## 2. The extension

- [ ] 2.1 The offer's tests in the Italian interface (D3, D9), `wordpopup` naming the pretérito indefinido « passato remoto ».
- [ ] 2.2 Translation's tests (D5): `model-residency`, `translation-setting`, `model-controller`, `translate-relay` if change 53 marks es-it.
- [ ] 2.3 If the owner withholds es-it's translation (D4): `routes["es-it"]` dropped, the route texts MODIFIED from their latest holders.

## 3. The description, the activation page and the listings

- [ ] 3.1 `_locales/it/messages.json` `extensionDescription` (D6), its translator's `description` naming Spanish (es-it); `yarn check:version`.
- [ ] 3.2 `copy.js` `it` `lede` and `step3` (D6); `apple-activation-page.spec.ts`: the Italian page names inglese, francese and spagnolo as listed, its labels equal to the extension's.
- [ ] 3.3 The listing files (D7): the Italian texts, the shared English fields, *What the app itself says*, the claims table's rows, each count recorded; the dashboards' procedure for this release.
- [ ] 3.4 [agent] Change 33's Italian reviewer agent reads every new Italian sentence (D6–D8); change 44's glossary test passes; its verdict in the pull request.

## 4. The site

- [ ] 4.1 `src/data/lingua-coverage.json` from task 1.2; the fixtures refreshed; `test/astro/lingua-page.spec.ts` counting the rows; `lingua-text.spec.ts` and the Italian home-card test with es-it (D8).
- [ ] 4.2 Where fr-it is not listed: the Italian captures taken again with Spanish, the seed recorded again, the alt text rewritten, the media within 50 MB (D8).
- [ ] 4.3 `yarn test`, `yarn build`, `yarn check:routes`; the Italian Lingua page read at 375 px, not scrolling sideways.

## 5. Checks

- [ ] 5.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn gen:pack:real`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; each zipped package's size in the pull request.
- [ ] 5.2 Nothing else moves: `cargo test -p lingua-pack`, `cargo test -p lingua-wasm` without re-blessing; `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` empty; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 5.3 `openspec validate enable-lingua-spanish-for-italian-speakers --strict`; the archive order; the MODIFIED blocks re-copied from change 50 as it then stands (from its sources if it is withdrawn, D13), only D12's clauses differing.

## 6. Dogfood and release (owner)

- [ ] 6.1 [manual] The pass of design D10 on the five targets, in the Italian interface.
- [ ] 6.2 [manual] The owner answers the Open Questions, Open Question 2 at change 53's figures.
- [ ] 6.3 [manual] The owner's go-ahead before merge; released on its own, TestFlight first, the site deployed first, the listings pasted after it (D11).
- [ ] 6.4 Change 54 marked done in `docs/lingua/italian-programme.md`.
