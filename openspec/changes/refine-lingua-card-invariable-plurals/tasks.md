# Tasks

Implemented on `main` as it stands, before change 51's implementation (recommended) or after it (design D3).

## 1. The description (apps/lingua-extension)

- [ ] 1.1 `src/reading/grammar-description.ts`: an exported predicate over a tag — a noun's, proper noun's, adjective's, determiner's or pronoun's plural without a degree, whatever its gender — and `describeReadings` leaving it out, with `isDictionaryForm`'s tags, on the card of a form spelled like its dictionary form (`same`); the comments of `describeReadings`, `FormDescription.own` and the module say the dictionary form's own card leaves out a plural spelled like it, for every studied language (D1, D2). `formKind`, `nameReadings`, `composeLines` and the three renderers unchanged. If change 51's implementation is already on `main`: its `CARD_NAMES` entry for the plural, its `composeLines` arm and their tests removed, French's cards byte for byte (D3); if not, its pull request reads its tasks 1.1, 1.3 and 5.1 as D3 lists.
- [ ] 1.2 `test/grammar-description.spec.ts`: on a form spelled like its dictionary form, a noun's plural of each gender, an adjective's, a pronoun's and a determiner's plural left out of `own`; kept when the form differs (`rápidas`, `casas`), kept in `others` (`leaves` → `leaf`); a comparative plural kept; a verb reading beside the plural kept (`paso` → `pasar`); *What each renderer names* gains: a plural spelled like its dictionary form gets no line from the French, English or Spanish renderer, for English and for Spanish studied (D7).
- [ ] 1.3 `test/word-grammar-en.spec.ts` and `-es.spec.ts`: « crisis » and "police" give no line; « paso » keeps its line about `pasar`, « gafas » its line about `gafa`; "leaves" still says the plural of "leaf". `test/word-grammar.spec.ts` unchanged and green (D7).

## 2. The snapshot and the measurement

- [ ] 2.1 `yarn vitest run test/word-card-es-en.spec.ts -u` (the flag after the file): `test/baseline/word-card-es-en.txt` loses exactly « grammar: may also be the masculine plural of menos » under `### word-grammar menos menos`, nothing else; `word-card-en-es.spec.ts`, `row-gloss-tables.spec.ts` (`selection-rows-fr.txt`) and the voice ranking pass without `-u` (D4).
- [ ] 2.2 The design's whole-pack comparison re-run on the implementation — every form of en-fr, es-fr, es-en and en-es, its dictionary form resolved as the page analysis does, rendered through `main`'s description and this change's in each interface language — giving D4's figures (194 English and 667 Spanish forms, each losing one line, nothing else of the 440,534); the counts in the pull request, and every moved card's French line before and after, grouped by studied language and ordered by rank, for task 4.1 (D5).

## 3. Gates

- [ ] 3.1 Nothing else moves: `git diff --stat origin/main -- crates scripts/lingua-data apps/lingua-extension/test/baseline/word-card-en-es.txt apps/lingua-extension/test/baseline/selection-rows-fr.txt apps/lingua-extension/test/baseline/voice-ranking.txt apps/lingua-extension/test/word-grammar.spec.ts` is empty; `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline` passes without re-blessing.
- [ ] 3.2 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (coverage ≥ 80 %), `yarn build` (the bundles' growth against `main`, every target, in the pull request), `yarn check:variants`.
- [ ] 3.3 `openspec validate refine-lingua-card-invariable-plurals --strict` passes; `python3 scripts/openspec_archive_order.py refine-lingua-card-invariable-plurals` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` still open (0 once they are archived); row 51b of `docs/lingua/language-matrix-programme.md` says where the change stands.

## 4. Owner

- [ ] 4.1 [manual] The owner approves, before the merge, the moved lines the pull request lists: the 194 en-fr and 667 es-fr cards in French — the programme's rule « en-fr and es-fr output does not move, nor the French interface » — and the es-en snapshot's line (M9) (D5).
- [ ] 4.2 [manual] The owner releases the extension (Chrome Web Store, addons.mozilla.org, the Safari host app) with the change.
