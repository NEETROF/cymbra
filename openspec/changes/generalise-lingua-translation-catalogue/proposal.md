# generalise-lingua-translation-catalogue — one catalogue of translation models, a route per language

## Why

« Traduction étendue » downloads one model, English to French, described by `model-manifest.json`:
one version, three files, one licence, one mirror release. Everything that reads it assumes one model
that translates English:
- the download and the database's completeness record;
- the engine (`TranslationModel("en", "fr", …)`);
- the setting's cost, written by hand (« 25,8 Mo », « 37 Mo »);
- the build's bundled copy and the variant check;
- the tool that assembles the model host, the step that keeps a mirror release of the model, and the
  check run before every store submission.

Spanish is translated through English: Mozilla has no direct es→fr model, so the route is es→en, then
en→fr, and en→fr is shared with English readers (`docs/lingua/spanish-programme.md`, Architecture).
The manifest therefore becomes a catalogue first: the models the package may download, and for each
studied language the route of models that translates it into French. The Spanish model is added
later, by `add-lingua-spanish-translation-pivot`.

This is change 16 of the programme, in R3, a silent English release. The catalogue holds en-fr alone
and the English route is en-fr: every reader downloads, stores and translates exactly as today.

## What Changes

- **`model-manifest.json` becomes a catalogue**, in the same file and place, so the reviewers' notes
  and the release checks keep pointing at it:
  - `models`: for each model, its id (`en-fr/base-memory/2.0`, the version stored today), its `from`
    and `to`, its licence and its mirror release, and its three files. Each file has its address, its
    size as served, its decompressed size (`unpacked`, new), the sha256 of its decompressed bytes, and
    Mozilla's source;
  - `routes`: for each studied language, the models that translate it into French, in order
    (`en: ["en-fr/base-memory/2.0"]`);
  - `base` and `sourceBase`, as today.

  The parser refuses a route that names an unknown model, does not start from its language, breaks
  the chain or does not end in French.
- **The runtime takes the English route from it.** The download, the database's completeness and the
  engine take the route's model. The engine's languages come from the model, not from constants.
  `generalise-lingua-translation-model-state` widens this to the routes of the reader's languages.
- **The setting's cost comes from the catalogue.** The download (« 25,8 Mo ») and the room the model
  takes on the device are sums over the route's files. The room was « 37 Mo », written by hand, and
  now reads « 36,7 Mo ».
- **The build, the variant check and the model host's tools cover every model of the catalogue:**
  - the bundled catalogue keeps only what the runtime needs;
  - the variant check compares every model and route with the committed catalogue;
  - the host's assembly takes every model, with its notice;
  - the host check checks every file of every model;
  - the deploy workflow's mirror step keeps one release per model, through a script it calls
    (`tool/mirror_models.mjs`) instead of shell that reads one model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: ADDED:
  - *Translation models are listed in one catalogue, with a route per studied language*;
  - *The setting's cost comes from the catalogue*.

  The open changes on this capability (`add-lingua-translation-delivery`, `-android`, `-safari`) hold
  requirements under other names.

## Impact

- **Products.** Cymbra Lingua's browser extension, the Safari app that bundles its package, and the
  model host's tooling:
  - `apps/lingua-extension/model-manifest.json` (its shape) and `src/translate/host/*` (manifest,
    database, download, controller, engine worker);
  - `src/reading/translation-setting.ts` (the cost);
  - `build.mjs`, `tool/check_variants.mjs`, `tool/assemble_model_site.mjs`,
    `tool/check_model_host.mjs`, and a new `tool/mirror_models.mjs`;
  - `.github/workflows/lingua-model-deploy.yml`, whose mirror step calls the script. The release
    workflows still run `check_model_host.mjs`, unchanged;
  - `TRANSLATION.md` and `REVIEWERS.md`.

  It consumes the model host as deployed: `models.cymbra.app` serves en-fr 2.0 at the same paths, so
  there is nothing to redeploy. No engine, pack, server or proto change. ID, Music, Live, the back
  office and the site are not affected.
- **Release.** R3, silent: the same model, files and paths, and a stored model stays complete. The
  setting's room on the device reads « 36,7 Mo » instead of « 37 Mo ».
