# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main` — every stage-1 Italian
change, it-fr's tables committed at or above their floor, change 17 merged (design D2), change 21's
description of Italian on the site (D8) — and change 19's figures are known (D5).

## 1. The list (apps/lingua-extension, scripts/lingua-data, apps/site)

- [ ] 1.1 `packs.json` `["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es", "it-fr"]` and `check_variants`'s `SHIPPED_PAIRS` the same, its comment naming this change (D1).
- [ ] 1.2 `python3 scripts/lingua-data/gloss_coverage.py --write`, committing `apps/site/src/data/lingua-coverage.json` with it-fr's figures; `test_gloss_coverage.py`: it-fr's committed figures « published once listed » (change 16's tests), at or above `FLOORS["it-fr"]` (D2, D8).
- [ ] 1.3 The tests that read the default list or held an Italian pair inert, each keeping the earlier lists it named as explicit arguments (design *Measured*, D1, D5): `packs.spec.ts`, `pairs.spec.ts` (a French speaker's three pairs), `manifests.spec.ts`, `model-manifest.spec.ts` (change 2's « no reader fetches the it-en model » read for the list), `studied-languages-view.spec.ts`; `tool/check_variants.mjs`'s second literal; `README.md`'s quoted list; `crates/lingua-pack/tests/committed_tables.rs`: change 9's *The reference and the pipeline* asserts `packs.json` lists it-fr (D10, D13).

## 2. The extension's Italian (apps/lingua-extension)

- [ ] 2.1 `src/analyzer/types.ts`: `StudiedLanguage = "en" | "es" | "fr" | "it"`; `src/state/profile.ts`: `NAMES` gains `Italian: "it"`; `src/analyzer/language-labels.ts`: `NamedLanguage`'s explicit `"it"` removed; `test/native-language-view.spec.ts`'s test-local `NAMES` gains Italian; every other table the type check names found filled by changes 15 and 18; `yarn typecheck` clean (D3).
- [ ] 2.2 `src/reading/session.ts`: `GUARDED.it` — the neighbours change 8's guard keeps, as its implementation settled them (by default `ca`, `gl`, `vec`, `scn`, `nap`, `sc`, `co`, `fur`), its comment naming `add-lingua-italian-detection-guard`; `THREE_LETTER` gains `ita`, `srd` and `cos`; tests: a page declaring `scn`, `vec` or `srd` reads no card in Italian, a page declaring `fr` still does (D3, the MODIFIED requirement).
- [ ] 2.3 Tests (D3): `profile.spec.ts` — a stored profile studying English then Italian, French native, read as `["en", "it"]`; `studied-languages-view.spec.ts` — English, Spanish and Italian offered to a French speaker, Italian to no English or Spanish speaker; `language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts` — Italian studied in the French interface (« Niveau d'italien estimé », its note); `wordpopup.spec.ts` — an Italian card's studied words carry `lang="it"`. The English and Spanish interfaces' assertions pass unchanged.

## 3. Translation and the memory sentence (apps/lingua-extension)

- [ ] 3.1 The comments that say « until change 20 » in `src/translate/markup.ts` and the routes' tests; it-fr's routes held inert by change 2's tests now read for the list (D5).
- [ ] 3.2 Tests (D6): `model-controller.spec.ts` and `translation-setting.spec.ts` — a French-native reader of Italian alone: en-fr and it-en summed from the catalogue, « environ 340 Mo »; of English, Spanish and Italian: en-fr, es-en and it-en summed, « environ 340 Mo »; every earlier row as before.

## 4. The description, the activation page and the documents

- [ ] 4.1 `_locales/fr/messages.json` `extensionDescription`: English, Spanish and Italian (design D7's draft, or the owner's words); `manifest.json`'s French literal the same; its translator's `description` naming the pairs it follows; `tool/check_version.mjs` `LANGUAGE_NAMES` gains Italian (« italien », "Italian", « italiano »), held to the catalogue by `check-version.spec.ts`; `yarn check:version` (112 characters; `add-lingua-french-listings`'s rule that each description names what its readers study) (D7).
- [ ] 4.2 The host app's activation page (D7): `apps/lingua-apple/Shared (App)/Resources/copy.js` `fr` `lede` and `step3` and `Base.lproj/Main.html`'s two French nodes, the same bytes — design D7's drafts or the owner's words; `apps/lingua-extension/test/apple-activation-page.spec.ts`: the French page names « anglais », « espagnol » and « italien », never « français »; every other key and test as committed.
- [ ] 4.3 The comments and documents that name `enable-lingua-italian` as the future (`git grep -n "enable-lingua-italian"` at implementation — not « change 20 », which names other programmes' changes in the code), `README.md`, `REVIEWERS.md`, `TRANSLATION.md`, `tool/marks/README.md`, `scripts/lingua-data/SOURCES.md` and the Italian tables' READMEs (D10); no table, pin or manifest of `scripts/lingua-data/tables/` moves.

## 5. Checks

- [ ] 5.1 In `apps/lingua-extension`: `yarn gen:pack` and `yarn gen:pack:real` build every listed pair's layer and base; `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the built manifests carry `_locales/fr`, `en` and `es` and no other, `default_locale` English; each zipped package's size in the pull request, against design *Measured*.
- [ ] 5.2 Nothing else moves (design *What moves*): `cargo test -p lingua-wasm` — every baseline and `cross_native.rs` — without re-blessing; `cargo test -p lingua-pack`; `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` empty; the extension's snapshots pass as committed; `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 5.3 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `python3 scripts/lingua-data/gloss_coverage.py --check`. In `apps/site`, with change 21's description of Italian in: `yarn test`, `yarn build`, `yarn check:routes` — the French Lingua page and home card name Italian, its table « Italien → français » (design D8). The `lingua-apple-build` workflow green, `CFBundleLocalizations` `[fr, en, es]` (D7).
- [ ] 5.4 `openspec validate enable-lingua-italian --strict` passes, and `python3 scripts/openspec_archive_order.py enable-lingua-italian` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` still open (0 once they are archived) — `refine-lingua-it-fr-glosses` added once proposed. The MODIFIED blocks re-copied from changes 4's, 9's, `read-lingua-card-in-another-studied-language`'s and `enable-lingua-french`'s requirements as they then stand, only D13's clauses differing (the diff in the pull request), and the archive simulated on a scratch copy of `openspec/`.

## 6. Dogfood (owner, with Claude where a session can drive the browser)

- [ ] 6.1 [manual] Chrome (macOS), Firefox (macOS), Firefox for Android, Safari (macOS), Safari (iOS): the pass of design D11, on test accounts, in the French interface; it carries change 15's on-device checklist (its task 4.1), whose findings go to this pull request; the owner ticks that task once they are in.
- [ ] 6.2 [manual] The owner reads the description's and the activation page's drafts (design D7, M9) and answers Open Questions 1 to 4.

## 7. Release (owner)

- [ ] 7.1 [manual] The prerequisites of proposal.md (Impact) met — among them a release that reads backup schema 4 live on every store (D4) and the model host serving it-en (change 2's task 4.1) — and the owner's go-ahead before merge.
- [ ] 7.2 [manual] Released on its own; TestFlight first; the first store submission carrying Italian carries change 21's listings and the description of task 4.1; the site deployed after the merge, with it-fr's figures, before the listings are pasted; each store submission is the owner's (D12, I12).
- [ ] 7.3 Change 20 is marked done in `docs/lingua/italian-programme.md`.
