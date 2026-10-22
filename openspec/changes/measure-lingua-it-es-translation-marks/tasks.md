# Tasks

## 1. Before the run

- [ ] 1.1 `tool/marks/README.md`: the Spanish criteria and examples of D3, committed before the results.
- [ ] 1.2 `soak_engine.mjs --pair it-es`, isolated and in one instance; the figures in `TRANSLATION.md` (D1).

## 2. The run

- [ ] 2.1 Assemble the it-en and en-es models; run `measure_marks.mjs --pair it-es --models <dir>` twice, the two results byte for byte the same; commit `results-it-es.jsonl`, recording which it-es gloss table was read (D2).

## 3. The judgment

- [ ] 3.1 [agent] One judge: every line against it-fr's and it-en's verdicts on the same selection and en-es's Spanish for the English sentence, a reason where both precedents differ; leaked words judged wrong; the doubtful lines named, committed `wrong` where they decide the tier (D3, D4); `judged-it-es.tsv` committed.
- [ ] 3.2 [agent] If the strict figures are under 96 % correct: a blind second judge reading Spanish, by change 38's protocol, its verdicts pasted in the pull request, the decision recorded there (Risks).

## 4. The list, tests and documents

- [ ] 4.1 [owner] The wrong, doubtful and departing lines read, any re-judged; not gating (D3).
- [ ] 4.2 `MARKED_PAIRS` gains it-es if its committed figures reach the first tier, the exact lists and a relay case (no `SHIPPED_PAIRS` asserted) following; otherwise not, and under 75 % or over 30 % the figures named for the owner at change 30 (D4).
- [ ] 4.3 `test/translate-marks.spec.ts`: *An Italian selection, through English, judged in Spanish* (D5); every other pair's results and judgments byte for byte.
- [ ] 4.4 `tool/marks/README.md`, `TRANSLATION.md`, the usage comments, the comment above `MARKED_PAIRS`; `docs/lingua/italian-programme.md` row 29's figures.

## 5. Gates

- [ ] 5.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`.
- [ ] 5.2 `openspec validate measure-lingua-it-es-translation-marks --strict`; the archive order; the MODIFIED block re-copied as change 38 then stands.
