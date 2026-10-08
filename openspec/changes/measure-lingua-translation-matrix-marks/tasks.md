# Tasks

## 1. The harness (apps/lingua-extension/tool)

- [ ] 1.1 `tool/marks/stop-words.mjs`: the French set as today, an English and a Spanish set; `measure_marks.mjs` picks the pair's native language's (D1). Test: no content word of the corpus's selections in any set.
- [ ] 1.2 The result key `translation`; `results-en-fr.jsonl` and `results-es-fr.jsonl` rewritten with the key renamed (D1). Test: every value equals the previous file's.
- [ ] 1.3 A trap restarts the engine and asks once more; a second trap records `trapped: true` and counts as withheld (D2). Test with a fake engine that traps.
- [ ] 1.4 The gloss experiment filled only when `tables/<pair>/gloss.tsv` exists (D3).

## 2. The measurements

- [ ] 2.1 The README's criteria rewritten for any native language, before the run (D4).
- [ ] 2.2 `measure_marks.mjs --pair es-en` and `--pair en-es` with the models assembled locally; `results-es-en.jsonl`, `results-en-es.jsonl` committed.
- [ ] 2.3 `judged-es-en.tsv`, `judged-en-es.tsv` judged and committed; the figures in the README and the programme's table (D4).
- [ ] 2.4 `MARKED_PAIRS` gains each pair on the first tier; `test/translate-marks.spec.ts` holds each measured pair's totals and tier (D5; *An English-native reader of Spanish*, *A Spanish-native reader of English*).

## 3. Gates and docs

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.2 `TRANSLATION.md` names the measured pairs; `openspec validate measure-lingua-translation-matrix-marks --strict` passes, and `python3 scripts/openspec_archive_order.py measure-lingua-translation-matrix-marks` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 26 is marked done in `docs/lingua/language-matrix-programme.md`.
