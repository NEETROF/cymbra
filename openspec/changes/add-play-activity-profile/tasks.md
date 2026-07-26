## 1. Server persistence & ingestion

- [ ] 1.1 Migration: `play_sessions` (id UUID PK = client session id, user_id, catalog/user score id, played_at TIMESTAMPTZ, tz offset, success_rate, summary metrics JSONB, created_at) + index by (user_id, played_at).
- [ ] 1.2 `RecordPlaySession` RPC (authenticated): idempotent upsert `ON CONFLICT (id) DO NOTHING`; returns an acknowledgement. Reject unauthenticated.
- [ ] 1.3 Per-day aggregate read (count + avg success rate per user per local day, bucketed by the recorded tz); on-demand query first, denormalize later if needed.

## 2. Reliable client delivery (app, no loss)

- [ ] 2.1 A durable local outbox (persisted store surviving app restarts) that stores a session record with a client-generated UUID v7 id at session end, before any network attempt.
- [ ] 2.2 Hook at session end (`session-summary`/`performance-scoring` result) to enqueue the summary + success rate + client tz into the outbox.
- [ ] 2.3 A background sender that drains the outbox: send → on ack remove; on failure keep + retry with exponential backoff + jitter; never drop an un-acked entry; resume on app launch.
- [ ] 2.4 Idempotent by session id end-to-end (client resends the same id; server dedupes); confirm no double-count and no loss across offline/restart/duplicate.

## 3. Heatmap (app)

- [ ] 3.1 A play-heatmap widget on the #4 curator profile: one cell/day, color by the day's avg success rate, count via intensity + tooltip (count + exact success %), empty days blank. Driven by an injectable provider reading the per-day aggregate.

## 4. Public profiles (backend + app)

- [ ] 4.1 A user profile-visibility setting (public/limited/private) on the user model + an RPC to update it.
- [ ] 4.2 `GetPlayerProfile(user)` read returning only the allow-listed public fields (handle/display name, level, badges, heatmap, songs-played); NEVER email, curator alignment/reliability, or moderation state; honor the visibility setting; reject unauthenticated.
- [ ] 4.3 App: a read-only public-profile view (reuses the profile widgets from #4 with the public field set) and an entry point to open another player's profile.
- [ ] 4.4 App: a visibility setting UI (public/limited/private).

## 5. Tests & verification

- [ ] 5.1 Flutter: outbox durability (survives restart), retry-until-ack, never drops un-acked, offline capture, duplicate send is safe — assert **no loss, no double-count**. `flutter test --coverage` ≥ 80%.
- [ ] 5.2 Flutter: heatmap colors by success rate, conveys count, blanks empty days (via a fake aggregate); public-profile view shows only public fields.
- [ ] 5.3 Rust: idempotent ingest (same id no-ops), per-day aggregation by local tz, public read allow-list (no email/alignment/moderation), visibility honored, unauthenticated rejected. `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 5.4 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [ ] 5.5 `openspec validate add-play-activity-profile --strict` passes.
