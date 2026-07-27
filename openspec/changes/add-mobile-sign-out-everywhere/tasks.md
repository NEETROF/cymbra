## 1. gRPC stubs + service seam

- [ ] 1.1 Regenerate the Dart gRPC stubs (`gen-grpc`) so `AuthService.RevokeAllSessions` (+ its request/response) is available. (Depends on `add-session-management` being on the proto.)
- [ ] 1.2 Add `revokeAllSessions()` to the `AuthService` interface and implement it in `GrpcAuthService` (`lib/services/grpc_client.dart`) as an **authenticated** call (`_authed` → `_client.revokeAllSessions(RevokeAllSessionsRequest())`), mirroring `deleteAccount`. No refresh token argument.

## 2. Session state

- [ ] 2.1 Add `SessionNotifier.signOutEverywhere()` (`lib/state/session_notifier.dart`): call `revokeAllSessions()`; **on success** run the same local teardown as `signOut()` (clear tokens, OIDC sign-out, route to entry); **on failure** keep the session and surface the error (do NOT clear locally). Share the teardown with `signOut()` to avoid drift.

## 3. UI

- [ ] 3.1 Add a "Sign out from all devices" item to `account_menu.dart` with a **confirmation dialog**, wired to `sessionNotifierProvider.notifier.signOutEverywhere()`.
- [ ] 3.2 Localize the label, dialog, and any error message across `app_en/fr/es/it.arb` (+ regenerate l10n).

## 4. Tests

- [ ] 4.1 `SessionNotifier` unit test (fake `AuthService`): success revokes all + clears tokens + routes to entry; a failed revoke keeps the session (tokens NOT cleared) and surfaces the error.
- [ ] 4.2 Widget test for the account-menu item: confirmation is shown; confirming triggers the action; cancelling does nothing. Fakes only, no native lib.

## 5. Checks

- [ ] 5.1 `melos run analyze` + `dart format` clean; `flutter test --coverage` keeps line coverage ≥ 80%.
- [ ] 5.2 `openspec validate add-mobile-sign-out-everywhere --strict` passes.
