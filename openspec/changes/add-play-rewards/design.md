## Context

The points economy (`add-curation-rewards`, capabilities `curation-rewards` +
`reward-unlocks`) is fully built and live:

- **Ledger**: `music.curation_points` — append-only, `award_kind IN ('coverage','honesty',
  'adjustment','redeem')`, signed `amount`. Lifetime = sum of non-`redeem`; spendable = sum
  of all. Levels derive from lifetime and never fall.
- **Anti-farming, already proven**: coverage is sized by a diminishing curve over the score's
  existing rating count (`coverage_bands`), clamped by a per-user `daily_cap`, gated on an
  engagement signal, and awarded **once per (user, score)** through the partial unique index
  `curation_points_coverage_once_idx`.
- **Config, not migration**: every value lives in `RewardConfig`, retunable without DDL.

What playing already records:

- `music.play_sessions` (0010) — one row per **scored** run, `overall_sync_pct` 0..100,
  `played_at` + `tz_offset_minutes`, client-generated UUID v7 id, ingested
  `ON CONFLICT (id) DO NOTHING` because the app retries from a durable outbox.
- `music.practice_sessions` (0022) — one row per **scoreless** measure-range run. Structurally
  incapable of carrying a score: a selective run never arms the scorer.
- `catalog_scores.level` — `beginner`/`intermediate`/`advanced`, already weighted 1.0/1.5/2.0
  by `GlobalConfig::level_weights` for the global season boards.
- `PlayGrpc` already holds a `CurationRewardsSink` and calls it on ingest (the coverage
  engagement signal, `add-post-play-rating-prompt`). The seam was placed at the gRPC
  composition point precisely so `PlayModule` stays ignorant of rewards.

Constraints that shape this design:

- **At-least-once ingest.** A retried session must not pay twice. The `ON CONFLICT DO
  NOTHING` on `play_sessions` is not by itself an award guard: a crash between the row
  insert and the ledger append would leave the award unpaid, and the retry would find the
  row present.
- **Lifetime only ever rises.** Nothing here may claw back; a mis-tuned award can be lowered
  for the future but never un-awarded.
- Coverage gate ≥ 80% both ecosystems, so the award math must live in a pure host-testable
  module with I/O behind the existing repo trait.

## Goals / Non-Goals

**Goals:**
- Playing and practising earn points, so level and the shop measure time at the instrument
  and not only curation work.
- Farming is **structurally** unprofitable — not policed after the fact, but unrewarding by
  construction, the way coverage already is.
- Reuse the existing ledger, balances, levels, shop, activity feed and difficulty weights
  rather than building a parallel economy.
- Every award is idempotent against its triggering event, under at-least-once ingest.
- Amounts, curves, gates and caps are configuration, retunable without a migration.

**Non-Goals:**
- A personal-best bonus (raising your own record on a piece). It is the purest "you improved"
  signal and deserves its own change; it also needs the most tuning, and folding it in here
  would make the anti-grind story harder to reason about.
- Awarding for leaderboard placement or season standing — that is competition, already
  rewarded with rank.
- Retuning `level_thresholds` for the new income. Levels never fall, so nobody is harmed by
  waiting; see Open Questions.
- Any back-office surface, and any change to how rating earns.
- Paying for MIDI activity outside a recorded session — there is no record to key on.

## Decisions

### D1 — A new pure module, not more of `curation_rewards_core`

`play_rewards_core.rs` mirrors the existing split: the award curve, the quality gate, the
difficulty weighting and the cap, with no I/O. `curation_rewards_core.rs` keeps owning the
rating math.

*Alternative rejected — extend `RewardConfig` and `curation_rewards_core` in place*: the two
economies answer different questions ("was this contribution scarce" vs "was this practice
real"), and merging them makes every future tuning of one a regression risk for the other.
They meet at the ledger, which is the right seam.

### D2 — Four independent brakes, because any one alone is farmable

Each brake closes a hole the others leave open, which is why all four ship together:

| Brake | Closes |
|---|---|
| **Accuracy floor** — a run below it earns 0 | mashing keys, walking away mid-piece |
| **Diminishing per piece** — the Nth paid run of a piece pays less | replaying one easy piece all evening |
| **Difficulty weight** — scaled by catalog level | farming the shortest trivial piece |
| **Daily cap** — a ceiling on what play pays per day | sheer volume |

Dropping the floor makes an idle keyboard pay. Dropping the per-piece curve makes one
20-second piece an infinite well. Dropping the weight makes the easiest piece the optimal
one. Dropping the cap makes an all-nighter beat a month of practice. The combination means
the profitable strategy *is* the intended behaviour: play varied pieces, well, regularly.

The per-piece curve is `coverage_bands` re-aimed: the same shape of `(bound, points)` bands,
read against **how many times this piece has already paid the user** instead of how many
ratings a score has. Reusing the shape means reusing its tests and its intuition.

### D3 — Practice pays once per LOCAL day, a flat amount

A scoreless run has no quality signal by construction, so none of D2's first three brakes can
apply to it. The only honest thing it evidences is *you sat down today*. So it pays a small
flat amount, at most once per player-local day, whatever the number of sessions or laps.

Local day, not server day: the session carries the client's UTC offset, and every other
play-side surface — the heatmap, the streak badges — already buckets that way. A player must
not lose their daily award because midnight UTC falls mid-evening for them.

*Alternative rejected — pay per practice session*: a loop restarted forty times would pay
forty times, and the client records one session per stop, so the "session" boundary is
player-controlled. Per-day is the only boundary the player cannot manufacture.

*Alternative rejected — pay nothing for practice*: it would tell a player drilling a hard
passage for an hour that their work was worth nothing, which is exactly the message this
change exists to remove.

### D4 — Idempotency is a durable key on the ledger row, not "did the insert happen"

The ledger gains one nullable `award_key TEXT` with a partial unique index on
`(user_id, award_key) WHERE award_key IS NOT NULL` — the same shape as the existing
coverage-once index. A performance award keys on the **client session id** (already unique
and already the ingest's idempotency key); the practice award keys on the player's
**local day**.

This is what makes the award exactly-once under at-least-once ingest, independently of
whether the session row itself was newly inserted. A retry re-attempts the award; the index
turns it into a no-op.

*Alternative rejected — award only when `play_sessions` reports a fresh insert*: it makes the
award depend on the ordering of two unrelated writes. The window between them is small but
real, and the failure is silent and unrecoverable (the retry sees the row and skips).

*Alternative rejected — two typed columns (`play_session_id`, `practice_day`)*: one generic
key covers both and any future reason, at the cost of an opaque string — worth it for a
column whose only job is de-duplication.

### D5 — Award at the ingest seam, through the existing sink

`CurationRewardsSink` gains `award_performance(...)` and `award_practice(...)`; `PlayGrpc`
calls them where it already calls `record_engagement`. `PlayModule` stays unaware of rewards,
exactly as it is today.

The award is **best-effort with respect to the ack**: a failure to award must not fail the
ingest, because the client's outbox would then retry the whole session and the player would
see their practice "not saved". A lost award is a missing handful of points; a failed ack is
a user-visible bug.

*Alternative rejected — a worker job over unpaid sessions*: it needs a scan and a "paid"
marker, delays the "+N" the player expects at the end of their run, and buys nothing, since
the ingest is already the exact moment the evidence arrives.

### D6 — The wire carries the awarded amount back

`RecordPlaySessionResponse` and `RecordPracticeResponse` gain `points_awarded`. The app shows
the "+N" on the session summary from the ack it is already waiting for — no second read, and
the number shown is the number recorded.

*Alternative rejected — refetch the profile after a session*: an extra round trip on a hot
path, and it cannot attribute the delta to *this* run.

### D7 — Difficulty weight is read from the catalog, and an unknown level is neutral

The weight comes from the piece's `catalog_scores.level` through the same table
`GlobalConfig` uses. A user upload, an unleveled catalog row, or a level string this build
does not know weighs **1.0** — never zero. A missing catalog level is a metadata gap, not a
reason to tell a player their practice was worthless.

## Risks / Trade-offs

- **A tuning mistake inflates lifetime points permanently** (nothing claws back) → the four
  brakes are multiplicative and the starting amounts are deliberately small next to the
  curation curve; the ceiling a determined farmer can reach in a day is the daily cap, which
  is one number to lower.
- **Self-reported sessions.** The client computes `overall_sync_pct` and can lie. This is
  already true of the leaderboards, which rank on the same field, so this change adds no new
  trust surface — it only adds a reason to lie that did not exist before. Accepted here;
  server-side verification is a whole change of its own and would have to start with the
  boards.
- **A second reader of `curation_points` on a hot write path** → the anti-grind reads are
  user-scoped and index-backed (the existing `curation_points_user_idx`, plus the new partial
  unique index which also serves the per-piece count); measure before caching.
- **The daily cap's day boundary is server-local for coverage and player-local here.** A
  deliberate split: the practice award is about the player's day, and changing coverage's
  semantics is out of scope and would move an existing user-visible line. Documented rather
  than silently unified.
- **Practice is the easiest thing to fake** (open a range, play one note, stop) → it is
  capped at one small award per day, so the return on faking it is a few points for the
  trouble of opening the app, which is roughly what showing up is worth anyway.
- **Levels arrive faster than the thresholds were tuned for** → nobody loses a level and no
  existing user is affected retroactively; see Open Questions.

## Migration Plan

Additive only. One migration extends the `award_kind` CHECK with the two new kinds and adds
the nullable `award_key` column plus its partial unique index. Existing rows carry
`award_key IS NULL` and are untouched by the index; every existing read (lifetime, balance,
activity feed) is unaffected because it filters on `award_kind` or not at all.

Rollback is a backend revert: the rows the new kinds wrote stay in the ledger and keep
counting toward lifetime — which is correct, since points already earned are never taken
back — and the old code, which never filters award kinds *in*, sums them without knowing what
they are.

## Open Questions

- **The starting numbers.** The floor, the per-piece bands, the weights, the daily cap and the
  practice amount need a first pass calibrated against real `play_sessions` volume, which does
  not exist yet at scale. They are all `RewardConfig`, so a first pass can be wrong and
  cheaply corrected — downward for the future, never retroactively.
- **Whether `level_thresholds` should be retuned** once play is an income stream. Deliberately
  deferred: retuning upward would slow existing curators, and the honest input is a month of
  real data, not a guess made the day the feature ships.
