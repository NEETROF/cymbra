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

### D3 — Payload is the existing immutable session-result record
`performance-scoring` already produces, at song end, an **immutable, serializable
session-result record** — explicitly designed "so a later change can upload it to the server
and route it to the correct leaderboard(s)". #5 is that later change: `RecordPlaySession`
carries that record (its **overall synchronization percentage**, per-mode sub-scores, run
classification `free`/`wait`/`mixed`, per-dimension aggregates, best combo, piece identity,
hand(s), …), and `play_sessions` stores it (id = client session id, user_id, score id,
played_at + client tz, the overall sync %, and the record as JSONB). The heatmap reads a
**per-day aggregate** (count + average overall sync % per user per local day) — computed on
demand first, denormalized later if needed. Day bucketing uses the user's timezone.
- **Why the full record**: it is already the canonical session artifact and is meant for
  server upload; storing it now **enables future leaderboards** (reaction/tempo) with no
  re-plumbing, while #5 itself only needs the overall sync % + count for the heatmap.
- **Why on-demand aggregate first**: volume is low (per the prod plan); avoid premature
  denormalization.

### D4 — Heatmap color = the day's average overall synchronization percentage
The session "success rate" is `performance-scoring`'s **overall synchronization percentage
(0–100)** — the single quality figure combining timing, correctness, and sustain. One cell
per day: **Color** maps to the day's **average overall sync %** (the requested weighting —
e.g. a low-to-high scale). **Presence/intensity** and the **tooltip** convey the **count** of
songs played (and the exact average %). Empty days render as blank cells.
- **Why this metric**: it is the established per-run success score; using it keeps the grid
  consistent with the in-app gauge and the session summary. Per-mode sub-scores are richer but
  belong to leaderboards, not the at-a-glance color.

### D5 — Public profile: explicit allow-list of public fields; sensitive fields never public
A public profile read returns only an **allow-listed** field set: handle/display name,
level, badges, the play heatmap, and songs-played totals. It MUST NOT include email, the
**curator alignment/reliability** figures (a moderation-trust signal, not for other players),
or any moderation state. A **visibility control** lets a user make their profile
public/limited/private.
- **Why allow-list (not deny-list)**: fail-closed — a new sensitive field is private by
  default unless explicitly added to the public set.
- **Default visibility (decided): opt-in — private by default.** Going public is an explicit
  user choice (RGPD-aligned, and protects minors). Community growth is driven by a friendly
  prompt to opt in, not by exposing everyone by default.
- **Viewer gating**: profiles are viewable by **authenticated** players (not anonymous web),
  consistent with the app's authenticated surfaces.

### D6 — Minor safeguard: neutral age gate, config threshold (default 16), server-enforced
Making a profile public SHALL require the user to be at least a **configured minimum age**
(`min_public_sharing_age`, **default 16**). The threshold is a single global config value —
16 is the strictest EU digital-consent age, so it is compliant EU-wide without per-country
detection, and comfortably above the UK/US 13; it can later become a country→age map without
schema change.
- **Neutral age gate at opt-in only**: age is asked **only when** the user first tries to go
  public (not at signup — data minimization), with a **neutral** prompt (ask the date of
  birth, not a leading "are you 16+?" checkbox).
- **Store only a derived date, discard the DOB**: from the DOB compute
  `share_eligible_from = DOB + min_public_sharing_age years` and persist only that as a plain
  **`DATE`**; the DOB is not stored.
- **Server-enforced, fail-closed**: `SetProfileVisibility(public)` refuses if the user is not
  yet eligible; the public-profile read is also fail-closed (never expose a profile whose
  owner is not eligible), so a modified client cannot bypass it. Under-age users stay private
  and keep full use of everything else; no parental-consent flow in v1.
- **UTC date check with a safe margin (not the user's timezone)**: eligibility is a **date**
  comparison done server-side in **UTC**; the only edge is the 16th-birthday day, resolved
  conservatively (`current_date_utc > share_eligible_from`, i.e. a one-day margin) so a
  timezone difference can never grant eligibility early. Being eligible one day late is
  harmless and self-corrects. **This is deliberately different from the heatmap** (D3/D4),
  which buckets by the **user's local timezone** because that is about lived UX, not safety.

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

## Resolved Questions

- **Default profile visibility (decided): opt-in — private by default**, with a friendly
  prompt to go public (D5).
- **Minor safeguard (decided): config `min_public_sharing_age`, default 16**, neutral age
  gate at opt-in, store only `share_eligible_from` (DATE), server-enforced fail-closed, UTC
  date check with a one-day margin; heatmap stays local-tz (D6).

## Resolved Questions (continued)

- **Public field allow-list (decided)**: handle/display name, level, badges, the play heatmap,
  and songs-played total. Explicitly excluded and never public: email, the curator
  alignment/reliability figures, and any moderation state.
- **Success metric for the color (decided)**: `performance-scoring`'s **overall
  synchronization percentage (0–100)**; the per-day cell color is the day's average of it.

## Open Questions

- **Retention/cap** of `play_sessions` and outbox entries — set a sane retention; never drop
  un-acked outbox entries.
- **Leaderboards** (reaction/tempo) are **out of scope** here but **enabled** by storing the
  full session-result record; a future change can add them without re-plumbing ingestion.
