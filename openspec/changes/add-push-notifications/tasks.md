## 1. Data model & token registry (backend)

- [x] 1.1 Migration: `push_tokens(user_id, token UNIQUE, platform, created_at, last_seen_at)`, `notification_prefs(user_id, category, enabled)`, and a per-user `timezone` column (or offset).
- [x] 1.2 Repo reads/writes: upsert/refresh token, unregister, prune-by-token; read/write category prefs; read/write timezone. Runtime sqlx queries, fully-qualified names.
- [x] 1.3 RPCs: `RegisterPushToken(token, platform)`, `UnregisterPushToken(token)`, `SetNotificationPref(category, enabled)`, `SetTimezone(tz)`.

## 2. Send seam & selection core (backend)

- [x] 2.1 Define `PushMessage`, `SendOutcome{delivered|retryable|invalid}` and the `PushSender` trait (`#[automock]`).
- [x] 2.2 FCM HTTP v1 implementation of `PushSender` (service-account OAuth; Apple via APNs bridging) — coverage-excluded glue.
- [x] 2.3 Host-testable `select_recipients(candidates, prefs, flags, category, local_hour) -> Vec<Token>` covering kill-switch / category-enable / opt-out / local-hour / token-present; unit-test every gate.
- [x] 2.4 On `invalid` outcome, prune the token (wire selection→send→prune).

## 3. Flags & worker dispatch (backend)

- [x] 3.1 Feature flags via `cymbra-feature-flags`: global kill-switch, per-category enable, per-category schedule hour (hot-reloadable).
- [x] 3.2 A generic worker job (e.g. `push_dispatch`) parameterised by category: load candidates → `select_recipients` → send → prune; retry/channel per `jobs` registry conventions. Register it.
- [x] 3.3 Document the seam for features to add a type (category + candidate query + schedule/trigger + message) without touching the core.

## 4. App integration (Flutter)

- [x] 4.1 Add `firebase_messaging`; platform setup (iOS/macOS APNs entitlements + signing, Android FCM).
- [x] 4.2 A push service seam (injectable): request permission (opt-in), obtain + refresh the FCM token, expose token/permission state — only notifiers call it.
- [x] 4.3 Register/refresh the token on launch for iOS/Android/macOS; unregister on logout; **skip** on Windows/Linux.
- [x] 4.4 Report/refresh the device timezone; a per-category preference toggle in settings.
- [x] 4.5 Widget/state tests via a faked push seam (permission granted/denied; register/skip by platform; pref toggle).

## 5. Back office (Vue)

- [x] 5.1 A notifications panel: global kill-switch, per-category enable, per-category schedule hour — behind the injectable client seam, `Async<T>` unions.
- [x] 5.2 Vitest coverage (load + toggle + error) via the client seam.

## 6. Coverage, gates & verification

- [x] 6.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (selection/consent cores covered; FCM client + gRPC/worker glue excluded).
- [x] 6.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [x] 6.3 Back office: `yarn lint` + `yarn test` green.
- [ ] 6.4 Manual: register a token on iOS + Android + macOS; a test dispatch reaches all three at the local hour; opt-out + kill-switch suppress; Windows/Linux register nothing.
- [x] 6.5 `openspec validate add-push-notifications --strict` passes.
