## Context

The moderation initiative shipped in layers: `add-score-moderation-gating` (#1) made
`pending`/`rejected` scores invisible to normal callers (hub search, `GetCatalogScoreBytes`,
library save all serve `accepted` only); `add-app-score-rating` (#2) added a swipe deck
that rates `accepted` scores and computes a hybrid `needs_review` flag; the just-merged
`review_queue` wiring surfaces community-flagged scores in the back-office queue.

The deck's sourcing (`improve-rating-deck-sourcing`) is `accepted`-only, so the community
can only re-rate validated scores — the ratings never reach the `pending` backlog. This
change opens the deck to `pending` scores so community signal arrives **before** a
moderator's first decision, while keeping the moderator as the sole validator.

The tension: rating a pending score requires previewing it (the deck's "listen before
rating" rule), but pending scores are crawled, **not yet validated** content that #1
deliberately hid from users. We reconcile this with a **narrow, deck-scoped preview
path** rather than reopening the accepted-only gates wholesale.

## Goals / Non-Goals

**Goals:**
- Source `pending` + `accepted` (never `rejected`) in the deck, least-rated-first.
- Allow submitting a rating on a `pending` or `accepted` score; reject `rejected`/unknown.
- Let the in-card preview play a pending score through a dedicated rating-preview byte
  path, read-only and deck-scoped, with attribution and a "potential new score" label.
- Feed community ratings on pending scores into the existing `needs_review` /
  review-priority signal so moderators triage the best/worst-rated pending scores first.

**Non-Goals:**
- **No auto-validation.** A moderator always makes the final `accepted`/`rejected` call
  (locked decision from #1). Consensus only prioritises, never promotes.
- No change to hub search, player-open (`GetCatalogScoreBytes`), or library save — those
  stay `accepted`-only. A pending score is never openable in the full player or savable.
- No new "trusted rater" tier or gating of who may rate pending (the community at large
  rates, per the stakeholder's intent).
- No moderation UI change (the back-office already consumes `needs_review`).

## Decisions

### D1 — Deck sources `pending` + `accepted`, `rejected` excluded
`rating_deck` changes its `WHERE moderation_status = 'accepted'` to
`moderation_status IN ('pending','accepted')`. Ordering (un-rated by caller, fewest
ratings first, `id` tiebreak) is unchanged. Rejected scores are never surfaced.
- **Alternative — pending-only deck**: rejected. Accepted scores still benefit from
  re-review signal (the `needs_review` flag), and one pool keeps the UX simple.

### D2 — Rating gate resolves `pending` + `accepted`, not `rejected`
`submit_rating` currently resolves the target through the accepted-only path
(`object_key(id, false)`), so a pending id reads as not-found. It gains a
rating-scoped resolution that accepts `pending` or `accepted` and rejects `rejected`
(and unknown). One rating row per (user, score) is unchanged; a score later rejected
keeps its rows but leaves the deck.

### D3 — A dedicated, deck-scoped rating-preview byte path (the key trade-off)
Previewing a pending score needs its bytes, but reusing `GetCatalogScoreBytes` (which #1
gates to `accepted`) would either weaken that gate or fail for pending. Instead add a
**separate rating-preview byte fetch** whose authorisation rule is: a **signed-in caller**
may fetch bytes for a score that is **`pending` or `accepted`** (never `rejected`),
**for previewing in the deck only**. This is a deliberate, minimal carve-out:
- It does **not** touch `GetCatalogScoreBytes` (player open) or library save — those stay
  `accepted`-only, so a pending score can be *heard in the deck* but never *opened/kept*.
- The app wires the in-card preview (not the player) to this path; the player's open path
  is untouched.
- **Alternative — a boolean `allow_pending` flag on `GetCatalogScoreBytes`**: rejected;
  overloading the player-open RPC blurs the gate and risks a caller passing the flag to
  open a pending score in the full player. A distinct RPC keeps the capability legible
  and independently auditable.
- **Alternative — extract-only preview (first N measures)**: deferred (see Open
  Questions); more backend complexity (server-side MusicXML truncation) for a marginal
  copyright reduction on content the user is explicitly evaluating.

### D4 — Positive "potential new score" framing, attribution retained
A pending card is labelled as a candidate the user is helping evaluate ("potential new
score"), not as "unvalidated/unsafe" content — the stakeholder's chosen framing. Licence
+ source attribution stay visible (as for accepted cards). This sets expectations
(the score may change or be removed) without a scary/legalistic tone.

### D5 — Consensus prioritises, never validates
Ratings on a pending score flow into the same aggregate + `needs_review` /
review-priority signal the back-office already sorts on. A well-rated pending score rises
for a moderator's attention (candidate to accept); a poorly-rated one rises as a likely
reject. The moderator still decides. No threshold ever flips `moderation_status`
automatically. (Extending the exact ranking of pending-by-community-score in the queue is
an incremental follow-up on the merged review-priority sort, not a new mechanism.)

## Risks / Trade-offs

- **Exposure of unvalidated / possibly-infringing content to all raters** → mitigated by
  D3's narrow path (deck preview only; no player-open, no save, no search), retained
  attribution, the "potential new score" label, and the fact that the crawler already
  only ingests license-tagged sources. Residual risk accepted per stakeholder intent.
- **A rater opens the deck expecting only "real" scores** → the label + attribution set
  expectations that pending cards are candidates.
- **Rating a score that is later rejected** → rating rows persist harmlessly (the score
  simply never returns to the deck); aggregates on rejected scores are never surfaced.
- **Preview-bytes RPC abused to enumerate pending content** → it still requires
  authentication and returns bytes only for a real catalog id in `pending`/`accepted`;
  it is the same data the deck already lists, not a new disclosure surface beyond preview.

## Migration Plan

1. Backend: widen `rating_deck` sourcing + `submit_rating` gate; add the rating-preview
   byte path (module + gRPC + proto). All additive; accepted-only gates elsewhere unchanged.
2. App: point the in-card preview at the rating-preview fetch; add the "potential new
   score" label on pending cards. No player/save changes.
3. No data migration. Rollback = revert; existing ratings and `moderation_status` are
   untouched.

## Open Questions

- **Extract-only preview for pending** (first N measures instead of the whole score):
  deferred to a follow-up if licensing review later wants tighter exposure; D3's path is
  shaped so an extract could replace the full-bytes response without an API change.
- **Explicit pending-first ranking in the queue** (weighting community score of pending
  candidates): can extend the merged review-priority sort later; out of scope here.
