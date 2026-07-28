## Why

After moderation gating (#1) the hub shows only validated scores, but there is no
community signal on how good a validated score actually is, and no engaging way for a
user to express preference. We want a **Tinder-style swipe + star rating** on the app:
fun to use, it drives hub ranking/recommendations, and — per the chosen **hybrid**
model — it feeds the moderation pipeline by flagging poorly-rated scores for a
moderator to re-review (the moderator still makes the final call, in #3).

## What Changes

- **Swipe-to-rate deck (app)** — a card-stack surface over **validated (`accepted`)**
  catalog scores. Swipe **left = pass**, **right = like**, **up = love**; the same
  actions are available as **tap buttons under the card** so swiping is a shortcut,
  never the only path (accessibility + discoverability).
- **Star rating (app)** — tapping a card opens a 1–5 star rating for finer
  granularity. Stars and swipe map to one stored rating per user per score.
- **In-card preview** — each card shows the score's info (title, composer, level,
  attribution) and a **Play** control that previews the score in the **same horizontal
  game-score render mode as play**, but **read-only** (visualize the notation and hear
  the notes; no user interaction on the score being played).
- **Discoverability** — a one-time first-run coach-mark ("swipe or tap the stars") and
  a subtle first-card hint, persisted so it shows once.
- **Rating backend** — a new gRPC operation to submit/update a user's rating for a
  score, a `music.score_ratings` table (one row per user+score, updatable), and a
  per-score aggregate (average stars + counts).
- **Hybrid moderation signal** — when a validated score's aggregate falls below a
  threshold with enough votes, it is **flagged for moderator re-review** (surfaced in
  the #3 back-office queue). Ratings never change a score's moderation status directly.

Out of scope: the moderator re-review queue UI and the accept/reject action
(change #3); rating of `pending`/`rejected` scores (normal users cannot see them, by
#1); leaderboards/social features.

## Capabilities

### New Capabilities
- `score-rating`: The backend rating model — a signed-in user submits/updates a rating
  (swipe verdict and/or 1–5 stars) for an `accepted` catalog score; one rating per
  user per score; per-score aggregation; and the hybrid re-review flag that marks a
  low-rated validated score for moderator attention without changing its status.
- `swipe-rating-deck`: The app-side swipe/star rating experience — the card stack over
  validated scores, per-card info + read-only preview playback, swipe and mirrored
  tap-button actions, star rating, and first-run coaching.

### Modified Capabilities
<!-- None — the deck sources accepted scores through the existing catalog search; no
     existing requirement's behavior changes. -->

## Impact

- **Backend**: new `SubmitScoreRating` RPC on the score service (proto + Rust);
  new `music.score_ratings` table + migration; aggregate read (either a column/materialized
  view on `catalog_scores` or an on-demand aggregate); the hybrid flag surfaced for #3.
- **App** (`apps/music`): new swipe-deck screen + Riverpod notifier/service seam
  (model on `CatalogSearch`/`catalogService`), reusing `ScoreCard` for the card visual
  and the existing player render engine in a **read-only preview mode**; a swipe
  interaction (custom `GestureDetector`/animation or a vetted card-swiper dependency);
  `shared_preferences` flag for the one-time coach-mark.
- **Dependencies**: possibly one Flutter card-swiper package (to be vetted for license
  + maintenance) or a custom-built stack; no backend deps beyond existing sqlx/tonic.
- **Coverage**: Rust ≥ 80% for the rating store/aggregate; Flutter ≥ 80% for the
  notifier/deck via fakes (native render behind the existing seam).
