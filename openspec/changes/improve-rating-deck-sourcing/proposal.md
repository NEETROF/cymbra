## Why

The swipe-rating deck (change: add-app-score-rating) sources cards through the
plain catalog search: **accepted piano scores ordered alphabetically by title**,
with only a within-session de-dup. Two consequences hurt the feature:

- **It re-shows already-rated scores** across sessions (there is no per-user
  "un-rated" filter — that prioritisation was deferred), so the deck never truly
  empties and the user is asked to re-judge what they already judged.
- **The order is the same for everyone** (alphabetical), so discovery is poor and
  the scores that most need community signal (few or no ratings) are not surfaced
  first — and titles late in the alphabet are rarely reached.

Separately, the library **"rate a few scores" invite** re-appears every few days
**indefinitely**: there is no terminal state, so a user who is simply not
interested keeps being nudged.

## What Changes

- **Deck sourcing = un-rated first, least-rated first (backend).** A dedicated
  operation returns the caller's **un-rated** `accepted` (piano) scores, ordered
  by ascending rating count (the scores that most need a vote come first), with a
  deterministic tiebreak, paginated. When the user has rated everything, it
  returns empty — the deck reaches its last-card/empty state naturally and the
  nudge has nothing left to point to.
- **The deck consumes the new source** instead of catalog search.
- **The invite eventually stops.** After the user **dismisses** the invite a
  configured number of times it is **not shown again** (a persisted dismissal
  count), so an uninterested user is never nagged past that point. Rating still
  resets the normal snooze window.

Out of scope: personalised/recommender ranking beyond least-rated-first; changing
how ratings feed the hub or the moderation re-review flag (unchanged).

## Capabilities

### Modified Capabilities

- `score-rating`: adds a per-user **deck-sourcing** read — the caller's un-rated
  `accepted` scores, least-rated first — alongside the existing submit/aggregate.
- `swipe-rating-deck`: the deck sources through that read (un-rated first), and the
  first-run/coaching capability gains a **terminal stop** for the recurring invite
  after repeated dismissals.

## Impact

- **Backend** (`cymbra-music`): new `ListRatingDeck` RPC + a `rating_deck` read on
  the catalog port (SQL `LEFT JOIN score_ratings` excluding the caller's rated
  scores, `ORDER BY rating_count ASC`); module method; no schema change.
- **App** (`apps/music`): a `ratingDeck` service call replaces `search` in the
  deck notifier; the rating-invite gains a persisted dismissal counter with a stop
  threshold.
- **Coverage**: Rust ≥ 80% for the sourcing query logic (fakes); Flutter ≥ 80% for
  the notifier + invite stop.
</content>
