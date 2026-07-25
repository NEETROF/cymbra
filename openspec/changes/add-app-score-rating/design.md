## Context

This is change #2 of 3, building on the moderation gating from #1
(`add-score-moderation-gating`). #1 hides non-`accepted` scores from normal callers;
therefore the rating deck can only ever show **validated** scores to users, and
ratings are a signal *about validated scores*, not a crowd-validation mechanism for
`pending` ones. The chosen model is **hybrid**: ratings prioritize/flag work for
moderators, who decide (the accept/reject action and its queue live in #3).

App stack is Riverpod 2 + Freezed (codegen); the hub is a card grid over
`catalogSearchProvider` → `catalogService.search` → gRPC `ScoreServiceClient`. There is
no swipe/rating UI or dependency today; `ScoreCard` is reusable; the player render
engine exists (horizontal game score). Backend is gRPC-only (tonic).

## Goals / Non-Goals

**Goals:**
- One updatable rating per user per validated score, combining a swipe verdict and an
  optional 1–5 star value.
- A fun, discoverable, accessible deck: swipe with mirrored tap buttons, read-only
  in-card preview playback, first-run coaching.
- A per-score aggregate usable for hub ranking/reco and for the hybrid re-review flag.

**Non-Goals:**
- Changing a score's moderation status from ratings (moderators decide — #3).
- Rating `pending`/`rejected` scores (invisible to normal users by #1).
- The moderator queue UI / accept-reject action (#3); leaderboards or social graph.

## Decisions

### D1 — One rating row per (user, score), swipe + stars unified
Table `music.score_ratings (user_id, catalog_score_id, verdict, stars, updated_at,
PRIMARY KEY (user_id, catalog_score_id))`. `verdict` ∈ `dislike|like|love` (the swipe:
**left = dislike**, **right = like**, **up = love**); `stars` ∈ 1..5 nullable. Re-rating
**upserts** the row (`ON CONFLICT ... DO UPDATE`).
- **Why unify**: swipe and stars are two granularities of the same "how much do you
  like this" signal. Storing both on one row avoids double-counting and lets the UI
  map a swipe to an implied star band (e.g. love≈5, like≈4, dislike≈1–2) or keep
  them independent — resolved in D2.
- **Skip without judging**: because `left = dislike` is a negative verdict (Tinder-style
  "nope"), a separate, non-recording **Skip** control lets a user advance a card they
  don't want to judge without writing any rating.
- **Alternative**: an append-only ratings event log. Rejected for #2 — a single
  current rating per user is enough for the aggregate; an event log can be added later
  if we need history/anti-gaming analytics.

### D2 — Swipe ↔ stars mapping
A swipe records a `verdict` and a coarse implied score; opening the card and setting
stars records an explicit `stars` and a derived `verdict` (5→love, 3–4→like, 1–2→dislike)
so the two never conflict. The aggregate (D3) is computed from an effective numeric
value per rating on the 1–5 scale: explicit `stars` when present, else the verdict's
implied value (dislike≈1.5, like≈3.5, love≈5). A `dislike` therefore pulls the aggregate
down — that is the negative signal the re-review flag (D4) needs.
- **Why**: keeps a single comparable scale for ranking while letting users act fast
  (swipe) or precisely (stars).

### D3 — Aggregate: average + count, read on demand (cache later)
Per score: `avg_effective`, `rating_count`, and a `love/like/pass` breakdown. For #2,
compute on demand via a grouped query (indexed by `catalog_score_id`), optionally
denormalized onto `catalog_scores` (e.g. `rating_avg`, `rating_count`) updated on write
if the hub needs to sort by it cheaply.
- **Why on-demand first**: avoids premature denormalization; the corpus/rating volume
  is small (<50 users for 6 months, per the prod plan). Add a denormalized column or
  materialized view when ranking-by-rating becomes a hot path.

### D4 — Hybrid re-review flag, not a status change
When a validated score reaches `rating_count ≥ N` and `avg_effective ≤ T` (tunable
config constants; **defaults N = 5 ratings, T = 2.0 on the 1–5 scale**), the backend
marks it **eligible for re-review** — a boolean/`needs_review` signal (a column or a
derived query) consumed by #3's queue. It **never** sets
`moderation_status`; a moderator re-opens and decides. Symmetrically, strong positive
ratings can raise a score's rank but never auto-accept anything (nothing is auto-set).
- **Why**: preserves #1's invariant that only a human validates, while making ratings
  actionable. Thresholds live in config so they can be tuned without a schema change.
- **Alternative**: auto-demote low-rated scores to `pending`. Rejected — that hides a
  score from everyone on crowd pressure alone, gameable and against "moderator decides".

### D5 — Read-only preview reuses the player render, non-interactive
The in-card Play previews the score in the existing horizontal game-score render, with
all user interaction (input judging, hand selection edits, wait-mode, scoring) disabled
— a "playback only" mode: notation scrolls and notes sound, nothing is scored.
- **Why**: matches the request exactly ("même mode que la partition de jeu … sans
  interaction utilisateur"). Implemented by driving the existing render/playback with a
  read-only flag rather than a second renderer.
- **Note**: keep the native render behind the existing injectable seam so the deck +
  notifier stay testable without the native lib.

### D6 — Swipe implementation: prefer a vetted package, fall back to custom
Evaluate a maintained Flutter card-swiper (license + upkeep) vs. a custom
`GestureDetector`/`AnimatedBuilder` stack. Either way the card visual reuses
`ScoreCard`, and swipe actions are duplicated as on-screen buttons (accessibility).
- **Why the fallback**: the app currently has zero swipe deps; if no package clears the
  license/maintenance bar, a bounded custom stack (drag + fling + snap-back) is small.

## Risks / Trade-offs

- **Rating abuse / gaming** → thresholds (D4) require a minimum vote count; #2 stores
  one rating per user (upsert), limiting trivial inflation. Deeper anti-gaming (rate
  limits, event log) deferred until real usage.
- **Deck sourcing** → the deck reuses catalog search (accepted-only); a user could
  "run out" of cards. Mitigation: page/re-query and prioritize un-rated scores; empty
  state when all rated.
- **Preview correctness** → driving the real render read-only risks accidental
  interaction/scoring. Mitigation: a single well-tested read-only flag gating all input
  paths; widget/unit tests assert no scoring events fire in preview.
- **Coverage** → Flutter ≥ 80% for notifier/deck via fakes; Rust ≥ 80% for the rating
  repo/aggregate. Native render preview excluded via the seam.

## Migration Plan

1. Add `music.score_ratings` (+ indexes) and, if chosen, denormalized aggregate columns
   on `catalog_scores`. Additive.
2. Add `SubmitScoreRating` RPC + repo + aggregate read; wire the hybrid `needs_review`
   signal for #3 to consume.
3. Ship the app deck behind the existing hub entry (a new tab/entry point), reusing
   `ScoreCard` + read-only preview.
4. **Rollback**: the deck is additive UI; removing the screen and RPC leaves scores
   untouched. Dropping `score_ratings` loses only rating data.

## Resolved Questions

- **Ratings apply only to `accepted` scores (decided).** Per #1's visibility gate, normal
  users only ever see validated scores, so the deck sources and rates `accepted` scores
  only. Any future "trusted user pre-rates pending" flow would need its own authorization
  carve-out — out of scope here.
- **Thresholds `N`/`T` (decided).** Config, not schema. Defaults: **N = 5 ratings, T = 2.0
  on the 1–5 scale**; tune later without a migration.
- **Left-swipe semantics (decided).** **Left = dislike**, a negative verdict that counts
  toward the aggregate (Tinder-style "nope"), giving the re-review flag a natural negative
  signal. A separate non-recording **Skip** control covers "advance without judging".
