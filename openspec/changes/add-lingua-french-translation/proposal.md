# add-lingua-french-translation — fr-en 2.0 pinned, French's routes, and their marks measured

## Why

Change 50 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en for English speakers, fr-es for Spanish speakers). « Traduction
étendue » translates a selection through the route the model catalogue gives the reader's pair
(change 8), and marks the selection only for a pair whose marks were measured and reached the
first tier (change 26). The catalogue holds three models — en-fr 2.0, es-en 2.0, en-es 2.1 — and
four routes; none starts from French.

The programme's architecture says what French needs: « Mozilla publishes no fr↔es model: en-es
2.1 is pinned in stage 2, fr-en 2.0 in stage 3, fr-es pivots through English. A reader needs at
most two models (≈ 322 MiB, as es-fr) ». Mozilla's registry holds one fr-en entry, `base-memory`,
`releaseStatus: "Release"`, whose model's decompressed sha256 is the one Firefox's Remote Settings
publish for fr→en 2.0 (measured 2026-10-09; Remote Settings lists 1.0 and 2.0, nothing later). The
study measured fr-en's marks at 94 / 96 and fr-es's at 90 / 91, one judge, ± 6 % — indicative
until each pair's committed measurement, which needs the routes and a corpus of French selections
that neither exists yet.

Nothing is offered to a reader here: a route is needed only by a reader whose pairs include it,
and no pair studying French ships before change 52 (`enable-lingua-french`).

## What Changes

- **The catalogue pins `fr-en/base-memory/2.0`**: Mozilla's three files
  (`model.fren.intgemm.alphas.bin.gz`, `lex.50.50.fren.s2t.bin.gz`, `vocab.fren.spm.gz`, run
  `retrain_hr_EFgIftH_RrCyzl5gjemVNg`), each with its served size, decompressed size, decompressed
  sha256, its path in the registry and the sha256 of the gzip file it serves, at content-addressed
  paths, with its mirror release `lingua-model-fr-en-base-memory-2.0`, MPL-2.0 — as en-es was
  pinned. Measured: 26,234,715 B to download, 37,200,311 B on the device; every digest equal to the
  registry's and Remote Settings'; the vocabulary the same bytes as en-fr's once decompressed.
- **Two routes**: `"fr-en": ["fr-en/base-memory/2.0"]`, direct, and
  `"fr-es": ["fr-en/base-memory/2.0", "en-es/base-memory/2.1"]`, through English (51,608,069 B,
  « 51,6 Mo »). Inert until change 52 lists the pairs in `packs.json`. With them, every native
  language's pairs need two models together: fr {en-fr, es-en}, en {es-en, fr-en}, es {en-es,
  fr-en}.
- **A French corpus for the marks**: `tool/marks/pud.mjs` pins UD French-PUD at the commit and
  sha256 change 43 pins (its D9 hands this pin over), and the selection rule runs over English,
  Spanish and French at once — the same 100 sentences: measured, French gives a word on every one,
  and English's and Spanish's 200 selections are byte for byte the committed ones.
- **The soak, by hand** (M25's recommendation, change 9): fr-en and fr-es over the 100 French
  selections, isolated and in one instance — prototype: 100 translated, no trap, no timeout, on
  both routes; recorded in `TRANSLATION.md`.
- **fr-en and fr-es measured**: `results-fr-en.jsonl`, `results-fr-es.jsonl`,
  `judged-fr-en.tsv`, `judged-fr-es.tsv`, judged in English and in Spanish by the criteria of
  change 26, French examples added before the run. Prototype: fr-en 95 / 96 shown marks correct,
  4 % withheld; fr-es 90 / 91, 9 % withheld — both on the first tier. A pair on the tier joins
  `MARKED_PAIRS`, inert until change 52; one short of it stays out, and the figures go to the owner
  under M15. The gloss experiment fills in only from a pair's committed glosses (changes 48, 49),
  never from the empty `gloss.tsv` change 43 commits.
- **The documents that name the models and the measured pairs**: `TRANSLATION.md`,
  `tool/marks/README.md`, `REVIEWERS.md`, the harness's and the soak's usage comments.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`:
  - ADDED *French is translated into English directly, and into Spanish through English*: the
    fr-en model pinned, fr-en's route direct, fr-es's through English, two models at most for every
    native language's pairs together;
  - MODIFIED *Translation models are listed in one catalogue, with a route per pair* — held by
    change 25 (`add-lingua-translation-matrix-models`, merged, open): its scenario *Every reader
    today* names the four models and six routes, every scenario kept, one added (*A route of a
    pair studying French*). Archived after change 25;
  - MODIFIED *A pair's marks are measured before they are shown* — held by change 26
    (`measure-lingua-translation-matrix-marks`, merged, open): every studied language's selections
    are taken from the same sentences by one rule, and adding a language moves no other's; every
    scenario kept, three added. Archived after change 26.

  Change 9's *The engine holds at most two models* and *A route can be soaked by hand*
  (`harden-lingua-translation-engine`, merged, open) are relied on and run, not modified; this
  change archives after it.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`model-manifest.json`,
  `tool/marks/`, `tool/measure_marks.mjs`, `src/translate/markup.ts` `MARKED_PAIRS`, the tests that
  hold them, `TRANSLATION.md`, `REVIEWERS.md`). The site reads the catalogue's routes for its
  shipped pairs only (`apps/site/src/lib/lingua-pairs.ts`), so its pages do not move; `site-check`
  runs on the catalogue and passes unchanged. ID, Music, Live and the back office are untouched.
- **Nothing a reader sees moves**: their pairs, routes, downloads and marks are en-fr's and
  es-fr's. No Rust, no table, no pack, no golden: `fr-en.golden`, S0, the es-fr, es-en and en-es
  goldens and the extension's snapshots are unchanged; en-fr's, es-fr's, es-en's and en-es's
  results and judgments are not rewritten.
- **The owner deploys the model host after the merge** (M18): until `lingua-model-deploy` has run,
  the check before a submission (`lingua-extension-release`) and before an App Store delivery
  (`lingua-apple-release`) refuses. On 2026-10-09 the host still answers 404 for en-es's files
  (change 25's task 4.1 has not run), so it refuses already; one dispatch serves both models.
- **Order.** After changes 9, 25 and 26 (archived after them). Independent of changes 40–49 in
  code: the engine's measurement needs no French analysis and no pack, so it can merge before 48
  and 49; their glosses only fill the experiment's columns, then or later. Before change 52, which
  ships fr-en and fr-es and offers translation as M15 settles it, and change 53, which writes the
  sizes into the listings (26.2 MB direct, 51.6 MB through English) — the privacy annex's « about
  26 MB direct, about 52 MB through English » (change 31) stays true.
- **Not here.** Shipping the pairs or offering translation to their readers (52, M15); the listings
  and the site's sentences (53); the copy of the setting (14, 15); a selection inside an elided word
  (51).
