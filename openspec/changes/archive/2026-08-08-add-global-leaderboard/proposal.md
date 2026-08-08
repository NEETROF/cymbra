## Why

Per-piece boards (#6) answer "who plays *this piece* best". The community also wants the
bigger question — **"who is the best player overall?"** — a single global ranking that is a
*destination*, not tied to one piece. The hard part is fair aggregation: a naive sum rewards
grinding many easy pieces over real skill. This change defines a **difficulty-weighted,
best-N, seasonal** global ranking that rewards playing **hard pieces well**, stays fresh via
seasons, and reuses everything already ingested (#5) and the #6 bests — no new play-data
plumbing.

## What Changes

- **Global season score** — per player and per mode, a score = the **difficulty-weighted sum
  of their best sub-scores over their best-N pieces this season** (config `N` and difficulty
  weights). Best-N caps volume grinding (beyond N pieces, only quality on hard pieces helps);
  difficulty weighting makes advanced pieces worth more.
- **Two global boards: tempo and reaction** — mirroring #6's two modes, ranked descending by
  the global season score, with a defined tie-break (more contributing pieces, then earliest
  reached). (An optional combined "overall" board is left as a future nicety.)
- **Seasons** — the global board runs in **seasons** (config length, default monthly, UTC
  boundaries). At season end the standings are **snapshotted** (a lightweight hall-of-fame /
  history) and a new season starts. Per-piece all-time bests (#6) are **not** reset; only the
  season accumulation rolls over, so newcomers can always compete.
- **Same privacy model as #6/#5** — **viewing is open** to any authenticated user (incl.
  under-16 / private); **being listed** requires the #5 opt-in + age-eligibility; a player
  always sees **their own global rank and score**. No sensitive fields exposed.
- **App surfaces (a destination)** — a dedicated **Community / Leaderboards** screen (global
  boards + season selector) and a **global-rank standing** on the profile.

Out of scope: prizes/rewards tied to global rank; a combined cross-mode "overall" board
(future); friends filtering; robust anti-cheat (inherited limitation from #6).

## Capabilities

### New Capabilities
- `global-leaderboard`: the global aggregation model — the difficulty-weighted best-N season
  score per (player, mode), the tempo and reaction global boards with tie-break, the seasonal
  windows with end-of-season snapshots, the monotonic per-season bests derived from ingested
  sessions, the same visibility/eligibility listing gate with own-rank-always, and open
  viewing.
- `global-leaderboard-views`: the app destination surfaces — a Community/Leaderboards screen
  (global boards + season selector) and the player's global-rank standing on the profile.

### Modified Capabilities
<!-- None. Reuses #5's play data + visibility gate and #6's per-piece bests/ingest hook;
     expressed additively in `global-leaderboard`. -->

## Impact

- **Depends on #5** (ingested sessions + per-mode sub-scores + visibility/eligibility gate),
  **#6** (the per-piece best maintenance/ingest hook and its viewing/listing split it mirrors),
  and **#4** (the profile). Uses catalog `level` (and optionally facets) for difficulty weight.
- **Backend**: a per-(user, season, piece, mode) season-best store maintained monotonically on
  ingest; a global-score computation (difficulty-weighted best-N) + ranked read RPC (mode +
  season → public ranked page + caller's own rank/score); season boundary + end-of-season
  snapshot; visibility/eligibility gating; erasure extends to the new rows.
- **App** (`apps/music`): a Community/Leaderboards destination screen + global-rank on the
  profile.
- **Privacy**: identical to #6 — open viewing, gated listing, own-rank always, no sensitive
  fields.
- **Coverage**: Rust ≥ 80% for the aggregation, best-N/difficulty weighting, season rollover,
  ranking/tie-break, visibility gating; Flutter ≥ 80% for the views via fakes.
