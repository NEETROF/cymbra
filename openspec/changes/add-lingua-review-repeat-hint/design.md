# add-lingua-review-repeat-hint — design

## Context

- The session's queue lives in the engine (`lingua-core`, `ReviewSession`): a missed card is put
  back after three other cards, at most three asks (`RETURN_AFTER`, `MAX_ASKS`). The engine counts
  the asks of each card but its card view (`reviewCurrent`) does not expose them.
- `ReviewController` (`src/review/session.ts`) is the one module every host drives — the side panel,
  the in-page drawer and the Cymbra Lingua app (`add-lingua-apple-review`). It reads each new card in
  `next()`, after a start, a grade, a mark-known or an ignore; `reveal()` reads the same card again.
- `renderReview` (`src/review/view.ts`) draws the card head: the count of cards remaining
  (`copy.remaining`), then the card's language when the reader studies several.

## Goals / Non-Goals

**Goals:** a card shown again in a session says so, the same way in every host and in the
interface language.

**Non-Goals:** changing when or how often a missed card comes back; saying it on the end summary
(it already counts the cards recovered); any engine, backup, sync or wire change.

## Decisions

### D1 — The controller counts the showings
`ReviewController` keeps, for the session, how many times each card was shown, keyed by language
and headword (one card per language and lemma, `lingua-decks-review`). It counts in `next()` only —
a card arriving — never in `reveal()`, which shows the same card's answer. A new session (`start`)
starts the count afresh. The view model gains `showing: number` (1 for a first showing).

*Alternative.* Expose the engine's own ask count on the card view: exact across a host's reload, but
a core, WASM and port change for a hint.

### D2 — The note replaces the remaining count
On a showing after the first, the head's first note is « À revoir · Nᵉ passage » instead of
« N carte(s) à revoir »: the line keeps its place and its height, so nothing moves on an e-ink
screen. The card's language note is unchanged.

### D3 — The copy is a function of the showing
`review.repeat(showing)` in each language's `review` module: « À revoir · 2ᵉ passage » (French, the
product owner's), "Again · 2nd time" (English), « Otra vez · 2.ª vez » (Spanish). The showing is at
most `MAX_ASKS` (3); the English ordinal is written for 2 and 3 and falls back to "Nth" beyond.

## Risks / Trade-offs

- [A host reloaded mid-session (Safari stopping the drawer's page) keeps the engine's queue but
  loses the page's count] → the card that comes back then reads as a first showing, as today.
  Rare, and never wrong the other way: a card is never said to come back when it does not.
- [Two cards with the same headword in two languages] → the key carries the language.
