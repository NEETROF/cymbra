# Tasks

Implemented once change 2 is: it-en in the catalogue, it-fr routed, and the models assembled with
`tool/assemble_model_site.mjs` or served after the owner's redeploy.

## 1. The corpus (apps/lingua-extension/tool/marks)

- [x] 1.1 `pud.mjs`: `PUD.it` at `71d932aa…` and its sha256 (D1); `select_corpus.mjs`: `LANGUAGES` gains `it`; `corpus.json` re-written — 100 Italian selections added after each step's French one, the 300 others byte for byte; `test/translate-marks.spec.ts`: Italian in the corpus's languages, in each step's languages and in the letters-only check, each step's Italian selection from its English one's sentence.

## 2. The soaks and the measurement

- [x] 2.1 [manual] `tool/soak_engine.mjs --pair it-en` and `--pair it-fr` over the 100 Italian selections, each `--isolate` and in one instance (D2); recorded in `TRANSLATION.md`, replacing change 2's « the it-en soak is row 19's » there and in `tool/marks/README.md`.
- [x] 2.2 The criteria gain the Italian examples of D3, committed before the run; `measure.mjs` over the Italian selections through it-fr's route; `results-it-fr.jsonl` committed.
- [ ] 2.3 [manual] One judge judges every shown mark against the French rendering; `judged-it-fr.tsv` committed; the figures computed by `tier.mjs` and recorded in `tool/marks/README.md` and the programme's row 19.

## 3. The list

- [x] 3.1 `test/translate-marks.spec.ts`'s structural test reads the list as the judged pairs on the tier — every listed pair judged, a judged pair listed exactly when on the tier — and its per-pair checks of results and judgments run over every judged pair (D4). On the first tier, `src/translate/markup.ts` `MARKED_PAIRS` gains `it-fr`, and the exact lists of `test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow; short of it, nothing is listed, and under 75 % correct or over 30 % withheld the figures go to the owner (D4).

## 4. The documents

- [x] 4.1 `tool/marks/README.md`, `pud.mjs`'s and `select_corpus.mjs`'s header comments and closing count, `measure_marks.mjs`'s and `soak_engine.mjs`'s usage, the comment above `MARKED_PAIRS`, `TRANSLATION.md` (D5); `git grep -n "English, Spanish and French\|three treebanks\|row 19's\|300 selections\|step's three\|fr-en or fr-es\|and fr-es through English" apps/lingua-extension` returns no line.

## 5. Gates

- [x] 5.1 The six measured pairs' results and judgments are byte for byte; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 5.2 `openspec validate measure-lingua-it-fr-translation-marks --strict` passes; `python3 scripts/openspec_archive_order.py measure-lingua-it-fr-translation-marks` names its prerequisites; `docs/lingua/italian-programme.md` row 19's status names this change's pull requests.
