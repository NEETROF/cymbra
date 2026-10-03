# generalise-lingua-translation-model-state — the models follow the reader's languages

## Why

`generalise-lingua-translation-catalogue` lists the translation models and gives each studied
language its route to French, but the device still handles one model:
- **The database** keeps one completeness record. Turning the setting off deletes everything.
- **The setting's state** says whether « the model » is ready.
- **The download** fetches the default language's model.
- **The engine** loads that model and translates whatever it is handed, without knowing the language
  of the sentence.

A reader of English and Spanish needs en→fr for English pages, and es→en then en→fr for Spanish
ones. en→fr is shared, and must stay while either language needs it. A sentence must reach the route
of its document's language.

This is change 17 of `docs/lingua/spanish-programme.md`, in R3, a silent English release. The
catalogue has a route for English only, so every reader keeps exactly what they have today: one
model, downloaded when they tick the setting, used for English pages.

## What Changes

- **The device keeps the models its reader's languages need.** The needed models are the union of
  the routes of the accepted languages; a language without a route needs nothing.
  - Ticking the setting downloads them all, with one progress bar over everything missing.
  - A model the accepted languages no longer need is deleted. A shared model stays while any route
    needs it.
- **Completeness is recorded per model.** The single record kept today is read as its model's
  record, so a stored English model stays complete.
- **The setting's state covers the needed models**, and names the ones complete. It gains `missing`:
  a language the reader added needs a model that was never downloaded. The setting says what it costs
  and offers « Télécharger »: nothing is fetched unasked. `removed` keeps meaning that the browser
  removed a model that was there.
- **A translation names its document's language.** The selection card asks in the language the
  session reads the document in, and the background answers only when every model of that language's
  route is complete. The engine loads that route:
  - a route of one model, as English's, translates;
  - a route through English answers as without a model, until
    `add-lingua-spanish-translation-pivot`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-translation`: ADDED:
  - *The models follow the reader's languages*;
  - *A translation is asked in its document's language*.

  The requirements of `add-lingua-translation-delivery`, still open, keep their names, and this
  change adds beside them.

## Impact

- **Products.** Cymbra Lingua's browser extension, and the Safari app that bundles it:
  - `src/translate/host/*`: the database, the download and its worker, the controller, the engine
    worker, the channel, the offscreen bridge and the relay;
  - `src/translate/*`: the setting's state, the messages, the translation request and its wire;
  - `src/reading/translation-setting.ts` and the selection card;
  - `src/background.ts`.

  It consumes the catalogue and the accepted languages (`acceptedLanguages`). No engine build, pack,
  server, proto or model host change. ID, Music, Live, the back office and the site are not
  affected.
- **Release.** R3, silent: one route, one model, as today.
