## Why

The points economy pays for **rating only**. Every ledger entry is a `coverage` award (you
rated an under-covered score), a `honesty` award (your rating matched the ground truth), a
moderator `adjustment`, or a `redeem`. Playing the piano earns nothing.

That has three consequences, all of them wrong for a piano app:

- **Levels measure curation, not musicianship.** Level and lifetime points are the app's one
  visible "how far have I come" number, and someone who practises daily for six months is
  still level 0.
- **The reward shop is gated behind moderation work.** SoundFonts cost points; points come
  from rating. A player who wants a better piano sound has to go and rate other people's
  scores to get it — an unrelated chore standing between them and their instrument.
- **The activity is already recorded and already trusted.** `music.play_sessions` carries a
  per-session `overall_sync_pct` that the leaderboards and the season boards rank on, and
  `music.practice_sessions` records scoreless measure-range work. Both feed the profile
  heatmap and (as of `add-achievement-badges`) the badge counters. Only the economy ignores
  them.

The reason it was built this way is defensible: `add-curation-rewards` set out to pay for a
*scarce* contribution — rating coverage — and paying for play looks like an invitation to
farm. That objection is real, and it is what this change has to answer, not wave away.

## What Changes

- Add **two award reasons** to the append-only ledger: a **performance** award for a scored
  run, and a **practice** award for showing up. Both are `curation_points` rows like every
  other award, so lifetime points, the spendable balance, levels and the activity feed all
  work unchanged.
- Make farming **structurally unprofitable**, mirroring the mechanisms coverage already
  proved rather than inventing new ones:
  - **A quality gate** — a run below an accuracy floor earns nothing, so mashing keys or
    abandoning a piece pays zero.
  - **Diminishing returns per piece** — the first good run of a piece pays in full and each
    further one pays less, approaching zero. Replaying one easy piece all evening is worth
    barely more than playing it once. This is the coverage curve, re-aimed from "how many
    ratings does this score have" to "how many times have you already been paid for this
    piece".
  - **Difficulty weighting** — an award scales with the piece's catalog level, reusing the
    weights the global leaderboard already ranks with rather than a second scale.
  - **A per-day cap** — a ceiling on what play can pay in one day, like the coverage cap.
- **Practice counts as showing up, once a day.** A scoreless measure-range run earns a small
  fixed award, at most **once per local day** — not per session and not per lap. Sitting
  down to drill a passage is worth acknowledging; drilling it forty times is not worth forty
  awards. This is the same judgement `add-achievement-badges` made for the consistency
  badges.
- **Award on the ingest path, idempotently.** Play and practice ingest are at-least-once by
  design (the client retries from a durable outbox until acked), so each award carries the
  event's own idempotency key and a retried session can never pay twice.
- **Feedback where the work happened** — the session summary shows the "+N" cue the rating
  action already shows, so the player learns what playing is worth without opening their
  profile.

**Products affected: Cymbra Music only.** It **consumes** the existing platform and ID
socle unchanged (no new auth, jobs, flags or DB access patterns) and **consumes** the
existing `reward-unlocks` machinery — ledger, balances, levels, shop, activity feed — which
this change feeds rather than reshapes. What is **new** is the earning rules for play. There
is **no back-office surface** and **no Cymbra ID or Live impact**.

## Capabilities

### New Capabilities
- `music-play-rewards`: how playing and practising earn points — the quality gate, the
  per-piece diminishing curve, difficulty weighting, the daily cap, the once-a-day practice
  award, and the idempotency each award is keyed on. The counterpart to `curation-rewards`,
  which owns the same rules for rating.

### Modified Capabilities
- `reward-unlocks`: the points feedback requirement stops being rating-only — a scored run
  that earns points shows its "+N" cue on the session summary, the same way a rating does.
  Balances, levels, the shop and the activity feed are unchanged; they simply have more than
  one source of income.

## Impact

- **Backend (`cymbra-music`)**: a new host-testable `play_rewards_core.rs` (the award curve,
  the gate, the weighting, the cap) alongside the existing `curation_rewards_core.rs`;
  `CurationRewardsSink` grows the two award entry points, which `PlayGrpc` already composes
  with at the ingest seam (it records the coverage engagement signal there today, and was
  written so play ingest itself stays ignorant of rewards); `CurationRewardsRepo` grows the
  reads the anti-grind rules need (how many times this piece has already paid, what play has
  paid today).
- **Database (`music` schema)**: the `award_kind` CHECK gains the two new kinds, and the
  ledger gains one nullable idempotency-key column with a partial unique index — the same
  shape as the existing `curation_points_coverage_once_idx`. Additive; existing rows and
  reads are untouched.
- **API (`score.proto` / `play.proto`)**: `RecordPlaySession` and `RecordPractice` return
  the points awarded, so the app can show the cue without a second round trip. Additive
  fields on existing responses; no new RPC.
- **App (`apps/music`)**: the session summary renders the "+N" cue; the existing reward
  celebration path is reused for a level-up triggered by play.
- **Out of scope (follow-up)**: a bonus for raising your own personal best on a piece (the
  purest "you got better" signal, and the one that needs the most tuning); retuning the level
  thresholds now that the economy has a second income stream; and any award for the
  leaderboard placements themselves.
