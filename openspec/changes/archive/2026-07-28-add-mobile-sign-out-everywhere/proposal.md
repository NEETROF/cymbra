## Why

A user who loses a device, or signs in on a shared/borrowed one, has no way to end
those other sessions from the app. The backend now exposes `AuthService.RevokeAllSessions`
(change: `add-session-management`) — a self "sign out everywhere" that revokes **all** of
the account's refresh sessions server-side. This surfaces it in the mobile app as a
single account action, which is where self-service session management belongs (the
admin-only back office deliberately does not carry it).

## What Changes

- **Account menu action "Sign out from all devices"** in the Flutter app: a confirmation
  dialog, then a call to `AuthService.RevokeAllSessions`.
- Because revoking **all** sessions kills the **current** device's refresh token too,
  on success the app performs the **same local teardown as a normal sign-out** (clear
  the secure-storage tokens, OIDC sign-out, route back to the entry screen). Unlike the
  best-effort local logout, sign-out-everywhere tears down the local session **only after
  the revoke succeeds** — a failed revoke keeps the user signed in and surfaces the error.
- Wire `revokeAllSessions()` through the existing `AuthService` seam
  (`GrpcAuthService` over the generated `AuthServiceClient`, authenticated call) and add
  a `SessionNotifier.signOutEverywhere()` Riverpod action.
- Regenerate the Dart gRPC stubs so `RevokeAllSessions` is available.

## Capabilities

### New Capabilities
- `mobile-session-signout`: a mobile account action to sign out of **all** the user's
  sessions (self), calling `AuthService.RevokeAllSessions` and then tearing down the
  local session. A single button — no session listing or per-device revoke.

### Modified Capabilities
<!-- None. Consumes the `session-management` capability's RevokeAllSessions RPC
     (add-session-management); no change to sign-in / refresh / token semantics. -->

## Impact

- **Depends on** `add-session-management` (the `RevokeAllSessions` RPC + regenerated
  protos). This change is UI + a thin service method only — no backend work.
- **App (`apps/music`)**: `AuthService` seam + `GrpcAuthService` (`revokeAllSessions`),
  `SessionNotifier.signOutEverywhere()` (Riverpod 2 + Freezed, per the repo CLAUDE.md),
  the `account_menu` item + confirm dialog, i18n (`app_*.arb`), regenerated gRPC stubs.
- **Tokens** stay in platform secure storage (`token_store`); no new storage.
- **Tests**: `SessionNotifier` unit test (fake `AuthService` records the revoke; tokens
  cleared on success; a failed revoke keeps the session) + a widget test for the menu
  item + confirmation.
