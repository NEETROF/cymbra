# Tasks

Implemented once changes 2, 19 and 38 are on `main` (proposal, Impact).

## 1. Before the run

- [ ] 1.1 `tool/marks/README.md`: the Italian criteria and examples of D3, committed before the results.
- [ ] 1.2 `soak_engine.mjs --pair fr-it` in one instance, beside change 2's isolated soak; the figures in `TRANSLATION.md` (D1).

## 2. The run

- [ ] 2.1 Assemble the fr-en and en-it models; run `measure_marks.mjs --pair fr-it --models <dir>` twice, the two results byte for byte the same; commit `results-fr-it.jsonl`, recording which fr-it gloss table was read, if any (D2).

## 3. The judgment

- [ ] 3.1 [agent] Judge A: every line against fr-en's and fr-es's verdicts on the same selection and UD Italian-PUD's human Italian, a reason where the verdict departs from both precedents; leaked words and marks on punctuation alone judged wrong (D3). Judge A fetches the 100 PUD Italian sentences for Judge B's brief, never committing them.
- [ ] 3.2 [agent] Judge B, change 33's Italian reviewer agent briefed here, in a checkout at the results' commit with this change's directory removed, given the results, the criteria and the PUD Italian, never A's verdicts, the pull request's description or the trial. Its verdicts are pasted in the pull request's description; the disagreements are committed `wrong` with `doubtful: A …/B …`; the doubtful lines are committed `wrong` where they decide the tier (D4); `judged-fr-it.tsv` is committed; the agreement count goes in the README.

## 4. The list, tests and documents

- [ ] 4.1 [owner] The wrong, doubtful and departing lines read against fr-en's and fr-es's anchors, any re-judged; not gating (D3).
- [ ] 4.2 `MARKED_PAIRS` gains fr-it if its committed figures reach the first tier, the exact lists and a relay case (no `SHIPPED_PAIRS` asserted) following; otherwise not, and under 75 % or over 30 % the figures named for the owner at change 50 (D4).
- [ ] 4.3 `test/translate-marks.spec.ts`: *A French selection, through English, judged in Italian* (D5); every other pair's results and judgments byte for byte.
- [ ] 4.4 `tool/marks/README.md`, `TRANSLATION.md`, the usage comments, the comment above `MARKED_PAIRS`; `docs/lingua/italian-programme.md` row 49's figures.

## 5. Gates

- [ ] 5.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`.
- [ ] 5.2 `openspec validate measure-lingua-fr-it-translation-marks --strict`; the archive order; the MODIFIED block re-copied as change 29 then stands.
