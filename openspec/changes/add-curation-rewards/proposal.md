## Why

Ratings (#2) power the hub ranking and the moderation re-review signal, but only if
users actually rate — and rate **honestly** and **broadly**. This change adds a reward
system that incentivizes exactly that: **coverage** of the catalog and **honest**
ratings, without the classic trap where rewarding volume produces spam that pollutes the
very signal we want. Rewards are earned for rating **from the app** (anyone, including
staff acting as players), tie into unlockable content, and give the back office a
per-user reliability indicator to inform manual moderator promotion.

## What Changes

- **Two-part points economy**
  - **Coverage points** — awarded **immediately** when a user rates, **diminishing** for
    already-well-covered scores, **capped daily**, and **gated by engagement** (the score
    must have been opened/previewed first). Rewards breadth, not blind swiping.
  - **Honesty bonus** — awarded **later**, at **settlement**, when a ground truth exists:
    the **community consensus** for the score and/or a **moderator's accept/reject
    decision**. A rating that aligns with the truth earns the bonus; a misaligned rating
    earns a **small floor (never negative)**, so honest minority taste is never punished.
    A user **never settles their own rating against their own moderation decision**.
- **XP/points as the single backbone** — points are the XP; **levels/tiers** unlock:
  - **Pianos/SoundFonts** (reuses `piano-sound-selection` + FreePats CC0; zero cost),
  - **Badges** (milestones: coverage, aligned-ratings, rare-score explorer…),
  - **Temporary premium access** — **deferred** (declared as a future tier, not built now).
- **Staff are players too** — an admin/moderator who rates **from the app** earns points
  like anyone (the bootstrap reality: the owner wants to play and earn). Moderation work
  in the BO (validate/reject/sort) earns **no** points; rating is **app-only** (there is
  no rating action in the back office).
- **Curator reliability indicator (BO)** — a read-only per-user panel: number of ratings,
  coverage contribution, and **alignment rate** (share of settled ratings that matched the
  ground truth). Informs **manual** promotion (no auto-promotion, per #3).

Out of scope: temporary-premium mechanics (declared, not built); leaderboards/social;
anti-abuse beyond the stated gates (rate-limit tuning, fraud analytics) tuned later.

## Capabilities

### New Capabilities
- `curation-rewards`: the points economy and settlement — coverage points (immediate,
  diminishing, capped, engagement-gated), the honesty bonus (deferred settlement via
  community consensus and/or a moderator decision, never self-settled, floor-not-negative),
  the append-only points ledger, and the rule that app ratings earn points regardless of
  the rater's role while BO moderation work does not.
- `reward-unlocks`: levels/tiers derived from points and what they unlock — pianos and
  badges now, temporary premium declared as a future tier — plus surfacing a user's
  points/level/unlocks and progress in the app.
- `curator-reliability-indicator`: the back-office per-user reliability panel (ratings
  count, coverage contribution, alignment rate) shown to admins/moderators to inform
  manual promotion decisions.

### Modified Capabilities
<!-- None. The engagement gate and settlement are expressed as reward-eligibility rules
     in `curation-rewards` rather than changing the rating operation from #2. -->

## Impact

- **Depends on #2** (`score_ratings`, the app rating flow, the re-review flag) and **#3**
  (moderator accept/reject decisions for settlement; the BO to host the reliability panel).
- **Backend**: a points ledger table (append-only award events) + per-user balance/level
  and per-score settlement state; award logic on rating (coverage) and on settlement
  (honesty) via consensus/moderator hooks; a reliability read for the BO; unlock/badge
  state. Config for all point values, thresholds, diminishing curve, and daily cap.
- **App** (`apps/music`): surface points/level/next-unlock and badges; the engagement gate
  (rate only after previewing) as a reward-eligibility condition; unlocked pianos flow into
  `piano-sound-selection`.
- **Back office** (`bo.cymbra.app`): the curator-reliability panel per user.
- **Coverage**: Rust ≥ 80% for award/settlement logic; Flutter ≥ 80% for the app surfacing
  via fakes; the Vue panel under its own test setup.
