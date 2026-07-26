## Context

Change #7, the "destination" companion to #6's per-piece boards. It answers "who is the best
overall?" with a single global ranking. It reuses: #5's ingested sessions (per-mode
sub-scores, run classification) and its visibility/eligibility gate; #6's ingest hook and its
viewing-open / listing-gated split; #4's profile; and catalog `level` for difficulty. The
central design problem is **fair aggregation** — combining many per-piece results into one
number without rewarding volume-grinding over skill.

## Goals / Non-Goals

**Goals:**
- A global ranking that rewards **playing hard pieces well**, not sheer volume or farming easy
  pieces.
- Freshness and fairness over time via **seasons** (newcomers can compete; leaders aren't
  entrenched forever).
- Reuse ingested data + the #5/#6 privacy model exactly; no new play-data plumbing.

**Non-Goals:**
- Prizes tied to rank; a combined cross-mode "overall" board (future); friends filtering;
  robust anti-cheat (client-authoritative scoring remains a known limit).

## Decisions

### D1 — Aggregation: difficulty-weighted, best-N, seasonal (the core formula)
Per player and per mode, the **global season score** = the sum, over the player's **best-N
pieces this season**, of `(best season sub-score on that piece / 100) × difficulty_weight(piece)`.
- **Best-N** (config, e.g. 20): only a player's N strongest pieces count, so playing *more*
  pieces beyond N gives nothing — you climb by playing **harder pieces better**, not by volume.
- **Difficulty weight** (config): from catalog `level` (e.g. beginner ×1.0, intermediate ×1.5,
  advanced ×2.0; unleveled → a neutral default). Optionally refined by facets later. Makes hard
  pieces worth more, countering easy-farming.
- **Why this over alternatives**:
  - *Sum of all bests* → pure volume grinding. Rejected.
  - *Average of bests* → one perfect easy piece tops it; needs a min-pieces hack. Rejected.
  - *Placement/percentile points per board* → depends on who else played (empty boards = cheap
    points), and couples the global score to board populations. Rejected as the primary metric;
    best-N-quality is population-independent and harder to game.
  - *Elo/rating* → most principled but heavy (pairwise comparisons, cold-start); disproportionate
    now. Revisit if the community grows.
- All parameters are **config**, so the curve is tunable without schema change.

### D2 — Two global boards: tempo and reaction
Mirror #6's two modes: a **tempo** global board (from free-run season sub-scores) and a
**reaction** global board (from Wait-Mode season sub-scores). A `mixed` run contributes its two
sub-scores to the two season aggregates respectively.
- **Why two, not one**: the modes measure different skills (as in #6); a single blended global
  number would be less meaningful. A combined "overall" board can be added later as a simple
  weighted sum.

### D3 — Seasons: config window, UTC boundaries, end-of-season snapshot
The global board runs in **seasons** (config length, default ~monthly). Season boundaries are
**UTC** (global boards are shared and not per-user experiences, so a single UTC boundary is
simplest and consistent — contrast the heatmap's local-tz bucketing, which is per-user UX). At
season end the final standings are **snapshotted** into a lightweight history (hall of fame),
and a new season starts fresh. Per-piece **all-time bests (#6) are not reset**; only the
per-season aggregate rolls over.
- **Why seasons**: freshness (recent play matters), fairness (newcomers can top a season without
  overtaking an all-time leader), and a natural recurring hook.
- **Season best is per-season**: the season score uses the best sub-score **achieved within the
  season**, so it reflects the season's play. #5 keeps the lightweight per-session summary
  long-term (sub-scores survive the 90-day detail prune), so seasons are computable.

### D4 — Maintain a per-(user, season, piece, mode) season best on ingest; compute ranking on read
On each ingested session (the #6 hook), monotonically upsert the player's **season best** for
that (piece, mode) if the new sub-score beats it — idempotent under at-least-once ingest. The
global season score is then a difficulty-weighted best-N over those season bests, computed on
read (low volume) or denormalized later.
- **Why a season-best store**: keeps reads fast and makes the best-N/difficulty computation a
  straightforward aggregation; monotonic upsert composes with #5's effectively-once delivery.
- **Alternative**: recompute from raw `play_sessions`. Rejected — #5 prunes heavy detail; and a
  season-best store is cheaper and season-scoped.

### D5 — Privacy identical to #6: viewing open, listing gated, own rank always
**Viewing** the global boards is open to any authenticated user (incl. under-16 / private).
**Being listed** requires the #5 opt-in + age-eligibility (fail-closed). A caller always sees
**their own** global rank and score among the public entries. No sensitive fields in entries.
- **Why**: consistency with #6/#5 — a global board must not become a backdoor exposing a
  private user or a minor.

## Risks / Trade-offs

- **Volume/easy-farming** → best-N + difficulty weighting (D1) make grinding easy pieces
  ineffective; you climb by playing hard pieces well. [Residual gaming] → tune N/weights in
  config; integrity checks inherited from #6.
- **Difficulty signal quality** → weights key off catalog `level`, which can be missing/heuristic
  (`level_source`). Mitigation: neutral default weight for unleveled; refine with facets later.
- **Season complexity** → boundaries, snapshots, rollover. Mitigation: UTC boundaries, a simple
  season-best store, snapshot as a lightweight history row set; season length is config.
- **Privacy** → identical gate to #6 (D5); private/minor never listed; fail-closed.
- **Erasure** → account deletion removes the user's season bests and snapshot entries (extend
  #5/#6 cascade / `purge_user`).
- **Coverage** → Rust tests for best-N/difficulty aggregation, season rollover + snapshot,
  monotonic season best, ranking/tie-break, listing gate + own-rank; Flutter tests via fakes.

## Migration Plan

1. Backend: a `global_season_bests` store (user, season, piece, mode, best sub-score) maintained
   on the #6 ingest hook; global-score computation + ranked read RPC (mode + season → public
   page + caller's own rank/score), visibility-gated; season boundary + end-of-season snapshot
   job (reuse `cymbra-worker`); extend erasure. Config for N, weights, season length.
2. App: a Community/Leaderboards destination screen (boards + season selector) + global-rank
   standing on the profile.
3. **Rollback**: additive; removing the global board + RPCs + screen leaves #5/#6 intact.

## Open Questions

- **N, difficulty weights, season length** — ship config defaults (e.g. N=20; beginner 1.0 /
  intermediate 1.5 / advanced 2.0 / unleveled 1.0; monthly) and tune with real usage.
- **Combined "overall" board** — a weighted sum of tempo+reaction as a third board: include now
  or defer? Leaning defer.
- **Snapshot depth** — how many seasons of hall-of-fame history to keep.
