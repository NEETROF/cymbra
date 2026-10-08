# Tasks

## 1. The harness (apps/lingua-extension/tool)

- [ ] 1.1 `tool/marks/stop-words.mjs`: the French set as today, an English and a Spanish set; `measure_marks.mjs` picks the pair's native language's (D1). Test: no content word of its own language in any set.
- [ ] 1.2 The result key `translation`; `results-en-fr.jsonl` and `results-es-fr.jsonl` rewritten with the key renamed, checked once with `git diff --word-diff` in the pull request (D1).
- [ ] 1.3 `tool/marks/measure.mjs`: the per-selection loop taking the engine factory; each trapping request asked once more on a fresh engine; a sentence trapped twice recorded `trapped: true` and judged `withheld`/`trapped twice`; a fragment trapped twice leaves the marks unreconciled (D2). Test with a fake engine that traps.
- [ ] 1.4 The gloss experiment filled only when `tables/<pair>/gloss.tsv` exists (D3).

## 2. The measurements

- [ ] 2.1 The README's criteria rewritten for any native language, before the run (D4).
- [ ] 2.2 `measure_marks.mjs --pair es-en` and `--pair en-es` with the models assembled locally; `results-es-en.jsonl`, `results-en-es.jsonl` committed.
- [ ] 2.3 `judged-es-en.tsv`, `judged-en-es.tsv` judged and committed; the figures in the README (its stale lines: "es-en, whose one model is measured when it ships", "models en-fr and es-en base-memory 2.0") and the programme's table; `measure_marks.mjs`'s comments naming en-fr and es-fr alone (D4).
- [ ] 2.4 `MARKED_PAIRS` gains each pair on the first tier; `test/translate-marks.spec.ts` holds each measured pair's totals and tier, and `test/translate-relay.spec.ts`'s es-en/en-es cases follow the list (D5; *A pair measured in another native language*, *An English-native reader of Spanish*, *A selection the engine traps on*); a pair under 75 % correct or over 30 % withheld reported to the owner (M15).

## 3. Gates and docs

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.2 `TRANSLATION.md` names the measured pairs; `openspec validate measure-lingua-translation-matrix-marks --strict` passes, and `python3 scripts/openspec_archive_order.py measure-lingua-translation-matrix-marks` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 26 is marked done in `docs/lingua/language-matrix-programme.md`.
