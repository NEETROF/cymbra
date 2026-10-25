# Tasks

Implemented once changes 2 and 19 are: it-en in the catalogue, the Italian selections and the it-en soak committed, and the models assembled with `tool/assemble_model_site.mjs` or served after the owner's redeploy.

## 1. The measurement

- [x] 1.1 The criteria gain the Italian examples in English of D2, committed before the run; `measure.mjs` over the Italian selections through it-en's route; `results-it-en.jsonl` committed (D1).
- [x] 1.2 [manual] One judge judges every shown mark against the English rendering — and the gloss experiment's column if change 22's table filled it; `judged-it-en.tsv` committed; the figures computed by `tier.mjs` and recorded in `tool/marks/README.md`, `TRANSLATION.md` and the programme's row 24.

## 2. The list

- [x] 2.1 `test/translate-marks.spec.ts` « An Italian selection, judged in English »: it-en's committed totals, its tier verdict against whether it is listed, `results("it-en")` with `results("it-fr")`'s ids (D4).
- [x] 2.2 On the first tier, `src/translate/markup.ts` `MARKED_PAIRS` gains `it-en`, the exact lists of `test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow, and `translate-relay.spec.ts` gains a case marking an Italian selection through it-en; short of it, nothing is listed, and under 75 % correct or over 30 % withheld the figures go to the owner (D3).

## 3. The documents

- [x] 3.1 `tool/marks/README.md` (introduction, file list, figures, Judging paragraph, the gloss experiment's row), `TRANSLATION.md` (the measured and listed pairs, it-en's figures, the run's traps) and the comment above `MARKED_PAIRS` (D4).

## 4. Gates

- [x] 4.1 Every other pair's results and judgments byte for byte — `git diff --stat origin/main -- apps/lingua-extension/tool/marks/corpus.json 'apps/lingua-extension/tool/marks/results-*' 'apps/lingua-extension/tool/marks/judged-*' ':!*it-en*'` prints nothing; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [x] 4.2 `openspec validate measure-lingua-it-en-translation-marks --strict` passes; `python3 scripts/openspec_archive_order.py measure-lingua-it-en-translation-marks` names its prerequisites; `docs/lingua/italian-programme.md` row 24's status names this change's pull requests.
