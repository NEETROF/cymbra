## 1. App: escape hatch

- [ ] 1.1 Add a low-emphasis "Use a different account" action to `handle_onboarding_screen.dart` (via `AuthScaffold`), always visible regardless of handle availability
- [ ] 1.2 For an account that already has a handle, wire the action to `SessionNotifier.signOut` (best-effort `Logout` → clear tokens → entry screen)
- [ ] 1.3 Widget test: action is present even when the handle field is empty/taken, and triggering it returns to the entry screen with no session

## 2. App: delete-on-abandon for brand-new accounts

- [ ] 2.1 Add an `abandonOnboarding()` path (notifier/flow) that, when the current account has no handle, calls `DeleteAccount` reusing the in-hand re-auth (fresh OIDC `id_token` or the just-entered password) instead of a plain sign-out
- [ ] 2.2 Route the escape action to `abandonOnboarding()` when `needsHandle`, else to `signOut`
- [ ] 2.3 Tests: handle-less account → escape deletes it; handled account → escape only signs out; delete failure still clears the local session and returns to entry

## 3. Backend: orphan reaper

- [ ] 3.1 Pure, host-testable selection predicate in the user module: account is eligible iff `handle IS NULL` AND `created_at < now - grace`
- [ ] 3.2 Repo query + delete for eligible accounts; a maintenance entry point (scheduled task / invokable command) gated by a configurable grace period
- [ ] 3.3 Unit tests for the predicate (boundary: exactly at grace, has-handle, recent) and the repo delete

## 4. Validation

- [ ] 4.1 `flutter analyze` + `dart run custom_lint` + `dart format` clean; `cargo fmt` + `cargo clippy -D warnings` clean
- [ ] 4.2 `flutter test --coverage --exclude-tags golden` and `cargo llvm-cov` green, coverage ≥ 80% both ecosystems
- [ ] 4.3 Manual: sign in with Google, on the handle screen use the escape action, confirm return to entry and that no handle-less account remains (grpcurl `ListIdentities`/DB check)
- [ ] 4.4 `openspec validate fix-handle-onboarding-escape --strict` passes
