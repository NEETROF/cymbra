# Tasks

Implemented once changes 2 and 19 are: it-en in the catalogue, the Italian selections and the it-en soak committed, and the models assembled with `tool/assemble_model_site.mjs` or served after the owner's redeploy.

## 1. The measurement

- [ ] 1.1 The criteria gain the Italian examples in English of D2, committed before the run; `measure.mjs` over the Italian selections through it-en's route; `results-it-en.jsonl` committed (D1).
- [ ] 1.2 [manual] One judge judges every shown mark against the English rendering; `judged-it-en.tsv` committed; the figures computed by `tier.mjs` and recorded in `tool/marks/README.md`, `TRANSLATION.md` and the programme's row 24.

## 2. The list

- [ ] 2.1 On the first tier, `src/translate/markup.ts` `MARKED_PAIRS` gains `it-en`, and the exact lists of `test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow; short of it, nothing is listed, and under 75 % correct or over 30 % withheld the figures go to the owner (D3).

## 3. The documents

- [ ] 3.1 `tool/marks/README.md`, `TRANSLATION.md` and the comment above `MARKED_PAIRS` give it-en's figures (D4).

## 4. Gates

- [ ] 4.1 Every other pair's results and judgments byte for byte; in `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 4.2 `openspec validate measure-lingua-it-en-translation-marks --strict` passes; `python3 scripts/openspec_archive_order.py measure-lingua-it-en-translation-marks` names its prerequisites; `docs/lingua/italian-programme.md` row 24's status names this change's pull requests.
