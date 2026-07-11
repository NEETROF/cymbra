## Why

Today the piano player teaches notes (Wait Mode gates on the right key) but never
tells the player *how well* they played: nothing measures whether a note landed on
the beat, whether it was held long enough, or whether the tempo was respected. There
is no reward loop. Adding a live "synchronization" score plus tasteful Guitar-Hero–style
feedback turns practice into a game the player wants to repeat, while keeping the
learning surface (falling notes, keyboard highlights, Wait Mode) unchanged so the
gamification helps rather than distracts.

## What Changes

- Introduce a **performance-scoring engine** that, during free-running playback,
  evaluates each expected note against what the player actually played and derives a
  rolling **synchronization percentage (0–100%)** from three dimensions: timing
  (note attacked at the right moment vs. the beat), correctness (right pitch at the
  right onset), and sustain (note held roughly the intended duration).
- Add an on-screen **sync gauge** (a compact meter, always visible during a scored
  run) and **tiered visual feedback** that escalates at 20% thresholds (0–20 … 80–100),
  Guitar-Hero–style hit sparkles / combo streaks — deliberately restrained so they
  never occlude the falling notes or keyboard and never fire in a way that misleads
  a learner.
- Wire this feedback into **every render mode** — Synthesia (waterfall), the horizontal
  scrolling staff, and the engraved vertical Partition — reusing their existing painters
  (the keyboard-anchored hit sparks show where a keyboard is present; the gauge shows in
  all modes). Switching modes mid-run keeps the run.
- On song end, show a **session-summary modal**: overall sync %, per-dimension
  breakdown (timing / correct notes / sustain), best combo, and a note-accuracy count.
- From that modal, let the player **replay the run on the horizontal score**, with the
  notes they missed / mistimed **highlighted** so they can see exactly what went wrong.
- Persist the **last session summary locally** so it survives the modal being closed
  (device-local only in this change).
- Add a small set of **localized strings** (en/fr/it/es) for the new UI, following the
  existing l10n workflow.

Scoring runs in **both** playing modes and in **both** Wait Mode on and off. The timing
dimension adapts to the mode: with Wait Mode **off** it measures the signed offset of each
attack against the note's scheduled onset (respect of tempo); with Wait Mode **on** — where
the cascade freezes at each onset and absolute-tempo offset is meaningless — it measures
**reaction time** from the moment the gate opens on an onset to the correct attack.
Correctness (wrong/extra presses) and sustain (hold vs. intended duration) are judged the
same way in both modes. Scoring is active in **all three** render modes (Synthesia, the
scrolling staff, and the engraved Partition). Wait Mode gating itself is unchanged —
scoring is layered on top.

Because Wait Mode can be toggled at any moment, each judged onset is **stamped with the mode
active at that instant** (not the run as a whole); a mid-run toggle never resets the run. At
song end the run is classified `free` / `wait` / `mixed`, and — since reaction and tempo
measure different things — it carries a **separate synchronization sub-score per mode** over
that mode's onsets. This feeds **two distinct leaderboards** (a tempo board for free-run, a
reaction board for Wait Mode): a pure run scores on one board, a mixed run yields a partial
sub-score for each, and the deferred server change decides whether to rank mixed runs by
dominant mode or exclude them — the client records what both policies need.

### Deferred to a follow-up change (explicitly out of scope here)

- Uploading scores/metrics to the server tied to the signed-in user's profile, with
  **local-first** capture when offline and **reconnect/retry** sync when the server
  returns, **routing each run to the tempo and/or reaction leaderboard by its mode
  classification** (and the mixed-run policy: dominant-mode vs. excluded). (Local capture
  and the per-mode sub-scores land here; the transport/queue and leaderboard routing do not.)
- A **user profile page** listing played pieces grouped by instrument, each with its
  session summary — this depends on the server sync above.

These are noted so the local data shape produced here is forward-compatible.

## Capabilities

### New Capabilities
- `performance-scoring`: Real-time evaluation of a live performance against the loaded
  score — per-note judgment (timing / correctness / sustain), the derived rolling
  synchronization percentage, and the immutable per-session result record that feeds
  the summary and (later) server sync.
- `gamified-feedback`: The on-screen sync gauge and the tasteful, tiered
  (per-20%) Guitar-Hero–style visual feedback shown in the Synthesia and horizontal
  scrolling-staff modes, constrained so it never disrupts learning.
- `session-summary`: The end-of-song summary modal (aggregate metrics + per-dimension
  breakdown), local persistence of the last summary, and the horizontal-score replay
  that highlights the player's mistakes.

### Modified Capabilities
<!-- None: scoring is additive and runs alongside Wait Mode / playback without changing
     their requirements. -->

## Impact

- **Flutter app** (`apps/music/lib`):
  - New scoring state (Riverpod notifier + Freezed models) fed by the player's
    note-on/note-off stream and the playhead; new `performance_scoring.dart` /
    `session_summary.dart` state modules (pure, host-testable logic in a `*_core`-style
    seam per the coverage rules).
  - `player_notifier.dart` / `player_data.dart`: emit the timing signals scoring needs
    (onset crossings, note-off with timestamps) without altering Wait Mode behavior.
  - `synthesia_painter.dart` and the scrolling-staff painter: additive hit/feedback
    layers; a new gauge widget; a summary modal widget; a replay overlay on the
    horizontal score.
  - New localized strings (en/fr/it/es) via the existing `l10n` flow.
- **No Rust engine changes** expected (scoring is Dart-side, driven by existing note
  timing); no new native FFI surface.
- **Dependencies**: none new anticipated (reuse `shared_preferences` via
  `PreferencesService` for the local summary).
- **Forward-compat**: the session-result record is designed to serialize cleanly for the
  deferred server-sync change.
