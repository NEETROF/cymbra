## Context

Change #5, building on #4's full-screen curator profile. Today the end-of-session summary
(`session-summary`, `session_summary.dart`, driven by `performance-scoring`) is **local
only** — nothing is persisted server-side. To power a public profile and a GitHub-style
activity grid, play stats must reach the server, and — per the explicit requirement —
**without loss**, even when the server can't record them at that instant.

Existing building blocks: the app is Riverpod 2 + Freezed; it already computes per-session
success/sync metrics; the backend is gRPC-only (tonic) with idempotent upsert patterns
(e.g. crawler `ON CONFLICT DO NOTHING`) already in use; users have a `handle`/`display_name`.

## Goals / Non-Goals

**Goals:**
- Deliver end-of-session stats to the server **reliably: at-least-once + idempotent =
  effectively-once, nothing lost**, surviving app restarts and transient server failure.
- Persist per-session records and aggregate **per day** (count + success rate).
- A **daily heatmap** on the profile colored by success rate.
- **Public profiles** viewable by other players, with a safe public field set and a
  visibility control.

**Non-Goals:**
- Social discovery (search, follow, leaderboards); changing the scoring model; a real-time
  activity feed; push notifications.

## Decisions

### D1 — Durable client outbox, not fire-and-forget
At session end the app writes a session record to a **durable local outbox** (a persisted
store — e.g. a local DB/box that survives process death), independent of network state. A
background **sender** drains the outbox: on success it removes the entry; on failure
(offline, timeout, server busy/5xx) it keeps the entry and retries with **exponential
backoff + jitter**. Entries are removed **only after server acknowledgement**.
- **Why**: "nothing is lost" requires the stat to be durably captured *before* any network
  attempt, and to outlive app kills. Fire-and-forget or in-memory retry would drop stats on
  a crash/offline close.
- **Alternative**: send synchronously at session end and show an error on failure. Rejected
  — that loses stats whenever the network/server hiccups, exactly the failure we must avoid.

### D2 — Idempotent server ingestion keyed by a client session id
Each session gets a **client-generated id** (UUID v7) at creation. `RecordPlaySession`
upserts by that id (`ON CONFLICT (id) DO NOTHING`), so a retried delivery is a no-op and a
session is **never double-counted**. The client keeps retrying the *same* id until acked.
- **Why**: at-least-once delivery (D1) needs idempotency to become effectively-once. A
  client-side id lets the client retry safely without server round-trips to dedupe.
- **Result**: at-least-once (client) + idempotent (server) ⇒ no loss, no duplicates.

### D3 — Server storage: per-session rows + per-day aggregate
A `play_sessions` table (id, user_id, catalog/user score id, played_at, success_rate, plus
the summary metrics) is the source of truth. The heatmap reads a **per-day aggregate**
(count + average success rate per user per local day) — computed on demand first, denormalized
later if needed. Day bucketing uses the user's timezone (sent with the record) so the grid
matches what the player experienced.
- **Why on-demand first**: volume is low (per the prod plan); avoid premature denormalization.

### D4 — Heatmap encodes two dimensions: count and success rate
One cell per day. **Color** maps to the day's **average success rate** (the requested
weighting — e.g. a low-to-high scale). **Presence/intensity** and the **tooltip** convey the
**count** of songs played (and the exact success %). Empty days render as blank cells.
- **Why**: the user asked specifically for color = success percentage; count is the second
  dimension carried without fighting the color channel.

### D5 — Public profile: explicit allow-list of public fields; sensitive fields never public
A public profile read returns only an **allow-listed** field set: handle/display name,
level, badges, the play heatmap, and songs-played totals. It MUST NOT include email, the
**curator alignment/reliability** figures (a moderation-trust signal, not for other players),
or any moderation state. A **visibility control** lets a user make their profile
public/limited/private.
- **Why allow-list (not deny-list)**: fail-closed — a new sensitive field is private by
  default unless explicitly added to the public set.
- **Default visibility**: the stated intent is that profiles are visible to other players, so
  the default is **public**; the opt-out exists for privacy/RGPD. Confirm the default (see
  Open Questions).
- **Viewer gating**: profiles are viewable by **authenticated** players (not anonymous web),
  consistent with the app's authenticated surfaces.

## Risks / Trade-offs

- **Stat loss (the core risk)** → durable outbox before any network attempt + retry-until-ack
  + idempotent ingest. [Crash mid-send] → entry stays until acked, resent next launch.
  [Duplicate send] → idempotent by session id. Tested with restart/offline/duplicate cases.
- **Outbox growth if server long-unavailable** → cap retries with backoff (not drop);
  entries persist; optionally coalesce very old entries. Never delete un-acked stats.
- **Clock/timezone skew on day bucketing** → record the client's timezone/offset with the
  session so the grid buckets by the player's local day, not UTC.
- **Privacy leak via public profile** → allow-list public fields; exclude email + moderation
  reliability; opt-out control; authenticated viewers only. RGPD-aligned.
- **Gaming the grid** → the grid reflects real recorded sessions; success rate comes from
  `performance-scoring`; no direct incentive/points attached to the grid itself here.
- **Coverage** → Flutter tests for outbox durability/retry/no-loss (fakes + simulated
  failures); Rust tests for idempotent ingest, per-day aggregation, and public-field allow-list.

## Migration Plan

1. Backend: `play_sessions` table + idempotent `RecordPlaySession` + per-day aggregate read
   + public-profile read (allow-listed) + a user visibility flag. Additive.
2. App: outbox store + sender (backoff), enqueue at session end; heatmap on the #4 profile;
   public-profile view; visibility setting.
3. **Rollback**: the outbox/sender and profile additions are additive; disabling them leaves
   play + the local session summary working. `play_sessions` data is inert if unused.

## Open Questions

- **Default profile visibility** — public by default (per the stated intent) with opt-out, or
  opt-in? RGPD-safe either way; confirm the default.
- **Which fields are public** — confirm the allow-list (handle, level, badges, heatmap,
  songs-played). Explicitly excluded: email, curator alignment/reliability, moderation state.
- **Success-rate metric for the color** — reuse `performance-scoring`'s primary success/sync
  figure; confirm which metric drives the cell color.
- **Retention/cap** of `play_sessions` and outbox entries — set a sane retention; never drop
  un-acked outbox entries.
