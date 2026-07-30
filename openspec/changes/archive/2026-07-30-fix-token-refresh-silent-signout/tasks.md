## 1. Shared token refresher seam

- [x] 1.1 Add `RefreshOutcome` sealed result (`Refreshed(accessToken)` / `Rejected` / `Transient`) and a `TokenRefresher` abstract seam in `services/token_refresher.dart`, with a `@Riverpod(keepAlive: true) tokenRefresher` provider.
- [x] 1.2 Implement `CoordinatedTokenRefresher`: single-flight (`Future<RefreshOutcome>? _inFlight`, awaited by concurrent callers, cleared on completion); reads the stored refresh token via `TokenStore`, calls `AuthService.refresh`, writes the rotated pair on success.
- [x] 1.3 Classify failures: `AuthError.unauthenticated` / `invalidArgument` ⇒ `Rejected` + `TokenStore.clear()`; every other failure (unavailable, deadline exceeded, unknown, offline) ⇒ `Transient`, leaving the store intact.

## 2. Rewire `authedCall`

- [x] 2.1 Change `authedCall<T>` in `services/grpc_client.dart` to take a single `Future<RefreshOutcome> Function() refresh` (drop `refreshAccessToken` + the no-op `onExpired`).
- [x] 2.2 Map the outcome: `Refreshed` ⇒ retry the call once with the new token; `Rejected` ⇒ propagate the original `UNAUTHENTICATED`; `Transient` ⇒ throw a non-`UNAUTHENTICATED` gRPC error so the session bootstrap keeps the user signed in.

## 3. Route every adapter through the shared refresher

- [x] 3.1 `GrpcAuthService` and `GrpcAccountService` (`services/grpc_client.dart`): remove both private `_refreshAccess` copies; inject `TokenRefresher` and use it in `_authed`.
- [x] 3.2 Replace the duplicated `_refreshAccess` in `services/play_sync_service.dart`, `services/profile_service.dart`, `services/catalog_service.dart`, `services/score_upload_service.dart`, and `services/rating_service.dart` with the injected `TokenRefresher`.
- [x] 3.3 Update the service providers (`accountService`, `authService`, and the five siblings) to pass `ref.watch(tokenRefresherProvider)`; wire the refresher's `AuthService` dependency without a provider cycle (auth service does not depend on the refresher).

## 4. Session bootstrap keeps the user signed in on transient failure

- [x] 4.1 Verify/adjust `state/session_notifier.dart` `_resolveAuthenticated` so a transient refresh outcome (now surfaced as a non-`UNAUTHENTICATED` `AuthException`) routes to `authenticated()` (unknown account), not `unauthenticated()`.

## 5. Tests (mockito doubles, ≥80% coverage)

- [x] 5.1 `CoordinatedTokenRefresher`: N concurrent callers ⇒ exactly one `AuthService.refresh` call, all receive the same `Refreshed` token (verify with a mockito mock + a completer-gated refresh).
- [x] 5.2 Refresher classification: `UNAUTHENTICATED`/`INVALID_ARGUMENT` ⇒ `Rejected` and `TokenStore.clear()` called; `UNAVAILABLE`/timeout ⇒ `Transient` and store untouched.
- [x] 5.3 `authedCall`: `Refreshed` retries once; `Rejected` rethrows `UNAUTHENTICATED`; `Transient` throws a non-`UNAUTHENTICATED` error.
- [x] 5.4 `session_notifier_test`: expired access token + transient refresh at launch ⇒ stays `authenticated`; refresh rejection ⇒ `unauthenticated` and tokens cleared.
- [x] 5.5 Regression guard: a transient refresh failure never calls `TokenStore.clear()`.

## 6. Verify

- [x] 6.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`.
- [x] 6.2 `melos run analyze` + `dart run custom_lint` + `dart format` clean.
- [x] 6.3 `flutter test --coverage --exclude-tags golden` green; line coverage ≥ 80%.
- [x] 6.4 `openspec validate fix-token-refresh-silent-signout --strict` passes.
