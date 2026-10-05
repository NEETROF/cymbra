# add-lingua-spanish-translation-pivot — Spanish translated through English

## Why

Extended translation serves English alone. Spanish reading ships (`enable-lingua-spanish`, change
28), and the generalised catalogue already gives each studied language a route of models to French
(`generalise-lingua-translation-catalogue`). Two things are missing:
- the catalogue has no Spanish route;
- the engine refuses any route of more than one model with `NO_PIVOT`.

Mozilla publishes no direct Spanish→French model, so Spanish goes through English. The
programme planned two changes: change 26 to ship the pivot, then change 27 to measure it and turn it
on platform by platform (decisions D2 and D3).

On 2026-10-05 the owner decided otherwise.
- Spanish translation is turned on everywhere at once. This replaces D3's per-platform release.
- Spanish sentences are translated without marking the selected word, until change 27 measures the
  marks. This applies D2's middle tier: the study's indicative sample scored about 12–13 correct
  marks out of 20 for the pivot, against about 17 for English.

This is change 26, R5 in `docs/lingua/spanish-programme.md`.

## What Changes

- **The es-en model in the catalogue.** `es-en/base-memory/2.0` comes from Mozilla's registry,
  run `retrain_hr_HbNjJ60BTwmVTbhfFxuduA`, released under MPL-2.0.
  - It is pinned like en-fr: each file's address, size as served, decompressed size and sha256.
  - The download is 26.2 MB.
  - Spanish's route is es-en then en-fr.
  - The model host's assembly, its mirror release and the pre-submission check cover it, because
    they read the catalogue.
- **The engine translates through a route of two models.**
  - Both models load once.
  - A Spanish sentence goes through both in one request (`translateViaPivoting`).
  - `NO_PIVOT` goes.
- **No mark where the marks are not measured.**
  - A language outside the list of marked languages (English today) is sent untagged, in one
    request, and the answer carries no mark.
  - Change 27 adds Spanish to that list if its marks pass D2.
- **The setting states what Spanish costs.**
  - For a reader of Spanish, the download is both models: 52.0 MB.
  - The memory figure is the pivot's.
  - A reader of English alone sees 25.8 MB, as today.
- **The listings and the site say Spanish is translated**, through English, with its download.
  Changes 29 and 30 deferred exactly this sentence to this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`:
  - « The answer is the reader's sentence with their selection marked » marks the selection only in
    a language whose marks were measured;
  - a requirement is added for Spanish's route through English.
- `lingua-browser-extension`, `site-lingua-page`: no requirement changes. The listings and the page
  already say per language what translation serves. Their text changes.

## Impact

- **Products:**
  - Cymbra Lingua: Spanish translation on every platform that offers extended translation;
  - the site's Lingua page;
  - the model host, `models.cymbra.app`, which gains the es-en files;
  - nothing is consumed from ID or the platform.
- **Code:**
  - `apps/lingua-extension/model-manifest.json`;
  - `src/translate/host/engine-worker.ts` and `engine.ts`;
  - `src/translate/host/relay.ts`, with the list of marked languages;
  - `src/reading/translation-setting.ts`;
  - their tests.
- **Copy:** the two `STORE-LISTING.md` files, and `apps/site/src/pages/lingua.astro` and
  `en/lingua.astro`.
- **Deployment (owner):** `lingua-model-deploy` is dispatched before the release.
  - It assembles the host with the es-en files.
  - It creates the mirror release `lingua-model-es-en-base-memory-2.0`.
  - It deploys, then runs `check_model_host`.
  - `lingua-extension-release` runs that check again before any store submission, so a package
    cannot ship ahead of its model.
- **Risk:** memory. Two models load where one did: the programme's study gave 322 MiB for the
  pivot. Dogfooding on the iPhone and the iPad says whether Safari on iOS holds it.
