## 1. Before any code

- [x] 1.1 Confirm the licence and attribution terms of `mozilla/firefox-translations-models` for redistributing the `en→fr` `base-memory` files from a Cymbra host; record them in design.md
- [x] 1.2 Run the mark evaluation corpus on the shipping engine artefact with `reconcile.ts`; record right / wrong / withheld marks; the product owner sets the release threshold from it
  - Threshold set by the product owner on 2026-09-26 from #532's measurement, as it stands: **83 of 87 marks right (95 %)** on the sentences the model translated correctly, 0 made wrong by the check. The 13 mistranslations (9 of 10 idioms rendered word for word, 4 sentences) are the model's and are accepted: the card labels the answer « traduction automatique » and shows the pack's gloss first. Not re-run on the pinned artefact: the corpus was judged by hand and is not committed.
- [x] 1.3 Choose the model's origin (design, Open Questions) and set up its deployment: content-addressed paths, `Cache-Control: immutable`, `Access-Control-Allow-Origin: *`, files as Mozilla publishes them
  - Done 2026-09-26: Cloudflare Pages project `cymbra-models` (Direct Upload), custom domain `models.cymbra.app` (CNAME, proxied), repo variable `CF_PAGES_MODELS_PROJECT`; `lingua-model-deploy` green (mirror release `lingua-model-en-fr-base-memory-2.0` created), `check_model_host.mjs` OK from outside.
- [x] 1.4 Draft the new copy **before** code: the setting row (fr), the Lingua annex (fr + en), the three store listings ("sans compte ni traduction étendue, aucune requête réseau"; page text still never leaves the device)

## 2. The engine in the package

- [x] 2.1 `build.mjs`: Chromium and Firefox release builds take the engine artefact and fail without it; Safari stays `none`; `offscreen` in the Chromium manifest; no model file in any package
- [x] 2.2 Bundle `model-manifest.json` (address, compressed size, sha256 of decompressed bytes per file) where the engine ships
- [x] 2.3 Invert `tool/check_variants.mjs` per variant: engine + manifest in Chromium and Firefox, none in Safari, `offscreen` only in Chromium, no model file anywhere, no code fetched at run time
- [x] 2.4 Release workflow: download the `lingua-engine-build` artefact for the pinned commit, verify its hash, package it
- [x] 2.5 AMO source archive: add the engine's reproducible build recipe; `REVIEWERS.md`: the engine, the model download, why no code is fetched

## 3. The setting

- [x] 3.1 `translationHost: "none" | "local"` in `storage.local`, absent = `"none"`; load/save with tests; never synced
- [x] 3.2 Settings row « Traduction étendue » with its cost stated before ticking; states: off, downloading (progress + cancel), ready, failed (retry), interrupted (resume), removed (download again); localized messages, no raw errors; tests per state
- [x] 3.3 Hidden on Firefox for Android (`runtime.getPlatformInfo().os === "android"`) and on Safari; tests
- [x] 3.4 `createTranslatorPort()` answers from the stored host and the model's readiness instead of the build constant; tests

## 4. Download and storage

- [x] 4.1 Download in the engine's host (offscreen worker / event-page worker); stream, `DecompressionStream("gzip")`, sha256 against the manifest; a mismatch is discarded; tests with fixtures including a 200 HTML body
- [x] 4.2 Progress, completion, failure and cancel over the engine channel; settings view keeps the host awake while open; interrupted download resumes only unverified files, never on its own; tests
- [x] 4.3 `lingua-model` IndexedDB database: decompressed bytes keyed by sha256 + completed-manifest record; `navigator.storage.persist()`; tests
- [x] 4.4 Engine loads its model from `lingua-model` instead of files beside the bundle; development override kept for side-loading; tests
- [x] 4.5 Turning the setting off terminates the worker (and the offscreen document) and deletes `lingua-model`; tests
- [x] 4.6 Setting on with an empty or incomplete database → "removed" state, nothing fetched; tests

## 5. Engine lifetime

- [x] 5.1 Host records the last translation asked; keep-warm pings do not update it; ten idle minutes → worker terminated (offscreen document closed on Chromium); tests with a fake clock
- [x] 5.2 No eager load when the setting turns on or the download completes; test

## 6. Surfaces

- [x] 6.1 Single word: engine asked only when the pack has no gloss and the word is not `ProperNounOutOfLexicon`, with its sentence and the word marked; tests (`disambiguation`, a glossed word, a proper noun)
- [x] 6.2 No model ready (off, downloading, failed, interrupted, removed): cards answer from the pack with no "translation on its way" line; tests per state
- [x] 6.3 Update `TRANSLATION.md` for the release path

## 7. Disclosure and release

- [x] 7.1 Publish the Lingua annex (fr + en) on the site before the first release that can download
  - Published 2026-09-26 by `site-deploy` (fr + en, « Traduction étendue » / « Extended translation »).
- [ ] 7.2 Update the Chrome Web Store and AMO listings (and `STORE-LISTING.md`)
  - `STORE-LISTING.md` updated (description fr + en, `offscreen` justification, remote code, data usage); the two dashboards are pasted by hand.
- [ ] 7.3 Update from the published version on Chromium and Firefox: extension stays enabled with the `offscreen` permission added; setting off; no request made
- [x] 7.4 Manual pass Chrome + Firefox desktop: tick, download, cancel, retry after a cut network, translate a phrase and an unknown single word, idle release after ten minutes, untick deletes the model, offline translation after download
  - Passed 2026-09-26: headless Chrome 153 + Firefox 156 against a throttled local host (cancel, network cut + retry, phrase and unknown word through the card, offline, 10-minute idle release, background-only download, untick), then the product owner on a real Chrome and a real Firefox against `models.cymbra.app`.
- [x] 7.5 Manual pass Firefox for Android: no setting, no download, cards unchanged
  - Passed 2026-09-26 on a Galaxy Tab S6 Lite (product owner): no setting, no download. Superseded since by `add-lingua-translation-android` (#553), which offers the setting there.

## 8. Gates

- [x] 8.1 `openspec validate add-lingua-translation-delivery --strict`
- [x] 8.2 Extension lint, typecheck, tests and `check:variants` green; coverage ≥ 80 %
