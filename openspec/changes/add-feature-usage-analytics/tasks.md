## 1. Database schema

- [x] 1.1 Write a migration that `CREATE SCHEMA IF NOT EXISTS analytics` (dedicated schema, per D10) and creates `analytics.usage_events` (id, user_bucket, action, variant NULL, subject_id NULL, platform, device_class, app_version, locale, occurred_at, received_at) with indexes on `occurred_at`, `action`, and a composite for common filters
- [x] 1.2 In the same migration create `analytics.usage_action_daily` keyed by `(day, action, variant, platform, device_class, app_version, locale)` with an `event_count`, PK/unique on the grain (`subject_id` stays raw-only, per D8)
- [x] 1.3 Create `analytics.usage_user_daily` keyed by `(day, user_bucket, platform, device_class)` (presence), unique on the grain, index on `day`
- [x] 1.4 Verify the migration applies cleanly on a fresh DB and is additive/idempotent

## 2. Shared action taxonomy & proto contract

- [x] 2.1 Create the client-owned action registry (a single shared constant). Initial set: `auth_sign_in`, `auth_sign_up`, `play_start` (+subject_id), `play_stop` (+subject_id), `play_mode_switch` (+variant: fall_note|vertical_partition|full_partition), `settings_change` (+variant: piano_type|hand_left|hand_right|…), `score_upload` (+subject_id), `score_propose` (+subject_id), `soundfont_upload`, `soundfont_propose`, `profile_view`, `favorite_add` (+subject_id), `favorite_remove` (+subject_id). `guest_session_start` is **pending/deferred** (per D9). `action` is a validated string, NOT a proto enum
- [x] 2.2 Add the `UsageEvent` message (action: string, variant: optional string, subject_id: optional string, platform, device_class, app_version, locale, occurred_at) and `ReportEvents` batched RPC on a dedicated `UsageService` (per design D5/D7/D8)
- [x] 2.3 Regenerate Rust + Dart gRPC bindings (`flutter_rust_bridge`/`gen-grpc` as applicable) and confirm both compile

## 3. Feature-flag config

- [x] 3.1 Add a retention-TTL config value to `cymbra-feature-flags` (default 6 months) with a typed accessor
- [x] 3.2 Add a collection kill-switch flag (default on) with a typed accessor
- [x] 3.3 Surface both in the back-office flags panel (`feature-flags-admin`) and confirm they round-trip

## 4. Backend ingestion (Rust, host-testable core + mockall)

- [x] 4.1 Implement period-salted `user_bucket = HMAC(master_secret, "YYYY-MM")` then `hash(user_id, salt)` in a host-testable core module (Option A: derive from master secret, nothing stored, reproducible within a month, unlinkable across months from analytics data alone)
- [x] 4.2 Implement validation in a core module: accept any well-formed `action` and (when present) `variant` matching the agreed shape rule `^[a-z][a-z0-9_]{0,63}$`, rejecting only malformed ones; accept `subject_id` as an optional bounded opaque string; reject out-of-range platform / device_class; clamp implausible `occurred_at`; stamp `received_at`
- [x] 4.3 Implement `PgUsageEventRepo` (insert batch into `analytics.usage_events`) behind a trait, mockall-doubled
- [x] 4.4 Wire the `ReportEvents` handler: authenticate, map to buckets, validate per-event (skip bad ones, keep the batch), persist; return counts
- [x] 4.5 Unit-test core (bucketing determinism within/across periods, validation, batch-with-bad-event) and handler (auth required, partial batch) — mockall doubles, ≥80% coverage on core

## 5. Worker jobs (rollup + purge)

- [x] 5.1 Implement the daily rollup job: fold each closed day of `usage_events` into `usage_action_daily` (counts, grouped incl. `variant`) and `usage_user_daily` (presence) via idempotent upserts (`subject_id` is not rolled up)
- [x] 5.2 Implement the daily purge job: `DELETE FROM usage_events WHERE occurred_at < now() - <TTL from flag>`
- [x] 5.3 Register both on the `cymbra-worker` schedule with rollup ordered before purge; confirm purge only touches already-aggregated days
- [x] 5.4 Tests: rollup idempotency (double-run identical), unique-user exactness over a multi-day window, purge leaves aggregates intact

## 6. Flutter client (Riverpod 2 + Freezed, injectable seam, mockito)

- [x] 6.1 Add a `UsageTrackingService` behind an injectable seam + Riverpod provider; derive `platform` and `device_class` (phone/tablet/desktop) on device
- [x] 6.2 Implement an offline event buffer (local persistence) with periodic best-effort flush via `ReportEvents`; failures silent and retried, no data loss, no user-facing error
- [x] 6.3 Gate emission on the kill-switch flag and the user's consent setting
- [x] 6.4 Add the consent toggle in settings (default opt-out = enabled) wired to a notifier; UI never calls the service directly (only the notifier does)
- [x] 6.5 Instrument the 12 in-scope taxonomy call sites (notifiers), passing `subject_id` (score UUID) for play/upload/propose/favorite actions and `variant` for `play_mode_switch` and `settings_change` (category only, never the value)
- [x] 6.6 Tests: buffer survives offline→online, flush failure is silent, kill-switch/consent suppress emission, device_class derivation — mockito doubles via provider overrides; keep coverage ≥80%; `dart run custom_lint` clean

## 7. Back-office "Usage" screen (Vue 3 + Pinia + ts-pattern)

- [x] 7.1 Add a Pinia store/composable behind the injectable client seam (`lib/api.ts` + `setClientsForTest`); async state as a single `Async<T>` `ts-pattern` union
- [x] 7.2 Backend read RPCs/queries: distinct users by period split by platform/device (from `usage_user_daily`), and action breakdown with composable filters (aggregates + raw within window)
- [x] 7.3 Build the screen: date-range + platform + device_class + action filters (freely combinable) + unique-users-by-platform view; the action filter is populated from `SELECT DISTINCT action` (data-driven, no hard-coded list); admin-scope gated; route + nav entry
- [x] 7.4 Note cross-period distinct-count caveat in the UI
- [x] 7.5 Tests: vitest for the store (`match(...).exhaustive()`), Playwright e2e via the gated fake-client seam (no backend); keep coverage ≥80%

## 8. Privacy, docs, gates

- [x] 8.1 Consent posture decided: opt-out (default on) + per-user toggle, distinct from the global kill-switch — reflect in the settings default and any legal copy
- [x] 8.2 Document the pipeline (taxonomy, buckets, retention, kill-switch) briefly for future maintainers
- [x] 8.3 Run full gates: `cargo fmt`/`clippy` + `cargo llvm-cov ≥80`, `melos run analyze` + `dart format` + Flutter tests ≥80%, back-office vitest + e2e; `openspec validate add-feature-usage-analytics --strict`
- [ ] 8.4 (Deferred slice — `guest_session_start`, D9) Adopted approach when scheduled: backend mints a short-lived, install-scoped, PII-free anonymous guest token (rate-limited at issuance); guest events flow through the same authenticated `ReportEvents` pipeline with a period-salted guest `user_bucket`, enabling distinct-guest counts without an unauthenticated ingress
