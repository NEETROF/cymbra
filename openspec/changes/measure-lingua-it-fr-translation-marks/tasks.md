# Tasks

## 1. The corpus (apps/lingua-extension/tool/marks)

- [ ] 1.1 `pud.mjs`: `PUD.it` at the commit and sha256 change 9's `measure/it-ud.sh` pins (D1); `select_corpus.mjs`: `LANGUAGES` gains `it`; `corpus.json` re-written — 100 Italian selections added, the 300 others byte for byte; `test/translate-measure.spec.ts` holds both.

## 2. The soak and the measurement

- [ ] 2.1 [manual] The soak over the 100 Italian selections through it-fr's route, isolated and in one instance (D2); recorded in `TRANSLATION.md`.
- [ ] 2.2 The criteria gain the Italian examples (D3), committed before the run; `measure.mjs` over the Italian selections through it-fr's route; `results-it-fr.jsonl` committed.
- [ ] 2.3 [manual] One judge judges every shown mark against the French rendering; `judged-it-fr.tsv` committed; the figures computed by `tier.mjs` and recorded in `tool/marks/README.md` and the programme's row 19.

## 3. The list

- [ ] 3.1 On the first tier, `src/translate/markup.ts` `MARKED_PAIRS` gains `it-fr`, and the test that holds the list follows; short of it, nothing is listed and the figures go to the owner (D4).

## 4. Gates

- [ ] 4.1 The six measured pairs' results and judgments are byte for byte; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 4.2 `openspec validate measure-lingua-it-fr-translation-marks --strict` passes; `python3 scripts/openspec_archive_order.py measure-lingua-it-fr-translation-marks` names its prerequisites; `docs/lingua/italian-programme.md` row 19's status names this change's pull requests.
