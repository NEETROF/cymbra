# Design — add-lingua-language-stats-review

## Context

See proposal.md (Why). Today:

| Where | What it does |
|---|---|
| `crates/lingua-core/src/decks/review.rs` | `ReviewSession::start(deck, now)` queues `deck.due_keys(now)`, keys `(language, lemma)` in due order, every language mixed. |
| `crates/lingua-wasm` | `startReview(now)`, `dueCount(now)` over every language. `reviewCurrent()` returns `{ headword, surface, sentence, source, gloss, revealed, remaining }`, without the language. |
| `src/review/session.ts`, `review-page.ts` | `ReviewController` starts, grades and marks known, and calls `record("review" \| "learned")`. The summary shows the deck and due counts. |
| `src/state/dailystats.ts` | `cymbra-lingua-daily-v2`: day → `{ exposures, unknownSeen, wordsLearned, reviews }`. The session records reading and words learned; the review page records reviews and words learned. |
| `src/sync/sync.ts` | `pushStats` sends each day with `language: "en"`. |
| `src/stats/view.ts` | `mountStats` shows the reader's first language (`readingLanguage`). |

## Goals / Non-Goals

**Goals:**
- One review queue across languages, filterable, each card naming its language (D8).
- Statistics counted, shown and synced per language.
- English unchanged: one language, no control shown, the day-only statistics as English.

**Non-Goals:**
- The server's consolidated statistics read (`GetStats`): the extension's statistics are the
  device's own.
- One review session per language: the product owner chose a mixed queue (D8).

## Decisions

### D1 — An optional language list on the review bindings

`ReviewSession::start_for(deck, now, languages)` keeps the due keys whose language is listed. An
empty list keeps every key, so `start` is `start_for` with none. The engine's `startReview(now,
languages?)` and `dueCount(now, languages?)` take an optional array of tags; absent, or with no
known tag, means every language. `reviewCurrent()` adds `language`, the tag of the current key.

`due_keys` returns the deck's order, (language, lemma), which is not by due date: the English
review has always run alphabetically. A queue that mixes languages is sorted by due date, as D8 asks,
new cards first, then by language and lemma. A single language keeps the deck's order, so the
English review, which the S0 baseline replays, does not move.

### D2 — The review page's filter

The page reads `acceptedLanguages`. With more than one, it shows a segmented control: « Toutes »,
then one segment per language, named from `language-labels`. The choice is local to the page, starts
at « Toutes », and passes `[language]` or nothing to `startReview` and `dueCount`. The card view shows
the card's language as a tag when several languages are accepted. `ReviewController` records each
grade and each mark-known with the card's language.

### D3 — Daily statistics per language, v3, reading v2 as English

`cymbra-lingua-daily-v3` holds day → language → `DailyStat`.
- **Reading.** `loadDailyStats` reads v3. When v3 is absent, it reads v2 and files each day under
  `en`. The first write after an update therefore carries the old days into v3, and v2 is never
  read again. `clearDailyStats` clears both.
- **Recording.** The recorders take a language: reading and learning record in the session's
  document language, reviews and learning in the review card's language.
- **Syncing.** `pushStats` sends one `DailyStat` per day and language, with that language.

*Rejected — keep v2's shape with a nested map for the other languages.* Two shapes in one value
would make every reader and writer handle both forever.

### D4 — The statistics page's language selector

`mountStats` reads `acceptedLanguages`. With more than one, it shows a segmented control, the first
language selected. The ladder, estimate, seed, marked words and daily figures are those of the
selected language, and choosing another re-renders the page. With one, there is no control and
nothing changes.

## Risks / Trade-offs

- **An older build after a downgrade reads v2 only**, which stops growing after the first v3 write →
  it shows the counts up to the update. Statistics are local and lossless in v3.
- **Two languages double the statistics a device sends per day** → one small row per language,
  which the server's (day, language) key already expects.
