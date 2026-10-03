## 1. The catalogue

- [x] 1.1 `model-manifest.json` becomes the catalogue (design D1): the en-fr model under its id, `unpacked` for each file, and `routes.en`.
- [x] 1.2 `src/translate/host/model-manifest.ts`: `parseCatalogue` (design D2), `routeOf`, `loadBundledCatalogue`, and the route's sizes. Specs:
  - the committed catalogue parses, and its English route is the en-fr model;
  - each refusal: an unknown model, a wrong first language, a broken chain, a route that does not end in French, a file without `unpacked`, a bad sha256, a `base` that is not http(s).

## 2. The runtime

- [x] 2.1 The model worker, the engine worker and the background take the English route's model (design D3); the engine's languages come from the model. Specs:
  - the download and the database take the route's model;
  - a completeness record written before the update stays complete.
- [x] 2.2 `ModelStatus.cost` from the catalogue, and the setting's sentences from it (design D4). Specs:
  - the status carries 25,752,472 and 36,749,127 bytes;
  - the setting says 25,8 Mo and 36,7 Mo;
  - without a cost, both sentences have no size.

## 3. The build and the tools

- [x] 3.1 `tool/model-catalogue.mjs`, read by `build.mjs` (the bundled catalogue: runtime fields and routes, `LINGUA_MODEL_BASE_URL` kept) and `check_variants.mjs` (every model and route compared). Specs for the module.
- [x] 3.2 `assemble_model_site.mjs` and `check_model_host.mjs` over every model. Run locally:
  - `node tool/check_model_host.mjs` passes against `models.cymbra.app`;
  - `node tool/assemble_model_site.mjs <tmp>` assembles en-fr with both digests and the sizes.
- [x] 3.3 `tool/mirror_models.mjs` (design D5), called by the mirror job of `lingua-model-deploy.yml`. Specs for the tag and for what is created. Run locally: `--dry-run` finds the en-fr release and creates nothing.
- [x] 3.4 `TRANSLATION.md` and `REVIEWERS.md` describe the catalogue.

## 4. Gates

- [x] 4.1 In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.
- [x] 4.2 `openspec validate generalise-lingua-translation-catalogue --strict` passes. In `docs/lingua/spanish-programme.md`, change 16 is marked done.
