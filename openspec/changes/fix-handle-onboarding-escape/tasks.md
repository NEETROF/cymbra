## 1. App: escape hatch

- [x] 1.1 Add a low-emphasis "Use a different account" action to `handle_onboarding_screen.dart` (via `AuthScaffold`), always visible regardless of handle availability
- [x] 1.2 For an account that already has a handle, wire the action to `SessionNotifier.signOut` (best-effort `Logout` → clear tokens → entry screen)
- [x] 1.3 Widget test: action is present even when the handle field is empty/taken, and triggering it returns to the entry screen with no session

## 2. App: delete-on-abandon for brand-new accounts

- [x] 2.1 Add an `abandonOnboarding()` path (notifier/flow) that, when the current account has no handle, calls `DeleteAccount` reusing the in-hand re-auth (fresh OIDC `id_token` or the just-entered password) instead of a plain sign-out
- [x] 2.2 Route the escape action to `abandonOnboarding()` when `needsHandle`, else to `signOut`
- [x] 2.3 Tests: handle-less account → escape deletes it; handled account → escape only signs out; delete failure still clears the local session and returns to entry

## 3. Backend: orphan reaper

- [x] 3.1 Pure, host-testable selection predicate in the user module: account is eligible iff `handle IS NULL` AND `created_at < now - grace` (`reaper_core::reapable`/`cutoff`; migration `0003_created_at.sql`)
- [x] 3.2 Repo query + delete for eligible accounts (`delete_orphans_before`, pg + fake); `UserModule::reap_orphans` driven by a startup background loop in `main.rs` gated by `CYMBRA_ORPHAN_REAP_GRACE` (0 disables)
- [x] 3.3 Unit tests for the predicate (boundary at cutoff, has-handle, recent) and the module reap over the fake repo + config defaults/disable

## 4. Validation

- [x] 4.1 `flutter analyze` + `dart run custom_lint` + `dart format` clean; `cargo fmt` + `cargo clippy -D warnings` clean
- [x] 4.2 Suites green (234 Flutter unit/widget + 21 backend user/platform); new code covered (session_notifier 92%, handle screen 93%, reaper_core 100%, module.reap 92%). Aggregate ≥80% is the CI gate over merged unit+widget+integration / full Rust workspace (not reproducible locally without the integration run)
- [ ] 4.3 Manual: sign in with Google, on the handle screen use the escape action, confirm return to entry and that no handle-less account remains (grpcurl `ListIdentities`/DB check) — **manual, pending** (run against the live app + backend)
- [x] 4.4 `openspec validate fix-handle-onboarding-escape --strict` passes
