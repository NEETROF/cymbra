## 1. Server persistence & ingestion

- [x] 1.1 Migration: `play_sessions` (id UUID PK = client session id, user_id, catalog/user score id, played_at TIMESTAMPTZ, tz offset, overall_sync_pct, session_result JSONB = the immutable session-result record, created_at) + index by (user_id, played_at).
- [x] 1.2 `RecordPlaySession` RPC (authenticated): carries the serializable session-result record from `performance-scoring`; idempotent upsert `ON CONFLICT (id) DO NOTHING`; returns an acknowledgement. Reject unauthenticated. (Storing the full record enables future leaderboards; #5 uses only the overall sync %.)
- [x] 1.3 Per-day aggregate read (count + avg overall synchronization % per user per local day, bucketed by the recorded tz); on-demand query first, denormalize later if needed.
- [x] 1.4 Retention & erasure: config `play_detail_retention_days` (default 90) + a prune of the heavy per-session detail (keep summary/aggregate); `play_sessions` FK to users `ON DELETE CASCADE` and extend the `purge_user` worker job so account deletion erases all play data. Outbox: never prune un-acked entries (client side, task 2.3).

## 2. Reliable client delivery (app, no loss)

- [x] 2.1 A durable local outbox (persisted store surviving app restarts) that stores a session record with a client-generated UUID v7 id at session end, before any network attempt.
- [x] 2.2 Hook at session end to enqueue the `performance-scoring` immutable session-result record (incl. overall sync %) + client tz into the outbox.
- [x] 2.3 A background sender that drains the outbox: send → remove **only on the server's persisted-ack** (never on mere request-sent); on failure keep + retry with exponential backoff + jitter; never drop an un-acked entry. **Resume triggers**: app launch, regained connectivity, and user (re)authentication; entries stay pending without a valid token and flush once signed in; deliver each entry under the producing account's identity.
- [x] 2.4 Idempotent by session id end-to-end (client resends the same id; server dedupes); confirm no double-count and no loss across offline/restart/duplicate.

## 3. Heatmap (app)

- [x] 3.1 A play-heatmap widget on the #4 curator profile: one cell/day, color by the day's avg **overall synchronization %**, count via intensity + tooltip (count + exact avg %), empty days blank. Driven by an injectable provider reading the per-day aggregate.

## 4. Public profiles (backend + app)

- [x] 4.1 User model: a profile-visibility setting (**default private**; public/limited/private) + a `share_eligible_from DATE` (nullable) + config `min_public_sharing_age` (default 16). Migration additive. Do NOT store date of birth.
- [x] 4.2 `SetProfileVisibility` RPC: to set public, require the user eligible — server-side, fail-closed, `current_date_utc > share_eligible_from` (one-day margin); refuse otherwise. A neutral age-gate flow computes `share_eligible_from = DOB + min_public_sharing_age years` from a DOB entered only at opt-in and discards the DOB.
- [x] 4.3 `GetPlayerProfile(user)` read returning only the allow-listed public fields (handle/display name, level, badges, heatmap, songs-played); NEVER email, curator alignment/reliability, or moderation state; honor visibility AND eligibility (fail-closed); reject unauthenticated.
- [x] 4.4 App: a read-only public-profile view (reuses the #4 profile widgets with the public field set) + an entry point to open another player's profile.
- [x] 4.5 App: visibility setting UI (default private) with an opt-in flow that shows the **neutral age gate** (asks date of birth, sends it once to derive eligibility; nothing about DOB kept locally) and a friendly prompt to go public.

## 5. Tests & verification

- [x] 5.1 Flutter: outbox durability (survives restart), remove only on persisted-ack, retry-until-ack, never drops un-acked, offline capture, duplicate send is safe, **resume on launch / connectivity / sign-in**, per-user delivery — assert **no loss, no double-count**. `flutter test --coverage` ≥ 80%.
- [x] 5.2 Flutter: heatmap colors by success rate, conveys count, blanks empty days (via a fake aggregate); public-profile view shows only public fields.
- [x] 5.3 Rust: idempotent ingest (same id no-ops), per-day aggregation by local tz, public read allow-list (no email/alignment/moderation), visibility default private + honored, age gate fail-closed (under-min-age refused to go public; UTC one-day margin; DOB not stored), unauthenticated rejected. `cargo llvm-cov ... --fail-under-lines 80`.
- [x] 5.4 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [x] 5.5 `openspec validate add-play-activity-profile --strict` passes.
