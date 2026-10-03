# Design — add-lingua-language-sync-client

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `backend/lingua/proto/deck.proto` | `CardOp.language` (11, empty = `en`) and `PullCardsRequest.languages` (2, empty = English only). Withheld cards are not consumed: the cursor is the highest sequence among the cards returned. |
| `backend/lingua/proto/lingua_data.proto` | `GetDataStateResponse.card_language`: whether the server keys cards by language. A server that predates the field answers `false`. |
| `backend/lingua/proto/known_words.proto` | `PullChangesRequest` has a cursor and no language filter: statuses and declared levels of every language come back. |
| `src/sync/sync.ts` | `sync()` reads the data state first (`checkErasure`), pushes statuses, levels, cards and statistics, fetches statuses and cards, applies them, saves the backup, then the two cursors. Pulled cards are built with `language: "en"`; pushed cards carry no language; daily statistics are sent as `"en"`. |
| `src/analyzer/engine.ts` | The applies load the pack of every listed language their records name (`package-lingua-packs-per-pair`). |
| `src/analyzer/pairs.ts` | `readingLanguage(port, pairs)`: the reader's first studied language that a shipped pair studies. |

## Goals / Non-Goals

**Goals:**
- Cards pulled, filed and pushed in their own language.
- A device holds the reader's languages, and never loses what it did not accept yet.
- English unchanged: the same records, under the same keys.

**Non-Goals:**
- Daily statistics per language (`add-lingua-language-stats-review`).
- The reader's choice of languages (`add-lingua-language-choice`).
- A status filter on the server: the device filters, and a server change is not needed for it.

## Decisions

### D1 — The accepted languages: the reader's, as the package ships them

`acceptedLanguages(port, pairs = SHIPPED_PAIRS)` returns the reader's studied languages that a
shipped pair studies, in the reader's order. When none is shipped, it returns the default pair's
language: a device always accepts one language, as `readingLanguage` always reads in one.

Why the reader's languages and not every shipped language: the profile is per device (D9 of
the programme), and a device that holds a language the reader does not study there would load
its pack for nothing. The study's rule, « re-pull when the declared set grows », is about this set.

### D2 — The pull names them; the device filters what the server cannot

- `pullCards` sends `languages: accepted`, and files each card under `c.language || "en"`. The
  server normalises the value already.
- Statuses and declared levels are filtered against the accepted languages before they are
  applied. A dropped record is not lost: the cursor moves past it, and D4 brings it back when its
  language is accepted.

### D3 — A non-English card waits for a server that keys cards by language

`checkErasure` already reads `getDataState`. It keeps `cardLanguage` for the sync. `pushCards`
sends `language` with every card, and leaves out the non-English ones when `cardLanguage` is
false. They stay on the device and go up the first time the server says it understands them.

### D4 — Re-pull from the start when the accepted set grows

The store keeps the languages of the last successful sync (`cymbra-lingua-sync-languages`). At
the start of a sync:
- if a language is accepted now that the stored set lacks, both cursors are set to 0 before
  the fetch;
- the new set is saved after the cursors, at the end of a successful sync.

A device that never stored the set is read as `["en"]`, what every client before this change
accepted. So updating the extension resets nothing.

Why both cursors: statuses of a language the device did not accept were skipped (D2), and before
this change an engine without the pack skipped them too (`generalise-lingua-wasm-engine`).

*Rejected — one cursor per language.* The server's cursors are per user, not per language, and
the card cursor is the highest sequence among the cards returned. A per-language cursor would
need a server change for the same result.

### D5 — The seam for tests

`SyncDeps` gains an optional `acceptedLanguages: () => Promise<StudiedLanguage[]>`, which defaults
to `acceptedLanguages(port)`. Specs can then sync as a device that accepts Spanish while the
bundle ships en-fr alone.

## Risks / Trade-offs

- **A full re-pull is heavier** → it happens once, when the reader adds a language. Applies are
  idempotent last-write-wins.
- **A reader who removes a language keeps its local records** → they stay in the backup and are
  still pushed. Narrowing never deletes anything.
- **An old server** never receives non-English cards (D3) → they wait on the device. The server
  in production understands card languages since 0.35.0.
