## 1. Before any code

- [ ] 1.1 Confirm the licence and attribution terms of `mozilla/firefox-translations-models` for redistributing the `en→fr` `base-memory` files from a Cymbra host; record them in design.md
- [ ] 1.2 Run the mark evaluation corpus on the shipping engine artefact with `reconcile.ts`; record right / wrong / withheld marks; the product owner sets the release threshold from it
- [ ] 1.3 Choose the model's origin (design, Open Questions) and set up its deployment: content-addressed paths, `Cache-Control: immutable`, `Access-Control-Allow-Origin: *`, files as Mozilla publishes them
- [ ] 1.4 Draft the new copy **before** code: the setting row (fr), the Lingua annex (fr + en), the three store listings ("sans compte ni traduction étendue, aucune requête réseau"; page text still never leaves the device)

## 2. The engine in the package

- [ ] 2.1 `build.mjs`: Chromium and Firefox release builds take the engine artefact and fail without it; Safari stays `none`; `offscreen` in the Chromium manifest; no model file in any package
- [ ] 2.2 Bundle `model-manifest.json` (address, compressed size, sha256 of decompressed bytes per file) where the engine ships
- [ ] 2.3 Invert `tool/check_variants.mjs` per variant: engine + manifest in Chromium and Firefox, none in Safari, `offscreen` only in Chromium, no model file anywhere, no code fetched at run time
- [ ] 2.4 Release workflow: download the `lingua-engine-build` artefact for the pinned commit, verify its hash, package it
- [ ] 2.5 AMO source archive: add the engine's reproducible build recipe; `REVIEWERS.md`: the engine, the model download, why no code is fetched

## 3. The setting

- [ ] 3.1 `translationHost: "none" | "local"` in `storage.local`, absent = `"none"`; load/save with tests; never synced
- [ ] 3.2 Settings row « Traduction étendue » with its cost stated before ticking; states: off, downloading (progress + cancel), ready, failed (retry), interrupted (resume), removed (download again); localized messages, no raw errors; tests per state
- [ ] 3.3 Hidden on Firefox for Android (`runtime.getPlatformInfo().os === "android"`) and on Safari; tests
- [ ] 3.4 `createTranslatorPort()` answers from the stored host and the model's readiness instead of the build constant; tests

## 4. Download and storage

- [ ] 4.1 Download in the engine's host (offscreen worker / event-page worker); stream, `DecompressionStream("gzip")`, sha256 against the manifest; a mismatch is discarded; tests with fixtures including a 200 HTML body
- [ ] 4.2 Progress, completion, failure and cancel over the engine channel; settings view keeps the host awake while open; interrupted download resumes only unverified files, never on its own; tests
- [ ] 4.3 `lingua-model` IndexedDB database: decompressed bytes keyed by sha256 + completed-manifest record; `navigator.storage.persist()`; tests
- [ ] 4.4 Engine loads its model from `lingua-model` instead of files beside the bundle; development override kept for side-loading; tests
- [ ] 4.5 Turning the setting off terminates the worker (and the offscreen document) and deletes `lingua-model`; tests
- [ ] 4.6 Setting on with an empty or incomplete database → "removed" state, nothing fetched; tests

## 5. Engine lifetime

- [ ] 5.1 Host records the last translation asked; keep-warm pings do not update it; ten idle minutes → worker terminated (offscreen document closed on Chromium); tests with a fake clock
- [ ] 5.2 No eager load when the setting turns on or the download completes; test

## 6. Surfaces

- [ ] 6.1 Single word: engine asked only when the pack has no gloss and the word is not `ProperNounOutOfLexicon`, with its sentence and the word marked; tests (`disambiguation`, a glossed word, a proper noun)
- [ ] 6.2 No model ready (off, downloading, failed, interrupted, removed): cards answer from the pack with no "translation on its way" line; tests per state
- [ ] 6.3 Update `TRANSLATION.md` for the release path

## 7. Disclosure and release

- [ ] 7.1 Publish the Lingua annex (fr + en) on the site before the first release that can download
- [ ] 7.2 Update the Chrome Web Store and AMO listings (and `STORE-LISTING.md`)
- [ ] 7.3 Update from the published version on Chromium and Firefox: extension stays enabled with the `offscreen` permission added; setting off; no request made
- [ ] 7.4 Manual pass Chrome + Firefox desktop: tick, download, cancel, retry after a cut network, translate a phrase and an unknown single word, idle release after ten minutes, untick deletes the model, offline translation after download
- [ ] 7.5 Manual pass Firefox for Android: no setting, no download, cards unchanged

## 8. Gates

- [ ] 8.1 `openspec validate add-lingua-translation-delivery --strict`
- [ ] 8.2 Extension lint, typecheck, tests and `check:variants` green; coverage ≥ 80 %
