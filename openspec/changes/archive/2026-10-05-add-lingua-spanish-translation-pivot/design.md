# Design

## Context

The R3 changes left Spanish translation one model and one engine path away:
- `model-manifest.json` is a catalogue: models, and a route per studied language
  (`generalise-lingua-translation-catalogue`). It names en-fr alone, with the route `en`.
- The device keeps the models its reader's routes need, and asks a translation in its document's
  language (`generalise-lingua-translation-model-state`).
- The engine worker builds each model of a route once. It refuses a route of more than one model
  with `NO_PIVOT`, and `translate` uses the route's first model.
- The bundled engine is the Firefox Translations build pinned by `engine-pin.json`. It exposes
  `BlockingService.translateViaPivoting(first, second, messages, options)`: one request through two
  models, alignments carried through the pivot.
- The selection's mark rides on the engine's alignment (`translate/markup.ts`):
  - the sentence goes out with the selection tagged `<b>`;
  - `relay.ts` sends it with the selection alone, which checks where the tag landed
    (`reconcile.ts`).

## Goals / Non-Goals

**Goals:**
- Spanish translated on every platform that offers extended translation, through English.
- No mark on a Spanish sentence until its marks are measured.
- The setting, the listings and the site state Spanish's cost like for like.

**Non-Goals:**
- A direct Spanish→French model, which Mozilla does not publish.
- Measuring the marks: that is change 27, `release-lingua-spanish-translation`.
- Remote translation, which is parked.

## Decisions

### D1 — The es-en model, pinned

Mozilla's registry (`db/models.json`, generated 2026-10-05) lists `es-en`:
- architecture base-memory, release status Release, MPL-2.0;
- run `retrain_hr_HbNjJ60BTwmVTbhfFxuduA`;
- flores200 BLEU 26.85, against 48.85 for en-fr.

Its three files are pinned as en-fr's are. The decompressed model's sha256 matches the registry's
`uncompressedHash`.

| File | Served (gzip) | Decompressed | sha256 (decompressed) |
|---|---|---|---|
| model | 23,288,494 | 31,561,787 | `4aed7734…b033` |
| lex | 2,543,246 | 4,636,248 | `e2610211…2daa` |
| vocab | 409,312 | 816,054 | `5ae254fa…58ad` |

The download is 26.2 MB, against en-fr's 25.8 MB. The catalogue id is `es-en/base-memory/2.0`:
- its mirror release is `lingua-model-es-en-base-memory-2.0`;
- its paths follow en-fr's, content-addressed by the decompressed sha256;
- Spanish's route is `["es-en/base-memory/2.0", "en-fr/base-memory/2.0"]`.

`assemble_model_site.mjs`, `mirror_models.mjs` and `check_model_host.mjs` read the catalogue, so
they cover it with no change. `check_variants` holds the bundled catalogue to the committed one.

### D2 — Pivoting in the engine

`load` builds every model of the route once, as today; a model two routes share stays one instance.
`translate` takes the route:
- one model: `service.translate`, as today;
- two: `service.translateViaPivoting(first, second, messages, options)`, with the same options (html
  and alignment);
- more: refused, since no catalogue route chains three models and the engine has no call for it.

`NO_PIVOT` goes.

### D3 — No mark where the marks are not measured

`MARKED_LANGUAGES`, in `translate/markup.ts`, names the languages whose selection is marked:
`["en"]`.

For a language outside the list, `relay.ts` sends the sentence untagged (`escapeText`). It makes one
request instead of two: the second request only checks a mark. It answers the sentence with no
mark.

The card already shows a translation without a mark when the engine drops the tag. So the reader
sees the French sentence, nothing bold in it.

Change 27 measures the pivot's marks against D2's threshold, fixed before measuring. It adds `es` to
the list if they pass.

### D4 — What the setting says

The download is already the sum of the needed models' files, read from the catalogue: 52.0 MB for a
reader of Spanish, 25.8 MB for a reader of English alone.

The memory sentence was written for one model: « environ 200 Mo de mémoire pendant la traduction ».
When a needed route has two models, it says « environ 340 Mo ». That is the programme study's 322 MiB
for the pivot. Dogfooding checks it.

### D5 — The copy

The listings and the site say Spanish is translated through English, with its download.
- **Extension listing:**
  - FR « … traduite dans sa phrase… L'activer télécharge une fois le modèle de traduction (25,8 Mo
    pour l'anglais, 52,0 Mo avec l'espagnol, traduit en passant par l'anglais) … »;
  - EN « … (25.8 MB for English, 52.0 MB with Spanish, translated through English) … ».
- **App Store listing:** « Pour l'espagnol, elle arrivera plus tard. » becomes « L'espagnol est
  traduit en passant par l'anglais : 52,0 Mo à télécharger avec lui. »
- **Site:**
  - FR « La traduction étendue, facultative, traduit l'anglais et l'espagnol (en passant par
    l'anglais). »;
  - EN "Extended translation, optional, serves English and Spanish (through English)."

### D6 — Order of deployment (owner)

The package must not ship before its model is served.
1. After this change merges, the owner dispatches `lingua-model-deploy`. It assembles the host with
   the es-en files, creates the mirror release, deploys and runs `check_model_host`.
2. `lingua-extension-release` runs the check again before any store submission. A release
   dispatched first fails there, naming the es-en files.

## Risks / Trade-offs

- **Memory.** Two models load where one did: about 340 MB instead of 200 MB, by the study's figure.
  - On Chrome, Firefox and Safari on macOS it fits.
  - On Safari for iOS the extension's memory is bounded. Dogfooding on the iPhone and the iPad says
    whether it holds.
  - If it does not, the remedy is a per-platform list like D3's, a change of its own.
- **Quality.** The French sentence goes through English: es-en scores 26.85 BLEU against en-fr's
  48.85. Tense, gender and « tu/usted » can blur. The card says it is a machine translation already.
- **Latency.** Two models translate where one did, about twice as long. It stays inside the
  pack-first card: a slow engine never costs the reader the pack's answer.
