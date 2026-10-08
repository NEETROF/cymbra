# Tasks

## 1. The catalogue (apps/lingua-extension)

- [x] 1.1 Download en-es's three files once from Mozilla's registry; compute each gzip's sha256 and length and each decompressed file's sha256 and length; check the model against the registry's `uncompressedHash` and the three against Remote Settings' 2.1 (D1).
- [x] 1.2 `model-manifest.json`: `en-es/base-memory/2.1` (paths, sizes, digests, source, mirror, licence) and the routes `es-en` and `en-es` (D1, D2). `assemble_model_site.mjs` run locally over it keeps every file.
- [x] 1.3 `test/model-manifest.spec.ts` holds the new catalogue exactly; `model-residency.spec.ts` and `model-controller.spec.ts` use `en-es/base-memory/2.1`; a test that a French-native reader's needs, downloads and loads are unchanged (D2, D5; *Every reader today*, *A route of a pair not shipped*).

## 2. The soak

- [x] 2.1 `tool/soak_engine.mjs --pair en-es --models <dir> --isolate` over the English corpus with the locally assembled models; the run recorded in `TRANSLATION.md` (D4; change 9's *en-es before it ships*).

## 3. Gates and docs

- [x] 3.1 `TRANSLATION.md` (the routes, the shared vocabulary, the soak); `tool/marks/README.md` (the bound's note, the soak section's « once change 25 pins » lines); `tool/soak_engine.mjs`'s usage comment and refusal message; `REVIEWERS.md` (the three models, and the files now coming from Mozilla's registry, not `mozilla/firefox-translations-models`) (D3, D5).
- [x] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [x] 3.3 `openspec validate add-lingua-translation-matrix-models --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-translation-matrix-models` exits 0; change 25 is marked done in `docs/lingua/language-matrix-programme.md`.

## 4. Owner

- [ ] 4.1 [manual] The owner dispatches `lingua-model-deploy` after the merge (it creates `lingua-model-en-es-base-memory-2.1` and deploys the host); `check_model_host` passes from outside.
