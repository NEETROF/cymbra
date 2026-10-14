# add-lingua-review-stop-session — design

## Context

- The session's queue lives in the engine (`lingua-core`, `ReviewSession`). Each answer is applied
  and written when it is given; `summary()` reports what the session has done *so far*, whenever it
  is asked. A card the session has not shown has been read, never changed.
- `ReviewController` (`src/review/session.ts`) drives the engine for every host — the side panel,
  the Firefox sidebar, the in-page drawer (Safari included) and the Cymbra Lingua app. At the end of
  the queue its `next()` reads the summary, then measures whether another session (« Encore 10 ») or
  practice would hold cards, by preparing it in the engine — which replaces the session that ended.
- `mountReview` (`src/review/review-page.ts`) runs each action, then — for an action that changed
  the deck — saves the backup and refreshes the counts. The session's last answer is such an action:
  the end of a session is always followed by a save made with no session under way.
- `renderReview` (`src/review/view.ts`) draws the card head: the count of cards remaining (or the
  practice note, or the repeat note), then the card's language when the reader studies several.
  The review already reads keys on its own root: Space or Enter reveals, ← and → answer.

## Goals / Non-Goals

**Goals:** a session under way can be ended in one tap, the same way in every host, losing nothing.

**Non-Goals:** a confirmation dialog; ending a session from outside the review (closing the page
already does); resuming an ended session; any engine, backup, sync or wire change.

## Decisions

### D1 — Where and when the link shows
« Terminer » is a small link (`review-link`) at the end of the card head, pushed to the right of the
line holding the count of cards remaining, with no padding of its own so the line keeps its height.
It shows on every card of a session under way — before and after the reveal, on a first showing and
on a card that came back, in a review and in practice — and nowhere else: not before a session
starts, not on the end. Its accessible name says what it ends (« Terminer la séance », matching the
end's « Séance terminée »).

No keyboard shortcut. The review has keys (Space, Enter, ←, →), so Escape was considered; but Escape
dismisses a popup, a panel or a recap everywhere else in Lingua and never acts on the reader's data,
and beside a book the reader's own Escape handling shares the page. Ending a session is rare and
never urgent; the link is reachable by Tab like the card's other links.

*Alternative.* A button in the action zone: bigger, and it would change the zone the reader's thumb
answers in. A link in the head is out of the way and moves nothing.

### D2 — What ending does to each card
The controller's `end()` drops the current card and runs the end of the session — the same path the
last answer takes. Nothing is asked of the engine about the cards themselves:
- a card already answered keeps its answer: it was graded, written and counted when answered;
- a card not yet shown is untouched — no answer, no showing — and stays due;
- a missed card waiting to come back is not asked again: its first answer, `again`, stands, and it
  is due when that answer scheduled it;
- the current card, revealed or not, is left unanswered: it stays due as it was;
- a card marked known or hidden in the session stays so.

Preparing the next session to measure « Encore 10 » replaces the engine's session, so the rest of
the old queue is simply forgotten. The page's count of showings goes with the session.

### D3 — The end screen
The end is the usual one: the session's summary, then « Encore 10 » while cards remain due, else
practice when it would hold cards. Ending early usually leaves cards due, so « Encore 10 » is usually
offered, and starts a fresh session over what is due — the unseen cards among them.
One difference: a session ended before any card was answered says it ended (« Séance terminée »)
with no count, instead of « Rien à réviser pour l'instant », which would contradict the « Encore 10 »
under it. The view model says the session was ended by the reader (`ended`).

### D4 — Every host, through the same end-of-session path
The link is drawn by the shared review view and the action lives in the shared review page, so the
side panel, the Firefox sidebar, the drawer, Safari and the Cymbra Lingua app all get it at once.
The page runs `end()` like an action that changed the deck: it saves the backup and refreshes the
counts, with no session under way. That is what a host sees at the end of every session; the app's
direct deck sync (`add-lingua-app-deck-sync`, in flight) pushes on exactly that, so ending early
pushes too, without either change depending on the other. The save writes nothing new when no card
was answered — the backup is the same — which costs one write.

### D5 — E-ink
A link inside the existing head line, no new block, no transition and no animation (the review's
stylesheet rule), coloured from the tokens like the card's other links. Pressing it repaints once,
straight to the end.

## Risks / Trade-offs

- [A reader taps « Terminer » by mistake] → nothing is lost: every answer is kept, and « Encore 10 »
  starts again over the same due cards. A confirmation would cost every deliberate tap for a mistake
  that costs one tap to undo.
- [A missed card is not asked again] → its miss is already its review for the session; asking it
  again would only decide whether it counts as recovered. Acceptable: the reader chose to stop.
