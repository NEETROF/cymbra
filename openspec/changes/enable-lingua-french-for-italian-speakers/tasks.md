# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main`: changes 45 (released), 46, 44, 47, 48 and 49.

## 1. The list

- [ ] 1.1 `packs.json` gains `"fr-it"` at the end, en-it ahead of it; `check_variants`'s `SHIPPED_PAIRS` the same, its comment naming this change (D1).
- [ ] 1.2 `gloss_coverage.py --write` with fr-it's figures; `test_gloss_coverage.py`: fr-it « published once listed », `ListedFloorsTest` naming it; `committed_tables.rs`: fr-it listed (D1, D2).
- [ ] 1.3 The tests that read the list (D9), each keeping the earlier lists as explicit arguments.

## 2. The extension

- [ ] 2.1 The offer's tests in the Italian interface (D3): `studied-languages-view`, `profile`, `language-labels`, `settings-language`, `onboarding-level-row`, `popup`, `stats-view`, `wordpopup` (« passato remoto »), `engine`, `native-language`, `native-language-view`.
- [ ] 2.2 Translation's tests: `model-residency`, `translation-setting` and `model-controller` (D5), `translate-relay` if change 49 marks fr-it.
- [ ] 2.3 If the owner withholds fr-it's translation (D4): `routes["fr-it"]` dropped, the route texts MODIFIED from their latest holders.

## 3. The description, the activation page and the listings

- [ ] 3.1 `_locales/it/messages.json` `extensionDescription` (D6's draft or the reviewer agent's words), its translator's `description` naming French (fr-it); `yarn check:version`.
- [ ] 3.2 `copy.js` `it` `lede` and `step3` (D6), its comments; `apple-activation-page.spec.ts`: the Italian page names inglese then francese, its labels equal to the extension's.
- [ ] 3.3 The listing files (D7): the Italian texts, the shared English fields, *What the app itself says*, the claims table's rows, each count recorded; the dashboards' procedure for this release.
- [ ] 3.4 [agent] Change 33's Italian reviewer agent reads every new Italian sentence (D6, D7, D8) and change 44's glossary test passes; its verdict in the pull request.

## 4. The site

- [ ] 4.1 `src/data/lingua-coverage.json` from task 1.2; the fixtures refreshed; `test/astro/lingua-page.spec.ts` counting the rows; `lingua-text.spec.ts` and the Italian home-card test with fr-it (D8).
- [ ] 4.2 The Italian captures taken again with French as the second studied language (`STUDIED.it = ["en", "fr"]`): `tool/capture/fixtures/state/it.json` recorded again (`yarn capture --record-seed`), `LINGUA_TEXT.it.shots.second.alt` and its caption rewritten (read in task 3.4), the media within change 41's 50 MB, the capture tool's tests following (D8).
- [ ] 4.3 `yarn test`, `yarn build`, `yarn check:routes`; the Italian Lingua page read at 375 px, not scrolling sideways.

## 5. Checks

- [ ] 5.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn gen:pack:real`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; each zipped package's size in the pull request.
- [ ] 5.2 Nothing else moves: `cargo test -p lingua-pack`, `cargo test -p lingua-wasm` without re-blessing; `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` empty; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 5.3 `openspec validate enable-lingua-french-for-italian-speakers --strict`; the archive order; the MODIFIED blocks re-copied as D13 says: from changes 30 and 31 if both are implemented; otherwise from the holders before them, changes 30 and 31 (or 31 alone) dropped from `archiveAfter`, and change 31's delta amended to carry this change's scenarios and name it; only D12's clauses differing.

## 6. Dogfood and release (owner)

- [ ] 6.1 [manual] The pass of design D10 on the five targets, in the Italian interface.
- [ ] 6.2 [manual] The owner answers the Open Questions, Open Question 2 at change 49's figures.
- [ ] 6.3 [manual] The owner's go-ahead before merge; released on its own, TestFlight first, the site deployed first, the listings pasted after it (D11).
- [ ] 6.4 Change 50 marked done in `docs/lingua/italian-programme.md`.
