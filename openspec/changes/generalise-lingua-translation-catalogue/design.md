# Design — generalise-lingua-translation-catalogue

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `model-manifest.json` | One model: `version` (`en-fr/base-memory/2.0`), `from`, `to`, `licence`, `base`, three `files` (path, size as served, sha256 of the decompressed bytes, Mozilla's source), `sourceBase`, `mirror`. |
| `build.mjs` | Bundles the manifest without its deployment fields, at `LINGUA_MODEL_BASE_URL` in a development build. |
| `src/translate/host/model-manifest.ts` | `parseManifest`, `fileUrl`, `totalSize`, `loadBundledManifest`. |
| `model-db.ts`, `model-download.ts`, `model-worker.ts` | Download the manifest's files, keep them by sha256, and record `{ version, files }` as complete. |
| `model-controller.ts`, `background.ts` | The setting's state machine; the download's total is `totalSize(manifest)`. |
| `engine-worker.ts` | Reads the three files by sha256 and builds `TranslationModel("en", "fr", …)`. |
| `src/reading/translation-setting.ts` | The cost written by hand: « Télécharge 25,8 Mo une fois », « Pas assez de place … (37 Mo) ». |
| `tool/check_variants.mjs` | Compares each package's bundled manifest with the committed one. |
| `tool/assemble_model_site.mjs`, `tool/check_model_host.mjs` | Assemble the host from Mozilla, the mirror or the host itself, both digests checked, then check it from outside. |
| `lingua-model-deploy.yml` | Its mirror job reads `.mirror`, `.version` and `.files.<role>.path` in shell to keep one release. |

## Goals / Non-Goals

**Goals:**
- One catalogue: the models the package may download, and a route per studied language to French.
- Every consumer reads the catalogue. The runtime takes the English route, and the tools take every
  model.
- English unchanged: the same model, files and paths; a model stored before the update stays complete.

**Non-Goals:**
- Several models on one device: completeness per model, deletion with reference counting, and the
  routes of the reader's languages are `generalise-lingua-translation-model-state`.
- The Spanish model and pivoting (`add-lingua-spanish-translation-pivot`).
- The memory figure (« environ 200 Mo »). It depends on the engine and on the route's length, is
  measured per platform (`release-lingua-spanish-translation`), and stays written by hand.

## Decisions

### D1 — The catalogue, in the same file

```json
{
  "base": "https://models.cymbra.app/",
  "sourceBase": "https://storage.googleapis.com/…/",
  "models": {
    "en-fr/base-memory/2.0": {
      "from": "en", "to": "fr", "licence": "MPL-2.0",
      "mirror": "https://github.com/NEETROF/cymbra/releases/download/lingua-model-en-fr-base-memory-2.0/",
      "files": {
        "model": { "path": "…", "size": 23045432, "unpacked": 31561787, "sha256": "…", "source": { "path": "…", "sha256": "…" } },
        "lex": { … }, "vocab": { … }
      }
    }
  },
  "routes": { "en": ["en-fr/base-memory/2.0"] }
}
```

- **The model's id is the version stored today.** The database's completeness record keeps
  `version: "en-fr/base-memory/2.0"`, and the files' paths already start with it. A model stored
  before the update is therefore still complete after it, and nothing is downloaded again.
- **`unpacked`** is the decompressed size, measured for each file: 31,561,787, 4,372,936 and 814,404
  bytes. The host's assembly checks it with the sha256, so a wrong figure cannot be committed
  silently.
- **The same file name** keeps `REVIEWERS.md`, `STORE-LISTING.md` and the release workflows pointing
  at the right place. Only the shape changes.

*Rejected — a new `model-catalogue.json`.* The name would be clearer, but it would move every
reference for no change in what the file is: the package's list of what it may download.

### D2 — What the parser refuses

`parseCatalogue` requires an `http(s)` `base`. Each model needs `from`, `to` and its three files, each
with a path, a size, an `unpacked` size and a sha256. Each route must:
- name known models;
- start from its language (`from` of the first model);
- chain (each `to` is the next `from`);
- end in French (`to` of the last model is `fr`).

A broken catalogue fetches nothing, as a broken manifest does today.

### D3 — The runtime takes the English route, a single model

`loadBundledCatalogue()` reads the package's copy. `routeOf(catalogue, language)` returns the route's
models as manifests: each model's id as `version`, the catalogue's `base`, its languages and its
files. These are the shape the database, the download and the engine already take. The runtime takes
the route of the default language. For English, that route holds one model, as this change keeps it:
- **The download worker** fetches the route's model.
- **The engine worker** builds `TranslationModel(model.from, model.to, …)`.
- **The controller** reports the route's sizes.

Several models, and the reader's languages, are the next change.

### D4 — The cost, from the catalogue

`ModelStatus` gains `cost: { download, stored }`, in bytes:
- `download` is the sum of the route's files as served;
- `stored` is the sum of their `unpacked` sizes.

The background computes it, because surfaces cannot all read the package's files: a content script
reads only web-accessible resources. The settings block writes « Télécharge 25,8 Mo une fois » and
« Pas assez de place sur cet appareil pour le modèle (36,7 Mo). » from it. A status without a cost
(a background that cannot read the catalogue) writes both sentences without a size.

### D5 — The tools read the catalogue through one module

`tool/model-catalogue.mjs` reads the committed catalogue and lists its models and their files. It is
used by:
- `build.mjs`, for the bundled copy: the runtime fields of every model, and the routes;
- `check_variants.mjs`, which compares every model and every route;
- `assemble_model_site.mjs`, which takes every model and writes a notice beside each one;
- `check_model_host.mjs`, which checks every file of every model;
- the new `mirror_models.mjs`.

`mirror_models.mjs <assembled-dir> [--dry-run]` keeps a mirror release for each model that names
one. It derives the tag from the mirror address, `gh release view`s it, and creates it with the
model's files and notice when it is missing. The workflow's mirror job calls it instead of its shell.
`--dry-run` says what it would create, so the script can be run against the real releases before a
dispatch.

## Risks / Trade-offs

- **The mirror step cannot run before a dispatch** → its logic lives in a script with unit specs, and
  the dry run against the real releases finds the en-fr release and creates nothing.
- **A route of several models before the runtime handles them** → none exists until
  `add-lingua-spanish-translation-pivot`, which comes after the next change. Until then, the parser
  accepts such a route and the runtime takes the English route alone.
