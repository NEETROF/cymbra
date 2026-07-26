## Context

Change #4 of the moderation/curation initiative. It sits on top of #2 (app ratings:
`score_ratings`, one upserted rating per user+score, the `needs_review` aggregate flag)
and #3 (moderators accept/reject in the BO; the BO exists to host UI). The goal is to
make users **rate broadly and honestly** through rewards, without letting rewards spawn
spam that pollutes the rating signal.

Key product decisions already locked in discussion:
- Rating is **app-only**; the BO has **no** rating action.
- **The rater's surface, not their role, decides**: app ratings count and earn points for
  everyone including staff (bootstrap: the owner plays too). BO moderation work earns
  nothing.
- Reward **coverage + honesty**, never raw volume.
- No auto-promotion to moderator; instead a **reliability indicator** informs manual calls.

## Goals / Non-Goals

**Goals:**
- Incentivize catalog **coverage** and **honest** ratings with a two-part points economy.
- A single **points/XP backbone** driving levels that unlock pianos and badges.
- Let staff earn points for app ratings; earn nothing for BO moderation.
- Give the BO a per-user **reliability indicator** for manual promotion.
- Make gaming unrewarding by construction (engagement gate, diminishing coverage, honesty
  settled against ground truth, no self-settlement).

**Non-Goals:**
- Building temporary-premium mechanics (declared as a future tier only).
- Leaderboards / social features; deep fraud analytics; changing the #2 rating operation.

## Decisions

### D1 — Two-part points: coverage (immediate) + honesty (deferred)
- **Coverage points** are awarded synchronously when a rating is recorded, sized by a
  **diminishing** function of the score's existing rating count (much for an under-rated
  score, ~0 for a well-covered one), subject to a **daily cap** per user, and only if the
  **engagement gate** passed (the user opened/previewed the score before rating).
- **Honesty bonus** is awarded later at **settlement** (D2), sized by alignment with the
  ground truth: full when aligned, a **small floor when misaligned, never negative**.
- **Why split**: coverage gives immediate feedback and drives breadth; honesty needs a
  ground truth that only exists later. Splitting lets solo/bootstrap users earn coverage
  now while honesty accrues once a community exists.

### D2 — Settlement: community consensus and/or moderator decision, never self
A score's ratings become settleable when a **ground truth** appears:
- **Community consensus**: the score reaches a settlement threshold (a minimum number of
  distinct raters, ≥ the re-review minimum) and its aggregate is taken as the truth.
- **Moderator decision**: a moderator's accept/reject (initial or re-review) is a truth
  for that score, **weighted above** consensus when both exist.
Each rating is settled (frozen) against the applicable truth; a later moderator decision
MAY re-settle to supersede a consensus-only settlement. **A rating is never settled
against a moderation decision made by that same user** (no self-settlement) — the rating
still counts in the aggregate and keeps its coverage points; only its honesty bonus waits
for an independent truth.
- **Why**: rewards honest blind rating (the truth forms after the rating), keeps the
  expert (moderator) authoritative, and removes the staff self-farm loop while preserving
  "staff can play and earn".
- **Alternative**: settle purely on moderator decisions. Rejected — most scores are never
  moderated, so coverage would go largely unrewarded on the honesty axis.

### D3 — Points ledger is append-only; balance/level derived
Award events (coverage award, honesty award, adjustments) are appended to a
`curation_points` ledger; a user's balance and level are derived (optionally denormalized
for cheap reads). Settlement state per (score) and per (rating) is tracked so honesty is
awarded **once**.
- **Why append-only**: auditable, replayable, and safe under the at-least-once job model
  if settlement runs as a worker task.
- **Where settlement runs**: on rating for coverage (inline); for honesty, either inline
  when a moderator decides, or via a worker sweep when a score crosses the consensus
  threshold (reuses the `cymbra-worker` sqlxmq infra; idempotent by settlement state).

### D4 — Engagement gate as a reward-eligibility rule (no #2 change)
"Rate only after previewing" is enforced as a **reward-eligibility** condition, not a
change to the #2 rating operation: a rating recorded without a preceding preview still
records, but earns **no coverage points**. The app naturally routes rating through the
card (which offers the preview), so this is mostly a backstop.
- **Why**: keeps #2's spec untouched (it is not yet archived) and puts the anti-blind-swipe
  rule where it belongs — in the reward logic.

### D5 — Points backbone → levels → unlocks
Points are the XP. **Levels** are point thresholds. Unlocks hang off levels:
- **Pianos/SoundFonts** unlocked at tiers, feeding the existing `piano-sound-selection`
  catalog (FreePats CC0 → zero marginal cost).
- **Badges** at milestones (first ratings, N aligned ratings, rare-score coverage…).
- **Temporary premium** declared as a **future** tier; not implemented now.
- **Why one backbone**: avoids parallel competing currencies; every reward is "spend a
  level/threshold", simple to reason about and extend.

### D6 — Staff inclusion by surface, not role
Award logic keys off **where** the rating came from (the app rating path), not the rater's
role. BO endpoints (validate/reject/sort) never award points. The only role-aware rule is
the no-self-settlement guard in D2.
- **Why**: matches the decision and the bootstrap need; simplest correct rule.

### D7 — Curator reliability indicator (BO), read-only
A per-user BO panel shows: total ratings, coverage contribution (how many under-covered
scores they rated), and **alignment rate** (share of *settled* ratings that matched the
truth). Read-only; informs manual promotion; no automation.
- **Why alignment rate over raw counts**: raw volume says nothing about trust; alignment
  is the honesty proxy an admin actually needs before granting `moderator`.

## Risks / Trade-offs

- **Reward-driven spam** → engagement gate + diminishing coverage + daily cap make volume
  farming pay ~nothing; honesty settles against ground truth so random ratings earn only
  the floor. [Residual gaming] → tune caps/curve in config; deeper fraud analytics later.
- **Herd bias from consensus settlement** → weight moderator (expert) above crowd; keep
  honesty bonus modest; never punish (floor, never negative) so minority taste survives.
- **Staff self-farm** → no-self-settlement guard (D2); BO work earns nothing (D6). Solo
  bootstrap still earns coverage, honesty defers to a real community — acceptable/intended.
- **Settlement complexity/idempotency** → append-only ledger + per-rating settlement state;
  run honesty settlement idempotently (inline on moderator decision, or worker sweep on
  consensus threshold). [Double-award] → guarded by settlement state, at-least-once safe.
- **Coverage** → Rust ≥ 80% for award/settlement; test diminishing curve, cap, gate,
  alignment, no-self-settlement, floor-not-negative, award-once.

## Migration Plan

1. Add ledger + settlement-state + level/unlock/badge tables (additive). Config for values.
2. Hook coverage award into the rating path; hook honesty settlement into the moderator
   decision path (#3) and/or a worker sweep on the consensus threshold.
3. App: surface points/level/next-unlock + badges; wire unlocked pianos into
   `piano-sound-selection`.
4. BO: add the read-only reliability panel.
5. **Rollback**: rewards are additive; disabling the award hooks and hiding the app/BO
   surfaces leaves ratings and moderation fully functional. Ledger data is inert if unused.

## Open Questions

- **Point values / thresholds / diminishing curve / daily cap** — ship config defaults and
  tune with real usage. Initial straw-man to confirm at implementation.
- **Consensus settlement trigger** — inline vs. worker sweep, and the exact minimum-rater
  threshold (≥ the re-review N=5). Lean worker sweep for scale.
- **Re-settlement policy** — does a late moderator decision override an earlier
  consensus-settled honesty award (adjust the ledger), or only settle unsettled ratings?
  Lean: moderator can adjust once, appended as a correcting ledger event.
- **Temporary premium** — mechanics deferred; confirm it stays a declared-future tier.
