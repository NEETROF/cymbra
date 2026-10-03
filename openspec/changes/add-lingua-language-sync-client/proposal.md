# add-lingua-language-sync-client — the extension syncs each card in its own language

## Why

The server keys deck cards by studied language since `add-lingua-card-language` (backend 0.35.0,
deployed 2026-09-30). It returns only the cards in the languages a pull names, English when the
pull names none, and its data state says whether it understands card languages. The extension
still syncs as if every card were English:
- a pulled card is filed under `"en"` whatever the server sends (`sync.ts`);
- a pushed card carries no language, so the server stores it as English;
- a pull names no languages;
- statuses are pulled for every language and applied whatever the reader studies.

This is change 11 of the Spanish programme (`docs/lingua/spanish-programme.md`), the first of R3,
a silent English release. Every reader's languages are English alone and the package ships
en-fr only, so the requests carry `en` and nothing a reader sees changes. The path is ready for
the day a reader studies Spanish.

## What Changes

- **The languages a device accepts** are the reader's studied languages that the package ships,
  or the default pair's language when none is (`acceptedLanguages` in `src/analyzer/pairs.ts`,
  the list form of `readingLanguage`).
- **A pull names them.** `PullCardsRequest.languages` carries the accepted languages. A pulled
  card is filed under the language the server returns, empty read as `en`. Statuses and declared
  levels in a language the device does not accept are not applied. The server has no filter for
  them, and applying one would load a pack the reader does not use.
- **A push carries each card's language** (`CardOp.language`). A card in another language than
  English is pushed only when the data state read at the start of the sync says the server
  understands card languages (`card_language`). Otherwise the device pushes its English cards
  only, as the server's contract asks.
- **Widening the accepted languages pulls again from the start.** The device remembers the
  languages of its last successful sync. When a language is added, both pull cursors go back
  to 0: the server keeps no memory of what it withheld, and the statuses of that language were
  skipped too. Re-applying what the device already holds changes nothing (last-write-wins). A
  device that has never stored the list is read as having accepted English, so updating the
  extension triggers no full re-pull.
- **Nothing else moves.** Daily statistics stay English until the statistics carry a language
  (`add-lingua-language-stats-review`). The status push already names each status's language.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-sync`: ADDED, three requirements:
  - *A device pulls the cards of its reader's languages*;
  - *A device pushes a non-English card only to a server that keys cards by language*;
  - *Widening a device's languages pulls again from the start*.

  The open changes that create this capability (`add-lingua-backend`,
  `add-lingua-connected-clients`, `add-lingua-card-language`) only add requirements under other
  names. This change archives after `add-lingua-card-language`.

## Impact

- **Products.** Cymbra Lingua's browser extension only: `src/sync/sync.ts`,
  `src/analyzer/pairs.ts`, and the sync specs. No server, proto or engine change: the fields exist
  since `add-lingua-card-language`. The Apple app embeds the extension, so it follows when it ships
  the bundle. The agent plugin is `add-lingua-agent-languages`.
- **Release.** R3, silent: requests carry `en`, as the server already assumed. Per
  `add-lingua-card-language` (task 5.2), no store package carries this before that change's
  checks from outside are done. Merging into `main` publishes nothing.
