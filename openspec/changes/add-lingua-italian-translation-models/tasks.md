# Tasks

## 1. The catalogue (apps/lingua-extension)

- [ ] 1.1 Download it-en 2.0's and en-it 2.1's three files once from Mozilla's registry; compute each gzip's sha256 and length and each decompressed file's sha256 and length; check every decompressed sha256 against Remote Settings' hash for that file at that version, and the models' against the registry's `uncompressedHash` (D1); verify the six digests match and the two vocabularies share one.
- [ ] 1.2 `model-manifest.json`: `it-en/base-memory/2.0` and `en-it/base-memory/2.1` (paths, sizes, digests, source, mirror, licence) and the six routes `it-en`, `en-it`, `it-fr`, `it-es`, `fr-it`, `es-it` (D2, D3); verify `node --experimental-strip-types tool/assemble_model_site.mjs` run locally keeps every file of both models and stores the shared vocabulary once.
- [ ] 1.3 `test/model-manifest.spec.ts` holds the new catalogue exactly (six models, twelve routes), and a test over today's shipped pairs shows no reader's pairs need an Italian route and no reader fetches the it-en or en-it model (D5); verify `yarn test test/model-manifest.spec.ts` passes.

## 2. The soaks

- [ ] 2.1 `tool/soak_engine.mjs --pair en-it --models <dir> --isolate`, then `--pair fr-it` and `--pair es-it`, over the English, French and Spanish corpora with the locally assembled models (D4); the three runs recorded in `TRANSLATION.md` with their figures and the ids that trapped; verify the section names each run's date, route and count.
- [ ] 2.2 `TRANSLATION.md` and `tool/marks/README.md` say the it-en soak and the Italian corpus are row 17's (`measure-lingua-it-fr-translation-marks`); verify both name the row.

## 3. Documents and gates

- [ ] 3.1 `TRANSLATION.md` (the six models, the twelve routes, the shared vocabularies, the three-models-per-native note of D2), `REVIEWERS.md` (the models the add-on may download), `tool/soak_engine.mjs`'s usage (the twelve routes); verify each names it-en 2.0 and en-it 2.1.
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants` pass.
- [ ] 3.3 `openspec validate add-lingua-italian-translation-models --strict` passes and `python3 scripts/openspec_archive_order.py add-lingua-italian-translation-models` exits 0; `docs/lingua/italian-programme.md` row 2 names this change's pull request; verify the programme table still parses.

## 4. Owner

- [ ] 4.1 [manual] The owner dispatches `lingua-model-deploy` right after the merge (it creates `lingua-model-it-en-base-memory-2.0` and `lingua-model-en-it-base-memory-2.1` and deploys the host); `check_model_host` passes from outside, and the next release is not refused by the model gate (D6).
