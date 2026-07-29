## Why

The swipe-rating deck was meant to let the community help moderate the catalog, but
today it sources **only `accepted` scores** — so community ratings never touch the
`pending` backlog they were supposed to help clear. Ratings only re-litigate scores a
moderator already validated, which is low value. Letting the community rate `pending`
scores too turns the deck into real moderation signal: the scores that most need a
human decision get community input **before** a moderator ever looks at them.

## What Changes

- The deck sources **`pending` and `accepted`** scores (never `rejected`). Un-rated,
  least-rated-first ordering is unchanged; pending scores simply join the pool.
- A signed-in user can **submit a rating on a `pending` or `accepted`** score; a
  `rejected` (or unknown) score is still un-rateable.
- The in-card preview can **play/hear a `pending` score** via a **dedicated
  rating-preview byte path** — a narrow carve-out to the "normal callers see only
  `accepted`" rule (change: `add-score-moderation-gating`). This path is **deck-scoped
  and read-only**: it serves bytes for previewing-before-rating only; it does **not**
  open a pending score in the full player, nor let it be saved to a library, nor appear
  in hub search. Attribution (licence + source) stays visible, and a pending card is
  labelled as a **"potential new score"** (positive framing — the user is helping
  evaluate a candidate, not consuming validated content).
- Community ratings on a `pending` score **prioritise the moderator queue** (they feed
  the same `needs_review` / review-priority signal just wired into the back-office).
  They **never auto-validate** — a moderator always makes the final `accepted`/`rejected`
  decision (a locked decision from `add-score-moderation-gating`).

## Capabilities

### New Capabilities
<!-- none — this modifies existing rating/deck/moderation behaviour -->

### Modified Capabilities

- `swipe-rating-deck`: the deck sources `pending` + `accepted` (was `accepted`-only);
  the in-card preview plays pending scores through the rating-preview path; a pending
  card is labelled "potential new score".
- `score-rating`: submitting a rating is allowed on a `pending` score (was
  `accepted`-only); `rejected`/unknown stays un-rateable.
- `score-moderation`: a narrow, deck-scoped **rating-preview byte path** may serve a
  `pending` score's bytes to a signed-in rater; the existing accepted-only gates on
  hub search, `GetCatalogScoreBytes` (player open), and library save are unchanged.

## Impact

- **Backend (`backend/music`)**: `rating_deck` sourcing SQL (`IN ('pending','accepted')`);
  the `submit_rating` gate (resolve pending+accepted, reject `rejected`); a
  rating-preview byte-fetch path (module + gRPC), authorised for a signed-in caller and
  scoped to a deck-sourced score. Proto: a preview-bytes RPC (or a flag on the existing
  fetch) that permits pending only under the rating context.
- **App (`apps/music`)**: the deck card's preview uses the rating-preview fetch; a
  "potential new score" label on pending cards; the deck/rating notifiers surface the
  new source pool. No player-open / save changes.
- **Moderation**: pending scores accumulate community signal that surfaces via the
  already-wired `needs_review` / review-priority ordering; no auto-validation.
- **Relates to**: `add-app-score-rating` (#2), `add-score-moderation-gating` (#1),
  `improve-rating-deck-sourcing`, and the merged `review_queue` wiring
  (`add-moderation-back-office`).
