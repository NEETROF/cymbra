# add-lingua-review-practice — practise when nothing is due

## Why

When no card is due, « Réviser » leads to « Rien à réviser pour l'instant » and nothing else: a reader
with ten minutes on the sofa and a deck of words cannot use it. Dogfooding the Cymbra Lingua app on an
iPad (2026-10-13), the product owner met that wall — 10 English cards, 0 due — and asked to be able to
review anyway.

## What Changes

- **Practice when nothing is due**: where the review says nothing is due, and at the end of a session
  that leaves nothing due, the review offers to practise anyway (French UI copy: « S'entraîner quand
  même »).
- **A practice session** holds up to 10 cards already reviewed and not retired, those the reader is
  most likely to have forgotten first, whether or not they are due. Its answers **never change a
  card's schedule** — its due date, its memory, its history — so practising cannot move a card's next
  review. A missed card still comes back in the session, as in a review. Marking a word known or
  hiding it stays the reader's decision and applies as in a review.
- **It is said on the card**: a practice card says it is practice, in place of the count of cards
  remaining (French UI copy: « Entraînement · N carte(s) »).
- **It counts in the day's statistics**: each card's first answer counts as a review in the day's
  counts, as the product owner decided on 2026-10-13. Nothing of it is synced as a card change, since
  no card changes.
- Every review host: the side panel, the in-page drawer and the Cymbra Lingua app.
- Copy in French, English and Spanish.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-decks-review`: added — a practice session when nothing is due, whose answers leave every
  schedule as it was.

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The core's review session (`crates/lingua-core`, a practice
  option), its WASM binding (`crates/lingua-wasm`, the session's start), and the extension's review
  (`apps/lingua-extension`: the port's options, `ReviewController`, the review view, the catalogue's
  `review` module), shown by the side panel, the drawer and the Apple app.
- Cymbra ID, Music, Live, the back office and the public site are untouched. No backend, no `.proto`,
  no flag. The backup format is unchanged; a saved review session gains a field older engines ignore.
