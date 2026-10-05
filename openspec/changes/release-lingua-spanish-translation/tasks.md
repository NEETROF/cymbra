# Tasks

## 1. The measurement

- [ ] 1.1 The selection rule (design D1) writes `apps/lingua-extension/tool/marks/corpus.json`. It fetches PUD English and Spanish at their pinned commits and checks them by sha256. The output is 100 selections per language: the same sentence ids in both, the parts of speech cycled noun, verb, noun, adjective.
- [ ] 1.2 `tool/measure_marks.ts` (design D2): the pinned engine, the catalogue's models checked by sha256, each selection marked as `relay.ts` marks it, through the language's route. It writes `tool/marks/results-en.jsonl` and `results-es.jsonl`, the gloss-located mark included (design D5).
- [ ] 1.3 Every mark judged with design D3's criteria, in `tool/marks/judged-en.tsv` and `judged-es.tsv`. A reason is given for each wrong mark. `tool/marks/README.md` states the rates (design D4) for the engine's marks and the experiment's.

## 2. The rule

- [ ] 2.1 D2 applied (design D4). On the first tier, `es` joins `MARKED_LANGUAGES`, with its test. Otherwise Spanish stays unmarked, or the result goes to the owner.

## 3. Checks

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn typecheck`, `yarn format:check`, `yarn test`.
- [ ] 3.2 `openspec validate release-lingua-spanish-translation --strict` passes.
- [ ] 3.3 In `docs/lingua/spanish-programme.md`, change 27 is marked done, with the rates.
