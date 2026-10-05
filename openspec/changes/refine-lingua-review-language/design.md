# Design

## Context

- **The review page** (`mountReview`, `src/review/review-page.ts`) is one builder, mounted:
  - by the in-page drawer (`reading/drawer.ts`, created by the reading session: Firefox, Safari,
    and the book reader there);
  - by Chrome's side panel (`sidepanel/sidepanel.ts`).
- **Its filter:**
  - it offers `null` (« Toutes ») then each accepted language, hidden with one;
  - `only()` passes `undefined` for « Toutes », which the engine reads as every language, due
    order across them;
  - its summary calls `deckCount()`, every language, and `dueCount(now, only())`.
- **The reading session knows the document's language** (`this.language`, chosen by the routing
  vote). `getStats`, which the popup sends the active tab, already returns it as
  `PageStats.language`, with `deckCount()` and `dueCount(now)` over every language.
- **The engine** counts due cards in given languages (`Deck::due_count_for`). Its deck count is the
  deck's length, every language.

## Goals / Non-Goals

**Goals:**
- One language per review, opened on the language being read, else the last chosen, else the first.
- The review's counts and the popup's count in that language.

**Non-Goals:**
- The statistics page's language. It keeps its own choice, the reader's first language by default
  (`add-lingua-language-stats-review`).
- Syncing the last choice across devices.
- Removing the engine's mixed due order. Nothing asks for it any more, and it costs nothing to keep.

## Decisions

### D1 — One language, no « Toutes »

The filter's segments are the accepted languages only, still hidden with one. `only()` always
names one language: the session, the due count and the deck count ask for it alone.

A session over in one language gives way to a new start when the language changes. The widget would
otherwise keep saying that nothing is left to review, of a language no longer shown.

### D2 — Which language: the page's, else the last chosen, else the first

`mountReview` takes an optional `pageLanguage(): Promise<string | null>` from its host, and asks it
on each refresh. When the answer differs from the page language it last followed (the first
refresh, or a page in another language), it resolves, in order:
1. the page's language, when it is one of the accepted languages;
2. the last language chosen, `cymbra-lingua-review-language` in the preferences area
   (`chrome.storage.local`), when it is accepted;
3. the first accepted language.

Otherwise the review keeps its language, so a choice holds (D3). A session under way keeps its
language whatever the page says. A language that leaves the accepted ones is resolved again.

The page's language from each host:
- **The in-page drawer:** the reading session gives `() => this.language`. On a book this is the
  book's language, as the reader runs the same session.
- **Chrome's side panel:** sends `getStats` to the active tab, the message the popup sends, and
  reads `language` from the answer. No answer (no reading session there, an extension page) leaves
  it null.

### D3 — The choice is remembered

Clicking a segment selects that language and writes it as the last chosen.
- It holds while the page's language does not change: switching views, closing and reopening the
  drawer beside the same page, a sync pull. A page in another language brings the review to that
  one.
- The last choice serves away from a page.
- It is a device preference, like the HUD toggle: never synced, never in the backup.

### D4 — The counts by language

- The core gains `Deck::count_for(languages)`. With no languages it counts them all, as
  `due_count_for` does.
- The wasm `deckCount` takes the optional `languages` its `dueCount` takes, with the same reading of
  an empty or unknown list.
- The port, the messaging port and the engine wrapper pass it.

The review's summary then counts the resolved language's cards. `getStats` counts the deck and the
due cards in the page's language, so the popup's « Réviser (N) » follows the page too.

### D5 — English alone

With one accepted language:
- the filter is hidden;
- the resolved language is English, the page's or the first;
- the counts name English, whose cards are every card.

So nothing a reader of English alone sees changes, and the English baseline does not move.

## Risks / Trade-offs

- **The side panel beside the book reader.** The panel asks the active tab with the popup's
  message, which the book reader answers (`session.ts`). Where nothing answers, the panel opens on
  the last choice.
- **A choice and the next page.** With Chrome's side panel left open, a choice of English made
  beside a Spanish page holds on the next Spanish page, as the page's language has not changed. The
  drawer belongs to one page, so the next page's drawer opens in Spanish.
- **A requirement held elsewhere.** « One review queue across languages » still says the review
  page offers all languages. It is reconciled after `add-lingua-language-stats-review` archives, by
  `refine-lingua-language-wording` (change 33).
