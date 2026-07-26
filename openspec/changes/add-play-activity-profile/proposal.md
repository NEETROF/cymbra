## Why

To grow the community we want play activity to be **visible and social**: a GitHub-style
contribution grid on the profile showing how much a player plays and how well, and
**public profiles** other players can view. That requires end-of-session play stats to be
**persisted server-side** — today they exist only locally in the session summary. Because
losing a player's stats would break the grid and feel unfair, the delivery of these stats
MUST be **reliable: nothing is lost**, even if the server is momentarily unable to record
them.

## What Changes

- **End-of-session stats sent to the server (reliably, no loss)**
  - At session end the app captures the session's play stats (score, timestamp, success
    rate, and the session summary metrics) and writes them to a **durable local outbox**
    that survives app restarts.
  - A sender **retries with backoff until the server acknowledges**; entries are removed
    only after acknowledgement. Server ingestion is **idempotent** (keyed by a
    client-generated session id), so retries never double-count and no session is lost.
  - The server persists per-session records and aggregates them **per day** (play count +
    success rate).
- **GitHub-style play heatmap on the profile** — a grid with one cell per day; each cell is
  **colored by that day's success rate** (percentage of success), with the number of songs
  played conveyed by the cell (size/intensity + tooltip). Extends the full-screen curator
  profile from #4.
- **Public player profiles (opt-in, private by default)** — a player's profile is
  **viewable by other authenticated players** only after the user **explicitly opts in**,
  showing a defined **public field set** (handle/display name, level, badges, the play
  heatmap, songs-played totals) and **excluding sensitive fields** (email, and the
  moderation/curator reliability figures). Going public requires meeting a **configured
  minimum age** (`min_public_sharing_age`, default **16** — the strictest EU threshold, so
  compliant EU-wide without per-country detection): a **neutral age gate at opt-in** stores
  only a derived eligibility date (no date of birth kept), and the safeguard is
  **server-enforced, fail-closed**, as a UTC date check with a one-day margin.

Out of scope: social discovery (search/follow/leaderboards) beyond viewing a profile;
changing how success is scored (owned by `performance-scoring`); push notifications.

## Capabilities

### New Capabilities
- `play-activity-sync`: reliable, idempotent delivery of end-of-session play stats — the
  durable local outbox (survives restart), retry-until-acknowledged, idempotent server
  ingestion keyed by a client session id (no loss, no double-count), and server-side
  per-session persistence + per-day aggregation (count + success rate).
- `play-activity-heatmap`: the GitHub-style daily contribution grid on the profile, colored
  by each day's success rate and conveying the day's play count.
- `public-player-profile`: viewing another player's profile — the public field set, the
  exclusion of sensitive fields, and the per-user visibility control.

### Modified Capabilities
<!-- None. Capturing stats at session end is expressed as a new requirement in
     `play-activity-sync` rather than changing the `session-summary` requirements. -->

## Impact

- **App** (`apps/music`): a durable outbox (persisted store surviving restarts) + a
  background sender with backoff; hook at session end to enqueue the summary; the heatmap
  widget on the #4 profile; a read-only public-profile view; a visibility setting.
- **Backend**: an idempotent `RecordPlaySession` ingestion RPC keyed by client session id;
  a `play_sessions` table + per-day aggregate (view or denormalized); a public-profile read
  RPC returning only the public field set; a visibility flag on the user.
- **Depends on #4** (the curator profile the heatmap and public view extend). Relates to
  `session-summary` / `performance-scoring` (source of the session metrics and success rate)
  and `user-account` (handle/display name, the visibility setting).
- **Privacy/RGPD**: public profiles expose a limited field set only; sensitive and
  moderation fields are never public; users can opt out. See design Open Questions.
- **Coverage**: Rust ≥ 80% for ingestion/idempotency/aggregation and the public read;
  Flutter ≥ 80% for the outbox/sender (retry, restart-durability, no-loss) and the heatmap
  via fakes.
