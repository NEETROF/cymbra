# refine-lingua-review-language — a review in the language being read

## Why

The review page of a reader of several languages offers « Toutes », then one segment per language
(`add-lingua-language-stats-review`). « Toutes » mixes the languages' due cards in one queue, by
due date: the programme's decision D8, « mixed review queue ».

Dogfooding Spanish, the owner judged that a review never mixes languages (2026-10-05). The review
should open in the language being read, and its counts should follow. Today:
- the « 25 carte(s) » beside a selected Spanish segment counts every language's cards;
- the popup's « Réviser (N) » counts every language's due cards.

## What Changes

- **No « Toutes ».** A review session and its counts cover one studied language. The segments name
  the reader's languages only.
- **The language it opens in:**
  1. the page's or the book's, when the review opens beside one in a language the reader studies;
  2. otherwise the last language the reader chose in the review on this device;
  3. otherwise the reader's first language.
- **The page's language reaches each host:**
  - the in-page drawer: the reading session's document language;
  - the book reader: the same session, reading the book;
  - Chrome's side panel: it asks the active tab's reading session, as the popup already does;
  - the popup's « Réviser (N) »: the page's language too.
- **A choice is remembered.** Choosing a language in the review keeps it as the last one, on the
  device, beside the other preferences. It holds while the page's language does not change.
- **The counts follow the language.** « N carte(s) · M à revoir » counts that language's cards. The
  engine's deck count takes the same language filter its due count takes.
- **Decision D8 is superseded.** The programme notes the owner's decision.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-decks-review`: a requirement is added. A review is in one language, the one being read,
  else the last chosen, else the first.
  - « One review queue across languages, with a language filter » is held by the open change
    `add-lingua-language-stats-review`, so it is not modified here.
  - Its mixed queue and its « all of them » are reconciled by `refine-lingua-language-wording`
    (change 33), the change the programme keeps for the requirements open changes hold, once they
    archive.

## Impact

- **Products:**
  - Cymbra Lingua: the review page, the popup's count, the side panel;
  - nothing is consumed from ID or the platform.
- **Code:**
  - `crates/lingua-core/src/decks/review.rs`: a deck count by language;
  - `crates/lingua-wasm/src/lib.rs`: `deckCount(languages?)`;
  - the extension's port, messaging port and engine;
  - `src/review/review-page.ts`;
  - `src/reading/drawer.ts`, `session.ts` (the drawer's language, the popup's counts);
  - `src/sidepanel/sidepanel.ts`;
  - their tests.
- **Data:** one device-local preference in `chrome.storage.local`, the last language chosen in the
  review. Nothing synced, nothing in the backup.
- **English alone:** unchanged. No segment is shown, and the counts are the same.
