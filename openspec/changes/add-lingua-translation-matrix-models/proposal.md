# add-lingua-translation-matrix-models — en-es 2.1 pinned, and a route for each new pair

## Why

Change 25 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2. « Traduction étendue » translates a selection with Mozilla's models through the route
the catalogue gives each pair (change 8). The catalogue holds two models today, en-fr and es-en,
and two routes, en-fr and es-fr (es-en then en-fr). The two new audiences need theirs:

- **es-en** — an English-native reader of Spanish — is translated by the es-en model alone. That
  model is pinned already, as es-fr's pivot (#709), but no route names the pair, so the harness of
  change 26 cannot measure it and change 34 could not offer it.
- **en-es** — a Spanish-native reader of English — needs Mozilla's en-es model, which nothing pins.
  The programme pins version 2.1: Mozilla's registry holds one en-es entry, `base-memory`,
  `releaseStatus: "Release"`, whose model's decompressed sha256 is the one Firefox's Remote
  Settings publish for en-es 2.1. Mozilla publishes no fr↔es model; nothing of this change
  concerns French as a studied language (stage 3).

The study measured en-es trapping the WebAssembly engine on some inputs (risk 3). Change 9 made a
trap cost one respawn and bounded the worker at two models, and gave the soak tool a scenario for
this moment: once en-es is pinned, the tool runs over the English corpus and names the selections
that trap, so that enabling Spanish speakers (change 35) is decided on measured inputs.

## What Changes

- **The catalogue pins `en-es/base-memory/2.1`**: Mozilla's three files (`model.enes.intgemm.alphas.bin.gz`,
  `lex.50.50.enes.s2t.bin.gz`, `vocab.enes.spm.gz`), each with its served size, decompressed size,
  decompressed sha256 and the source path and gzip sha256 in Mozilla's registry, served from the
  model host at content-addressed paths, with its mirror release
  `lingua-model-en-es-base-memory-2.1` — as es-en was pinned. The values are read from the files
  themselves, downloaded once from the registry, and checked against the registry's and Remote
  Settings' hashes for 2.1.
- **Two routes, one model each**: `"es-en": ["es-en/base-memory/2.0"]` and
  `"en-es": ["en-es/base-memory/2.1"]` — no pivot. They are inert until their pair ships: a
  reader's pairs are the shipped pairs of their native language (`packs.json`, changes 34 and 35),
  so no reader of today needs either route, and nothing is downloaded.
- **The en-es soak**, by hand (M25's recommendation, change 9): `tool/soak_engine.mjs --pair en-es
  --isolate` over the 100 English selections of the marks corpus, its figures and the ids that
  trap recorded in `TRANSLATION.md`.
- **The documents that name the models**: `TRANSLATION.md`, `tool/marks/README.md`'s notes on the
  bound, and `REVIEWERS.md` (the add-on's source archive), which names en-fr alone today.
- **The tests**: `test/model-manifest.spec.ts` holds the new catalogue exactly; the residency and
  controller specs' placeholder id `en-es/base-memory/2.0` becomes the pinned `2.1`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: ADDED *English and Spanish are translated into each other directly*;
  MODIFIED *Translation models are listed in one catalogue, with a route per pair* — its scenario
  *Every reader today* names the three models and four routes, every scenario kept, one added (a
  route of a pair not shipped). Held by no open change. Change 9's *A route can be soaked by hand*
  (open) is run, not modified.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`model-manifest.json`, the tests that
  hold it, `TRANSLATION.md`, `REVIEWERS.md`, `tool/marks/README.md`). Nothing a reader of today
  sees moves: their pairs, their routes, their downloads and their marks are en-fr's and es-fr's.
  ID, Music, Live, the back office and the site are untouched.
- **The owner deploys the model host right after the merge** (M18): until `lingua-model-deploy`
  has run, the check before a submission (`lingua-extension-release`) and before an App Store
  delivery (`lingua-apple-release`) refuses, since the host does not serve en-es yet.
- **Order.** After change 9 (merged as #765). Before change 26 (the routes it measures) and changes
  34 and 35 (which ship the pairs with their routes).
- **Not here.** The marks of es-en and en-es (26); the listings' sizes (36, 37); the copy of the
  setting in English and Spanish (14, 15); fr-en and fr-es (stage 3).
