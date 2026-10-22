# Tasks

## 1. Before the run

- [ ] 1.1 `tool/marks/stop-words.mjs` and `stop-words.d.mts`: `ITALIAN`, the elided pieces included; the header and the README's language list; `test/translate-measure.spec.ts`'s table check with `it`, its lookup trying the word with an apostrophe, and mapping « all », « dall », « nell », « sull » to `a`, `da`, `in`, `su` (D1).
- [ ] 1.2 `tool/marks/README.md`: the Italian criteria and examples of D3, committed before the results.

## 2. The run

- [ ] 2.1 Assemble the en-it model (`node tool/assemble_model_site.mjs <dir>`); record a one-instance soak beside change 2's isolated one; run `measure_marks.mjs --pair en-it --models <dir>` twice, the two results byte for byte the same; commit `results-en-it.jsonl` (D2).

## 3. The judgment

- [ ] 3.1 [agent] Judge A: every line against the engine's sentence, UD Italian-PUD's human Italian and en-fr's and en-es's verdicts, a reason where both precedents differ (D3).
- [ ] 3.2 [agent] Judge B, change 33's Italian reviewer agent briefed here, in a checkout at the results' commit, blind to A; its verdicts pasted in the pull request's description; the disagreements committed `wrong` with `doubtful: A …/B …`; `judged-en-it.tsv` committed; the agreement count in the README (D3).

## 4. The list, tests and documents

- [ ] 4.1 `MARKED_PAIRS` gains en-it if its committed (strict) figures reach the first tier, the exact lists and a relay case following; otherwise not, and under 75 % or over 30 % the figures named for the owner at change 45 (D4).
- [ ] 4.2 `test/translate-marks.spec.ts`: *An English selection, judged in Italian* (D5); every other pair's results and judgments byte for byte.
- [ ] 4.3 `tool/marks/README.md`, `TRANSLATION.md`, `measure_marks.mjs`'s usage, the comment above `MARKED_PAIRS`; `docs/lingua/italian-programme.md` row 38's figures.

## 5. Gates

- [ ] 5.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`.
- [ ] 5.2 `openspec validate measure-lingua-en-it-translation-marks --strict`; the archive order; the MODIFIED block re-copied as change 24 then stands.
