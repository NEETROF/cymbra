# generalise-lingua-translation-routes-by-pair — a translation route per pair

## Why

Change 8 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the fifth of stage 1 and a silent release. The programme's architecture says it in one line:
translation routes are keyed by pair.

Today the catalogue keys its routes by studied language — `en` is the en-fr model, `es` is es-en
then en-fr — and the parser refuses a route that does not end in French. Everything inward of the
reading page is keyed the same way: the engine's contract, the offscreen document's protocol, the
worker's loaded routes, the model state the device records, the list of languages whose marks are
measured, and the marks harness. The reader's native language reaches none of it: the background
uses it only to filter the studied languages (change 4), then asks for "English".

A reader of English who studies Spanish needs es-en, the one model, and not es-fr's pivot. A reader
of Spanish who studies English needs en-es. Neither can be a catalogue entry while the key is the
studied language, and the French target is written into the parser. Changes 21, 22 and 25 add those
pairs and their models; this change makes the entry possible, with no model added and no route
moved.

This is a silent release. For a reader of French, nothing moves:
- en-fr and es-fr translate through the routes they have today, with the same marks;
- the French interface is byte for byte the same;
- the wire between the reading page and the background still asks in the document's language;
- a model state written by an earlier release is read as it was meant.

## What Changes

- **The catalogue keys its routes by pair.** `routes` is `{"en-fr": [...], "es-fr": [...]}`. The
  parser reads the studied and the native language from the key, checks that a route starts from
  the one and ends in the other, and no longer knows French. The bundled catalogue still equals
  the committed one, so `check_variants` is unchanged.
- **The pair is formed in the background**, from the document's language the page asked in and the
  reader's native language, read from their stored profile as the model controller already reads
  it. From the background inward — the engine's contract, the offscreen document, the engine
  worker and its channel — everything carries the pair. The three hosts keep their shapes; only
  the key they carry changes.
- **The device records which pairs are ready**, not which languages. A state written before,
  which named languages, is read as the pairs of the default native language: every pair shipped
  so far is glossed in French. A reading page still gates on the document's language, through the
  ready pairs that study it.
- **Marks are measured per pair.** `MARKED_PAIRS` names en-fr and es-fr, whose marks were measured;
  a pair outside it is translated without a mark, as a language outside the list is today. The
  harness measures a pair, and its results and judgments are filed by pair; the corpus stays per
  studied language, since a selection is of the text it was made in.
- **The model controller counts pairs**: the pairs the reader's accepted languages make with their
  native language, their routes' models, which of them pivot, and which pairs are translatable.
- **Docs follow**: `TRANSLATION.md`, the harness README, and two stale lines (`README.md` still
  lists one pack, `env.d.ts` still says no shipped build hosts the engine).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`:
  - RENAMED and MODIFIED *Translation models are listed in one catalogue, with a route per studied
    language* → *…, with a route per pair*: a route starts from the pair's studied language and
    ends in its native language.
  - RENAMED and MODIFIED *The models follow the reader's languages* → *The models follow the
    reader's pairs*: what the device keeps is what the reader's pairs' routes need; the native
    language changing (the choice is change 20) changes the pairs.
  - RENAMED and MODIFIED *A language's marks are measured before they are shown* → *A pair's marks
    are measured before they are shown*: es-en is not marked on es-fr's measurement.
  - ADDED *A translation goes through the reader's pair*: the background forms the pair; the page
    asks in the document's language; a pair without a route is unavailable without the engine
    starting.
  - MODIFIED *A translation is asked in its document's language*, *The answer is the reader's
    sentence with their selection marked* and *Spanish is translated through English*: the
    sentences that said "that language's route", "English, and Spanish" and "Spanish's route" now
    say the reader's pair, en-fr and es-fr; every scenario kept.

No requirement this change modifies is held by an open change (`add-lingua-translation-delivery`,
`-android` and `-safari` hold the setting's, the download's and the loading's requirements, none of
these four).

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`model-manifest.json`,
  `src/translate/**`, `src/background.ts`, `tool/measure_marks.mjs`, `tool/marks/`, `TRANSLATION.md`,
  `README.md`, `env.d.ts`). ID, Music, Live, the back office and the site are untouched.
- **No model, host or byte moves.** The catalogue lists the same two models; `models.cymbra.app`
  and `lingua-model-deploy` are untouched; the engine pin is untouched; the two packs and their
  goldens are untouched; the French copy is untouched.
- **Compatibility.** The wire message types (`lingua-translate`, `lingua-translate-warm`) keep
  their fields. A stored `ready` or `missing` that names `languages` is read as pairs once and
  rewritten as pairs on the next reconciliation.
- **Not here.** The engine's respawn after a trap and the eviction of models no pair needs
  (change 9); es-en, en-es and their models (changes 21, 22, 25); the marks of the new pairs
  (change 26); the native language's choice in Réglages (change 20).
