## Why

The scoring engine already produces, per run, a **tempo** sub-score (free play) and a
**reaction** sub-score (Wait Mode) — `performance-scoring` explicitly says a run can "feed
the reaction leaderboard, the tempo leaderboard, or both", and #5 already uploads and stores
the full session-result record. The pieces are in place; what's missing is the competitive,
community-growing payoff: **leaderboards**. This change ranks players per piece so they can
compete and compare — reusing #5's ingested data with no new plumbing.

## What Changes

- **Per-piece, per-mode leaderboards** — for each **validated (`accepted`) catalog score**,
  two rankings: a **tempo** board (from the free-run sub-score) and a **reaction** board
  (from the Wait-Mode sub-score). Ranked by the per-mode synchronization sub-score, with a
  defined tie-break (the mode's timing metric, then earliest achieved).
- **Personal best per (player, piece, mode)** — a player's leaderboard entry is their
  **best** result on that piece in that mode; it updates **monotonically** as better sessions
  are ingested (never regresses). Derived from #5's `play_sessions` — no new ingestion path.
- **Privacy-respecting visibility (reuses #5)** — a player appears on a **public** leaderboard
  only if their profile is **public and age-eligible** (the #5 opt-in + minimum-age safeguard).
  A private/ineligible player is **not listed to others**, but always sees **their own
  personal best and their own rank** among public players. Leaderboards never expose a
  sensitive field.
- **Basic integrity checks** — server-side sanity checks on submitted scores (in range,
  consistent with the piece's onset counts) to reject obviously impossible results; robust
  anti-cheat is acknowledged as a limitation (client-authoritative scoring) and deferred.
- **App leaderboard views** — from a score (and the profile), view its tempo/reaction boards,
  your rank and personal best, and top players.

Out of scope: a global cross-piece ranking / overall "best players" board (a possible future
extension); friends/social-graph filtering; prizes; robust anti-cheat.

## Capabilities

### New Capabilities
- `leaderboards`: the ranking model — per-piece, per-mode (tempo/reaction) boards ranked by
  the per-mode synchronization sub-score with a defined tie-break; the monotonic
  personal-best per (player, piece, mode) derived from ingested sessions; the rule that only
  public, age-eligible players are listed to others while a player always sees their own best
  and rank; and the basic integrity checks on submitted scores.
- `leaderboard-views`: the app surfaces for viewing leaderboards — a piece's tempo/reaction
  boards, the viewer's own rank and personal best, and top players, reachable from the score
  and the profile.

### Modified Capabilities
<!-- None. Leaderboards read #5's play data and reuse #5's public-profile visibility/eligibility
     as an authorization input, expressed additively in `leaderboards`. -->

## Impact

- **Depends on #5** (`play_sessions` + the per-mode sub-scores it stores; the public-profile
  visibility + age-eligibility gate) and #4 (the profile entry point). Relates to
  `performance-scoring` (the sub-scores and run classification) and `catalog-search`/
  `score-hub` (pieces are `accepted` catalog scores).
- **Backend**: a personal-best store per (user, piece, mode) maintained monotonically on
  session ingest (hooking #5's `RecordPlaySession`); leaderboard read RPCs (piece + mode →
  ranked public entries + the caller's own rank/best); visibility/eligibility gating; integrity
  checks. No change to the ingestion transport.
- **App** (`apps/music`): leaderboard view(s) reachable from a score and the profile; the
  viewer's rank/PB display.
- **Privacy**: public listing strictly follows #5's opt-in + minimum-age safeguard; own data
  always visible to the owner; no sensitive fields exposed.
- **Coverage**: Rust ≥ 80% for PB monotonicity, ranking/tie-break, visibility gating, integrity;
  Flutter ≥ 80% for the views via fakes.
