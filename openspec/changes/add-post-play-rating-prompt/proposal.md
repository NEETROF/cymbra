## Why

Today the only way to rate a catalog score is the swipe deck, reached from a
library banner — a context where the user has heard at most a short auto-preview
of a score they did not choose. The strongest rating signal in the app is wasted:
a user who just **played** a piece from end to end has an informed opinion and
nowhere to express it. Sourcing ratings from actual play also improves the
signal's quality (a played score is genuinely evaluated, not swiped on 8 seconds
of preview) and feeds the existing `needs_review` re-review flag and hub ranking
with better data.

## What Changes

- **Rate at the end of a run.** When a scored run reaches the end of the piece,
  the session-summary modal carries a compact rating affordance (1–5 stars, the
  verdict derived exactly as the deck derives it) for the score just played. It
  is an *addition* to the modal, never a fourth mandatory choice: rating does not
  dismiss the modal and skipping it costs nothing.
- **Rate on early exit.** Leaving the player before the end (back button) opens a
  compact, **non-blocking** bottom sheet offering the same rating. Any dismissal
  — rate, close, tap outside, back — leaves the player. The exit is never
  cancelled or confirmed.
- **One prompt per score, ever.** The suppression is **per catalog score**, not a
  global nag counter: a score is offered for rating at most once on the device.
  Rating it, or skipping the prompt, retires that score permanently from the
  prompt. There is no global dismissal count and no cross-score snooze — the
  mechanism is self-bounding because the user only sees prompts for scores they
  actually played.
- **Never prompt for a score already rated.** A new authenticated read exposes the
  caller's own rating of a given catalog score, so the prompt is suppressed for a
  score rated on any device (the deck already excludes rated scores; the player
  needs the same truth for a single score).
- **Playing counts as engagement for curation rewards.** Engagement is currently
  recorded only by the deck's preview fetch, so a rating submitted from the player
  would earn zero coverage points. Opening a catalog score in the player, and
  ingesting a play session for it, now record the same engagement signal — an
  offline-cached score that never hits the bytes endpoint is covered by the
  session ingest.
- **Gated to genuinely-played, rateable scores.** The prompt only appears for a
  signed-in user, on a public-catalog score (bundled and user-contributed scores
  are not rateable), after a configured minimum of actual playback.

Not in scope: changing what a rating *is* (verdict, stars, aggregate, re-review
flag are unchanged), the swipe deck, the library banner's own thresholds, and any
new reward kind.

## Capabilities

### New Capabilities
- `post-play-rating`: rating a catalog score from the player — the end-of-run
  affordance in the summary modal, the non-blocking early-exit sheet, the
  eligibility gate (signed in, catalog score, played enough, not already rated),
  and the per-score one-prompt-ever suppression.

### Modified Capabilities
- `score-rating`: adds an authenticated read returning the caller's own rating of
  a single catalog score (rated or not, with the recorded verdict/stars), so a
  client can suppress a prompt for an already-rated score without paging the deck
  source.
- `curation-rewards`: widens the coverage-eligibility engagement signal — playing
  a catalog score (opening it in the player, or the ingest of a play session for
  it) records engagement, alongside the existing deck preview.
- `session-summary`: the end-of-song summary modal additionally offers to rate the
  piece just played; the affordance never dismisses the modal and the existing
  explicit see-mistakes / retry / quit choice is unchanged.

## Impact

- **Proto / backend** (`backend/music/proto/score.proto`, `backend/music/src`):
  one new RPC on `ScoreService` for the caller's own rating of a score; a new
  repo read in `score_rating.rs`; engagement recording added to the player-open
  bytes path and the play-session ingest (`module.rs`, `play_module.rs`).
  Regenerating the Dart/TS gRPC stubs is required.
- **Flutter app** (`apps/music`): a new rating-prompt notifier + persisted
  per-score prompt memory (preferences seam), a shared rating-prompt widget used
  by both surfaces, changes to `session_summary_modal.dart` and the player's back
  affordance in `player_screen.dart`, and new localized strings in the four ARB
  files.
- **No schema migration** for the prompt itself (the rating and engagement tables
  already exist); no change to the rating aggregate, the `needs_review` flag, the
  deck source, or the moderation pipeline.
- **Guests and offline** are unaffected: no prompt without a signed-in identity,
  and an unknown already-rated state (offline) suppresses the prompt rather than
  risking a failing submission.
