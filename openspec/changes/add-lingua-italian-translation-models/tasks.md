# Tasks

## 1. The catalogue (apps/lingua-extension)

- [x] 1.1 Download it-en 2.0's and en-it 2.1's three files once from Mozilla's registry; compute each gzip's sha256 and length and each decompressed file's sha256 and length; check every decompressed sha256 against Remote Settings' hash for that file at that version, and the models' against the registry's `uncompressedHash` (D1); verify the six digests match and the two vocabularies share one.
- [x] 1.2 `model-manifest.json`: `it-en/base-memory/2.0` and `en-it/base-memory/2.1` (paths, sizes, digests, source, mirror, licence) and the six routes `it-en`, `en-it`, `it-fr`, `it-es`, `fr-it`, `es-it` (D2, D3); verify `node tool/assemble_model_site.mjs <dir>` run locally keeps every file of both models, and that the two vocabulary entries carry one decompressed sha256 and `unpacked` 812,724 (the device stores it once, D3).
- [x] 1.3 `test/model-manifest.spec.ts` holds the new catalogue exactly (six models, twelve routes); its *Every native language's pairs* test asserts the routes into each of fr, en, es and it, three models each and the download sums recomputed from 1.1's gzip sizes; a test over today's shipped pairs shows no reader's pairs need an Italian route and no reader fetches the it-en or en-it model (D5); `test/model-residency.spec.ts`'s placeholder `en-it/base-memory/2.0` becomes `en-it/base-memory/2.1`; verify `yarn test test/model-manifest.spec.ts test/model-residency.spec.ts` passes.

## 2. The soaks

- [x] 2.1 `node --experimental-strip-types tool/soak_engine.mjs --pair en-it --models <dir> --isolate`, then `--pair fr-it` and `--pair es-it`, over the English, French and Spanish corpora with the locally assembled models (D4); the three runs recorded in `TRANSLATION.md` with their figures and the ids that trapped; verify the section names each run's date, route and count.
- [x] 2.2 `TRANSLATION.md` and `tool/marks/README.md` say the it-en soak and the Italian corpus are row 19's (`measure-lingua-it-fr-translation-marks`), and that the soak tool reports no selection for it-en until then; verify both name the row and the change.

## 3. Documents and gates

- [x] 3.1 `TRANSLATION.md` (the six models, the twelve routes, the shared vocabularies; its « four models … two together » notes become six and three, D2), `tool/marks/README.md`'s *Soaking a route* bound paragraph (likewise), `REVIEWERS.md` (« lists four models » becomes six, the models the add-on may download), `tool/soak_engine.mjs`'s usage (the twelve routes); verify each names it-en 2.0 and en-it 2.1 and none still says four models or two together.
- [x] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants` pass.
- [x] 3.3 `openspec validate add-lingua-italian-translation-models --strict` passes and `python3 scripts/openspec_archive_order.py add-lingua-italian-translation-models` names only `add-lingua-translation-matrix-models` and `add-lingua-french-translation` (exit 10 while they are open, 0 once archived); `docs/lingua/italian-programme.md` row 2's Status column names this change's pull request; verify the table's column count is unchanged.

## 4. Owner

- [x] 4.1 [manual] The owner dispatches `lingua-model-deploy` right after the merge (it creates `lingua-model-it-en-base-memory-2.0` and `lingua-model-en-it-base-memory-2.1` and deploys the host); `check_model_host` passes from outside, and the next store submission and App Store delivery are not refused by the model gate (D6).
