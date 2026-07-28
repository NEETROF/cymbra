## Context

`SessionNotifier` (`lib/state/session_notifier.dart`, a Riverpod `@riverpod` notifier)
owns the app's session. Its `signOut()` does a **best-effort** `Logout` (ignoring
failures) then always clears the local session (`_tokens.clear()`, OIDC sign-out) and
routes to the entry screen. `account_menu.dart` triggers it. Auth RPCs go through an
`AuthService` seam implemented by `GrpcAuthService` over the generated `AuthServiceClient`
(`lib/services/grpc_client.dart`); authenticated calls (e.g. `deleteAccount`) attach the
bearer via the `_authed` helper. Tokens live in `token_store` (secure storage).

The backend now exposes `AuthService.RevokeAllSessions` (change: `add-session-management`)
— caller identity from the access token, revokes every session for the account.

## Goals / Non-Goals

**Goals:**
- One account action to sign out of **all** the user's sessions, reusing the existing
  `AuthService` seam and `SessionNotifier`.
- Correct teardown: since the current refresh token is revoked too, end the local
  session on success.

**Non-Goals:**
- A session list / per-device revoke / "this device" flag (single button only).
- Any change to sign-in, refresh, logout, or token semantics.
- Backend work — `RevokeAllSessions` already exists.

## Decisions

**1. A new authenticated seam method `revokeAllSessions()`.**
Add it to the `AuthService` interface and implement in `GrpcAuthService` as an
authenticated call (`_authed` → `_client.revokeAllSessions(RevokeAllSessionsRequest())`),
mirroring `deleteAccount`. No refresh token is passed — the caller is identified by the
access token.

**2. Revoke first, then tear down — unlike best-effort logout.**
`SessionNotifier.signOutEverywhere()` calls `revokeAllSessions()` and **only on success**
runs the same local teardown as `signOut()` (clear tokens, OIDC sign-out, route to
entry). A failed revoke leaves the session intact and surfaces the error — the user is
told it didn't work rather than being silently signed out locally while other devices
stay live. (Plain `signOut()` stays best-effort: its local logout is idempotent and
losing connectivity shouldn't trap you signed in.)

**3. Confirm before acting.** The account menu shows a confirmation dialog ("sign out of
all devices?") before calling the action — it's destructive across devices.

**4. Riverpod 2 + Freezed only.** The action is a method on the existing
`@riverpod SessionNotifier`; dependencies (`AuthService`, `token_store`) are providers,
overridden with fakes in tests. No `ChangeNotifier`/`setState` (repo CLAUDE.md).

## Risks / Trade-offs

- **Residual access-token window** → other devices keep their in-memory access token
  until it expires (~15 min); revocation is immediate at the refresh layer. Inherited
  from `add-session-management`; documented, not solved here.
- **Ordering vs. logout** → the deliberate difference (revoke-then-teardown vs.
  best-effort) is a small inconsistency, justified by decision 2; both share the local
  teardown code to avoid drift.
- **Stub regen** → depends on the regenerated Dart protos carrying `RevokeAllSessions`;
  CI runs `gen-grpc` before analyze/test.
