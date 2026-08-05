## Why

Today a run is all-or-nothing: the player always plays the **whole piece** from the top,
and that run is **scored** — graded, persisted, and uploaded to the leaderboards and the
activity heatmap. A learner drilling four hard measures has no way to isolate and repeat
just that passage, and no safe way to do it: naively playing a subrange would either upload
a catastrophic ~5% grade (every un-played note is marked `missed`) or, if scored over the
subrange, let anyone farm a 100% by looping the easiest bar. We want a **practice** path —
pick a measure range, loop it, ramp the tempo — that lives **beside** the scoring system
without corrupting it, while still counting as real practice on the player's profile.

## What Changes

- **Active measure range** — a run gains an optional measure range `[start, end]`. The whole
  piece (the default) is a **full run**, scored exactly as today. Any narrower range is a
  **selective run**.
- **Selective runs are practice — never scored** — a selective run does **not** begin a
  scored run: no sync% grade, no combo/leaderboard entry, no `SessionResult` upload. It plays
  (and can loop) the chosen measures with the normal audio/Wait-Mode/hand behavior, but the
  scoring engine stays idle. This is a scoping rule on `performance-scoring`, not new math.
- **Practice still counts as activity** — a completed practice session is recorded as a
  **scoreless activity event** and surfaces on the profile's daily-activity grid as a
  **practice count** for that day, distinct from scored plays (it carries no sync%, so it does
  not change the day's success color). Requires a scoreless activity record end-to-end
  (app outbox → gRPC → aggregation).
- **Loop the range** — a selective run can loop: a **loop count** (repeat N times, or
  infinite) and an optional **practice tempo ramp** (each lap increases the playback speed by
  a chosen step). Laps run continuously reusing the existing loop-wrap + silence-all seam; no
  per-lap modal.
- **Two ways to pick the range** — measure **steppers** in the setup sheet (works in all three
  render modes) **and** **tapping** the start/end measure directly on the engraved Partition
  view (new hit-testing over the measure rects).
- **Entry points** — the **pre-play setup modal** offers "full run" vs "selective (practice)"
  with the range picker; the **end-of-run summary** offers "practice this section" so a
  learner can drill what they just missed.
- **Saved per score** — the chosen range and loop settings **persist per musical score** and
  are pre-filled when the score is reopened.

Out of scope: scoring or ranking partial runs; a backend "practice leaderboard"; A/B markers
inside a measure; section names/presets beyond the single saved selection; automatic
detection of "hard" passages.

## Capabilities

### New Capabilities
- `measure-range-practice`: the practice primitive — an active measure range on a run; the
  rule that a narrower-than-whole range is an **unscored** selective run; looping the range
  with a loop count (finite N or infinite) and an optional per-lap tempo ramp; the two range
  pickers (setup-sheet steppers + tap-on-score in the Partition view); and per-score
  persistence of the range and loop settings.

### Modified Capabilities
- `performance-scoring`: a scored run begins **only for a full run** (the range is the whole
  piece). A selective run (a narrower range / explicit practice) SHALL NOT begin a scored run.
- `play-activity-sync`: capture and deliver a **scoreless practice-activity** record
  (session id, score, timestamp — no `SessionResult`/sync%) with the same durable, idempotent,
  no-loss guarantees, so a practice counts toward activity without polluting scoring stats.
- `play-activity-heatmap`: the daily grid conveys a day's **practice count** distinct from its
  scored-play count; because practice carries no sync%, it does not contribute to the day's
  success color, but its count is available (intensity/tooltip).
- `pre-play-setup`: the setup modal lets the user choose a **full run or a selective (practice)
  run**, and when selective, pick the measure range (and loop settings); the choice is applied
  to the session that starts.
- `session-summary`: the end-of-run summary offers a **"practice this section"** action that
  opens the range picker for a selective run, alongside the existing see-mistakes / retry /
  quit choices.

## Impact

- **App state** (`apps/music/lib/state/`): `PlayerData` gains range + loop fields;
  `Player.advance`/loop-wrap and the run-start guard (`_maybeStartRun`/`_atStart`) become
  range-aware; the scorer is gated off for selective runs; a scoreless practice-activity
  capture path is added next to `play_sync_notifier`/`play_sync_service`.
- **UI** (`apps/music/lib/screens/player_screen.dart`, widgets, `PartitionPainter`): setup
  sheet range picker + loop controls; per-measure hit-testing/gesture layer over the engraved
  score; summary-modal "practice this section" action; profile heatmap cell shows practice
  count.
- **Backend / contract (the one bridge-crossing part)**: a scoreless practice-activity ingest
  path — a proto/gRPC addition (a `recordPractice`-style RPC or a `kind=practice` variant) and
  server aggregation so `getPlayActivity` returns per-day practice counts. Idempotent by
  client session id, like scored sessions.
- **Persistence**: per-score practice settings stored via the local preferences/store seam.
- **No Rust engine/synth change** — playback timing stays Dart-driven; the synth is unaffected.
- **Tests**: pure range/loop logic and the scoring-gate rule are host-testable (Dart unit);
  new widget tests for the picker/gesture layer and heatmap practice count; backend tests for
  the scoreless ingest + aggregation. Coverage ≥ 80% both ecosystems.
