# Design

## Context

Per-piece bests are monotonic server-side and feed `GetLeaderboard` / `GetMyStandings`;
the app batches standings per frame in `MyStandings` for the score cards' rank badges and
refreshes them when a played session is delivered. The session summary knows the run's
overall percentage. The piece step's default pass threshold is 80 %. See proposal.md — Why.

## Goals / Non-Goals

**Goals:**
- Stars that are exactly the stored best, on every surface, with one derivation.
- Zero new storage.

**Non-Goals:**
- No per-hand or per-mode stars; the overall percentage is the one the player knows.
- No configurable thresholds server-side: three constants, stated in the spec, aligned
  with the pass threshold; revisit only if the pass threshold becomes content-configurable.
- No stars for drums pieces in this change (the drum best is the same field; the widget is
  keyboard-only until drum scoring is out of its own change).

## Decisions

### D1 — Stars are derived client-side from the best, thresholds as constants
`starsFor(bestPct) => pct >= 95 ? 3 : pct >= 80 ? 2 : pct >= 60 ? 1 : 0` in a pure module.
Alternatives: computing stars server-side; rejected because the leaderboards already send
the best and a second derived field would be a second source of truth.

### D2 — Reuse `MyStandings`, extend the standing with the best percentage
`LeaderboardStanding.best_overall_sync_pct` (float, additive). Cards already request their
piece's standing; the star row reads the same map, so a page of cards still costs one call
and refreshes on delivery like the rank badge.

### D3 — The summary compares the run to the previous best
The summary shows the stars this run reached; if the run's stars exceed the stars of the
standing known before the run, it raises the new-star cue once. A standing not loaded yet
counts as zero stars, so the first scored run on a piece always celebrates its stars.

### D4 — Empty stars on unplayed pieces
Three outlined stars, no text: the invitation is the design.

## Risks / Trade-offs

- [Standings only exist for ranked players] → a piece with no standing shows empty stars;
  the first delivered run creates the standing.
- [A practice-range run has no score] → stars are untouched by practice; the summary for a
  practice run shows no star row.
