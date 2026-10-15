# add-lingua-review-stop-session — the reader can end a session in progress

## Why

Once a review session has started (at most 10 cards, `refine-lingua-review-session`), the only ways
out are answering every card or closing the page. Dogfooding on 2026-10-14, the product owner wanted
to stop after a few cards and found no way to do it: a short session is meant to fit a short moment,
and a moment can end before the cards do.

## What Changes

- **A small « Terminer » link beside the count of cards remaining**, at the top of the card, shown
  only while a session is under way (a review or a practice session). A link in the existing line —
  no new block, no transition — so nothing moves on an e-ink screen. Its accessible name says what it
  ends (French UI copy: « Terminer la séance »).
- **Ending is safe, so it asks nothing**: the cards already answered keep their answers (already
  saved); the cards not yet shown stay due, untouched — no answer, no showing; a missed card waiting
  to come back is not asked again in this session, its miss standing; a card revealed but not
  answered stays unanswered.
- **Then the usual end of a session**: what the session did, then « Encore 10 » while cards remain
  due — which they do after ending early — or practice as today. Ending before answering any card
  says the session ended without a count, never « Rien à réviser pour l'instant ».
- **Practice too** (`add-lingua-review-practice`): the same link, the same rule; nothing is
  scheduled by practice anyway.
- **No new keyboard shortcut**: the review answers Space, Enter and the arrows, but Escape stays a
  dismissal everywhere in Lingua, never an action on the reader's data (design D1).
- Copy in French, English and Spanish: « Terminer » / "End" / « Terminar ».
- Every review host: the side panel (Chrome), the sidebar (Firefox), the in-page drawer, Safari, and
  the Cymbra Lingua app, which all render the same review page.
- Unchanged: how a session is built, what an answer does, the end summary's counts, the engine, the
  backup and sync.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-decks-review`: a new requirement — the reader can end a session in progress, with what
  ending does to each card and the end it leads to.

## Impact

**Products.**
- **Cymbra Lingua — changed behaviour.** The review (`apps/lingua-extension`, `src/review/`), shown
  by the side panel, the sidebar, the drawer and the Apple app (`apps/lingua-apple`, through the `app`
  build). New copy in the catalogue (`review` module, fr/en/es).
- The core (`crates/lingua-core`) is untouched: its session already reports what it has done so far,
  and the cards it has not shown are left as they are.
- No WASM interface, backup, sync, backend, `.proto` or flag change.
- Cymbra ID, Music, Live, the back office and the public site are untouched.

**Code.** `src/review/session.ts` (ending the session), `src/review/view.ts` (the link and the end
without a count), `src/review/review-page.ts` (the action, through the usual end-of-session save),
`src/styles/review.css`, `src/i18n/{fr,en,es}/review.ts`.

**In flight.** The app's direct deck sync (`add-lingua-app-deck-sync`, not merged) pushes at the end
of a session, read as a save with no session under way. Ending early takes the same path, so that
push happens; this change does not depend on that one.
