## Why

Users are randomly signed out at launch — dropped to the entry screen with no
message, forced to sign in with Apple again. Two defects in the token-refresh
path cause it, and both end at the same silent `TokenStore.clear()`:

1. **Transient refresh failures are treated as terminal.** `_refreshAccess`
   clears the whole session on *any* exception from `Refresh` — including a
   network timeout / `UNAVAILABLE` on a weak or switching Wi-Fi. When the access
   token is already expired at launch, this resurfaces as `UNAUTHENTICATED` and
   defeats the existing "offline keeps the user signed in" guard.
2. **Uncoordinated concurrent refreshes.** The `_refreshAccess` logic is
   duplicated in six service adapters with no single-flight. When the library
   and play-sync providers fan out authenticated calls at launch with an expired
   access token, several calls replay the *same* refresh token. The backend
   rotates once and treats the replays as reuse, **revoking the entire session
   family** — so even the winner's fresh token is dead, and every adapter's
   `catch` clears the local session.

Both are more likely on iPad (larger library → wider concurrency window) and on
flaky networks, matching the reported "random, on launch, no message, must
re-login with Apple".

## What Changes

- Introduce a single **coordinated token refresher** shared by every
  authenticated service: at most one in-flight `Refresh` per session; concurrent
  callers await the same result instead of each replaying the stored refresh
  token.
- **Classify refresh failures**: only a genuine refresh-token rejection
  (`UNAUTHENTICATED` / `INVALID_ARGUMENT`) clears the local session. A transient
  failure (`UNAVAILABLE`, deadline exceeded, offline) **keeps** the session and
  leaves the stored tokens intact so a later launch/online retry recovers.
- Ensure a needed-but-transiently-failed refresh no longer masquerades as
  `UNAUTHENTICATED` to the session bootstrap, so `_resolveAuthenticated` keeps
  the user signed in (unknown account) on offline instead of routing to entry.
- Replace the six duplicated `_refreshAccess` copies (grpc auth + account,
  play-sync, profile, catalog, score-upload, rating) with the shared refresher.

No user-facing UI, no backend, and no wire-protocol change.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `account-access`: the **Silent token refresh** requirement gains
  concurrency-coordination and failure-classification behavior — a single
  in-flight refresh per session, and clearing the session only on a real
  refresh-token rejection (never on a transient/offline failure).

## Impact

- **Code (Flutter, `apps/music`)**:
  - New shared refresher seam (e.g. `services/token_refresher.dart`) + Riverpod
    provider.
  - `services/grpc_client.dart` (`authedCall`, `GrpcAuthService._refreshAccess`,
    `GrpcAccountService._refreshAccess`), `services/play_sync_service.dart`,
    `services/profile_service.dart`, `services/catalog_service.dart`,
    `services/score_upload_service.dart`, `services/rating_service.dart` — route
    refresh through the shared refresher.
  - `state/session_notifier.dart` (`_resolveAuthenticated`) — keep the session
    on a transient refresh outcome.
- **Tests**: unit coverage for single-flight (N concurrent callers ⇒ one
  `Refresh`), transient-failure-keeps-session, terminal-rejection-clears-session,
  and a session-notifier test that offline-at-launch stays signed in.
- **No change** to backend, gRPC contracts, storage keys, or Apple/Google flows.
