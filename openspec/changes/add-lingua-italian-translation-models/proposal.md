# add-lingua-italian-translation-models — it-en 2.0 and en-it 2.1 pinned, and a route for each Italian pair

## Why

Change 2 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 0. « Traduction
étendue » translates a selection with Mozilla's models through the route the catalogue gives each
pair (matrix change 8); the catalogue holds four models — en-fr, es-en, en-es, fr-en — and six
routes, one per pair of the matrix. Italian adds six pairs and none of them is routed:

- **it-en** (an English-native reader of Italian) and **en-it** (an Italian-native reader of
  English) each need one of Mozilla's two Italian models, which nothing pins. Mozilla's Remote
  Settings publish it-en at version 2.0 and en-it at 2.1 (read on 2026-10-21); Mozilla publishes
  no fr↔it and no es↔it model.
- **it-fr, it-es, fr-it, es-it** go through English, as es-fr and fr-es do: two models each, every
  one of them already pinned or pinned here.

Pinning now, before any Italian pack exists, lets the owner redeploy the model host once for the
whole programme (the release gate refuses a package whose catalogue names a model the host does
not serve), lets the marks harness measure each Italian pair through the catalogue's route when its
row comes (rows 17, 22, 27, 36, 47, 51), and lets the soak try the en-it model over the three
corpora that exist (M25: a manual tool) before any Italian speaker is offered it. The routes are
inert until their pair ships: a reader's pairs are the shipped pairs of their native language.

## What Changes

- **The catalogue pins two models** (design D1): `it-en/base-memory/2.0` and
  `en-it/base-memory/2.1`, Mozilla's three files each (`model.<pair>.intgemm.alphas.bin.gz`,
  `lex.50.50.<pair>.s2t.bin.gz`, `vocab.<pair>.spm.gz`), each with its served size, decompressed size
  and sha256, the source path in Mozilla's registry and the gzip's sha256, served from the model
  host at content-addressed paths, with a mirror release each — as en-es and fr-en were pinned.
  The values are read from the files themselves and checked against Remote Settings' hashes for
  2.0 and 2.1.
- **Six routes** (D2): `it-en` and `en-it` one model each; `it-fr` it-en then en-fr; `it-es` it-en
  then en-es; `fr-it` fr-en then en-it; `es-it` es-en then en-it. Two models at most per route, as
  the engine's hardening allows (matrix change 9).
- **One vocabulary for the two models** (D3): it-en 2.0's and en-it 2.1's vocabularies decompress
  to the same bytes (812,724 B, one sha256), stored once on a device, as es-en's and en-es's are.
- **Three soaks by hand** (D4): the en-it model through `en-it`, `fr-it` and `es-it` over the
  English, French and Spanish corpora, each selection isolated, the figures and the ids that trap
  recorded in `TRANSLATION.md`. The it-en model is soaked by row 17, which brings the Italian
  corpus; it does not exist yet.
- **The documents that name the models** — `TRANSLATION.md`, `tool/marks/README.md`,
  `REVIEWERS.md`, the soak tool's usage — and **the tests**: `test/model-manifest.spec.ts` holds the
  new catalogue exactly and shows that no reader of today's shipped pairs needs an Italian route or
  model.
- **The owner redeploys the model host after the merge** (D6): until then, every release of the
  extension and of the Apple app is refused by the gate that checks the host serves every pinned
  model. The implementation merges when the owner can dispatch `lingua-model-deploy` right after.
- Not in this change: the marks of any Italian pair (`MARKED_PAIRS` keeps its five), the Italian
  corpus, any pack, any Italian in the interface.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: ADDED *Italian is translated to and from English directly, and to and
  from French and Spanish through English*; MODIFIED *Translation models are listed in one
  catalogue, with a route per pair* — its scenario *Every reader today* names the six models and
  twelve routes, every scenario kept, one added (a route of an Italian pair). The requirement is
  held by two open changes, `add-lingua-translation-matrix-models` (25) and
  `add-lingua-french-translation` (50): this change archives after both, and its text is 50's with
  the Italian models and routes added.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `apps/lingua-extension/model-manifest.json`: two models, six routes; `test/model-manifest.spec.ts`;
  `TRANSLATION.md`, `tool/marks/README.md`, `REVIEWERS.md`, `tool/soak_engine.mjs`'s usage.
- The model host (`models.cymbra.app`): two models more, ≈ 69 MB served, deployed by the owner;
  two mirror releases the deploy workflow creates.
- No engine change, no core, no server, no pack, no store listing. A reader of today downloads,
  keeps and loads exactly what they do now.
