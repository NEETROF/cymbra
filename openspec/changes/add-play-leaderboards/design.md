## Context

Change #6, the competitive payoff for the play data #5 persists. `performance-scoring`
already emits, per run: an overall sync %, and — crucially here — **per-mode sub-scores**
(a **tempo** sub-score over free-run onsets, a **reaction** sub-score over Wait-Mode onsets),
a run classification (`free`/`wait`/`mixed`), mean tempo offset and mean reaction time, and
per-mode onset counts. Its spec explicitly anticipates feeding "the reaction leaderboard, the
tempo leaderboard, or both". #5 uploads and stores the full session-result record in
`play_sessions`. So #6 only needs to **derive rankings** — no new ingestion.

#5 also established public-profile **visibility** (opt-in, private by default) and a
**minimum-age safeguard** (server-enforced, fail-closed). Leaderboards are a public surface,
so they must respect that gate: appearing on a public board is a form of public exposure.

## Goals / Non-Goals

**Goals:**
- Per-piece, per-mode (tempo/reaction) leaderboards, ranked meaningfully with a stable
  tie-break.
- A monotonic personal best per (player, piece, mode), derived from ingested sessions.
- Public listing strictly gated by #5's opt-in + age-eligibility; a player always sees their
  own best and rank.
- Basic integrity checks; own data always visible.

**Non-Goals:**
- A global cross-piece "best players" board (possible future); friends filtering; prizes;
  robust anti-cheat (client-authoritative scoring is a known limit).

## Decisions

### D1 — Two boards per piece: tempo and reaction, over `accepted` catalog scores only
For each **`accepted` catalog score**, maintain a **tempo** board (free-run sub-score) and a
**reaction** board (Wait-Mode sub-score). User uploads (`user_scores`) are owner-private, so
they have no shared board.
- **Why per-mode**: the two measure different skills (playing in time vs reacting), already
  scored separately; mixing them would be meaningless. A `mixed` run contributes to **both**
  boards via its two sub-scores; a pure run contributes to one.
- **Why accepted-only**: a board is a shared/public artifact; only validated public pieces
  qualify, consistent with hub visibility.

### D2 — Rank by the per-mode sub-score; defined tie-break
Rank descending by the per-mode **synchronization sub-score** (0–100). Tie-break: the mode's
timing quality — **smaller mean tempo offset** (tempo board) / **faster mean reaction time**
(reaction board) — then **earliest achieved** (first to reach it wins ties). The ranking
metric and tie-break order are config.
- **Why the sub-score, not raw reaction time**: the sub-score is the normalized, comparable
  quality already computed; raw timing is the human-meaningful tie-break and a display detail.

### D3 — Monotonic personal best per (user, piece, mode), maintained on ingest
Keep a `leaderboard_bests` row per (user_id, catalog_score_id, mode) holding the best
sub-score, its tie-break metric, and achieved_at. On each session ingest (#5's
`RecordPlaySession`), **upsert the best only if the new result is better** — a monotonic
update, so it is **idempotent and safe under at-least-once ingest** (a replayed session never
lowers or duplicates a best). Boards read from `leaderboard_bests`.
- **Why maintain on ingest**: cheap, keeps boards read-fast, and monotonic upsert composes
  perfectly with #5's effectively-once delivery.
- **Alternative**: compute boards on the fly from `play_sessions`. Rejected — #5 prunes heavy
  detail after 90 days (D7 there); a durable best must not depend on retained raw sessions.
  Keeping `leaderboard_bests` as its own durable summary survives detail pruning.

### D4 — Public listing gated by #5's visibility + eligibility; own rank always visible
A leaderboard **listed to other players** SHALL include only players whose profile is
**public and age-eligible** (the #5 gate). A **private or ineligible** player is **not shown
to others**, but the API SHALL always return to a caller **their own** best and **their own
rank** among the public entries. Ranks shown to others are computed over public entries.
- **Why**: a board must not become a backdoor that exposes a private user or a minor,
  bypassing #5's safeguard. Fail-closed: unknown/!public ⇒ not listed.
- **Consequence**: a player's own rank is "your position among publicly ranked players", so a
  private player still gets motivating feedback without being exposed.

### D5 — Basic integrity checks; robust anti-cheat deferred
On ingest, validate the submitted result against cheap server-side invariants: sub-scores in
[0,100], onset counts consistent with the piece, timing metrics within plausible bounds; drop
a result from the boards if it fails (still stored as a session, just not board-eligible).
- **Why only basic**: scoring is client-authoritative, so a determined cheat can forge a
  record; full anti-cheat (server re-scoring, attestation) is disproportionate now. Log
  rejected results; revisit if abuse appears.

## Risks / Trade-offs

- **Cheating / forged scores** → basic integrity checks (D5) catch the obviously impossible;
  full prevention is out of scope. [Fake top scores] → range/consistency checks + logging;
  a future server-side re-score could harden it.
- **Privacy leak via a board** → strict #5 gate (D4); private/minor users never listed;
  fail-closed. Own rank is computed but not exposed to others.
- **Best lost when detail pruned** → `leaderboard_bests` is its own durable store (D3),
  independent of the 90-day detail prune in #5.
- **Idempotency under replay** → monotonic best upsert (D3): a replayed session can't lower or
  duplicate a best. Erasure: on account deletion, remove the user's bests too (extend #5's
  cascade / `purge_user`).
- **Coverage** → Rust tests for monotonic PB, ranking + tie-break, public-only listing +
  own-rank, integrity rejection; Flutter tests for the views via fakes.

## Migration Plan

1. Backend: `leaderboard_bests` table; maintain it in #5's ingest path (monotonic upsert +
   integrity gate); leaderboard read RPCs (piece+mode → public ranked page + caller's own
   rank/best), visibility/eligibility-gated; extend erasure to bests.
2. App: leaderboard views from a score and the profile; show own rank/PB.
3. **Rollback**: additive; removing the boards + RPCs leaves #5's play data intact.

## Open Questions

- **Global/aggregate board** — a cross-piece "top players" ranking (by summed/averaged bests)
  is appealing for community but needs an aggregation rule; deferred as a possible #7.
- **Ranking metric confirmation** — sub-score primary with timing tie-break (D2); confirm, or
  rank reaction board directly by reaction time.
- **Board size / paging** — top-N page size and whether to expose full paging.
