## 0. Backend: `SetLocalCredential` RPC (design D8)

- [x] 0.1 Add `SetLocalCredential(email, password)` to `auth.proto` (+ regenerate Rust via build.rs and Dart via `melos run gen-grpc`)
- [x] 0.2 Add `set_local_credential(user_id, email, password)` to the `AuthPort` trait and implement it in `AuthModule` (guard one local per account; insert credential → link `(local, email)` identity with compensating erase on failure → enqueue verification email; unverified until confirmed)
- [x] 0.3 Wire the gRPC server adapter handler (`caller` from the access token) in `backend/auth/src/grpc.rs`
- [x] 0.4 Rust tests: add-then-verify-enables-sign-in (same account), second-password refused, email-owned-elsewhere refused without wiping the other account (≥80% coverage)

## 1. Service seams + gRPC adapters

- [x] 1.1 Add `listIdentities()` to the `AccountService` seam returning a Freezed `LinkedIdentity` list (`provider`, `subject`, `linkedAt`); implement in `GrpcAccountService` via the generated `ListIdentities` stub
- [x] 1.2 Add `linkIdentity(idToken)`, `unlinkIdentity(provider, subject)` and `setLocalCredential(email, password)` to the `AuthService` seam; implement in `GrpcAuthService` via the generated `LinkIdentity`/`UnlinkIdentity`/`SetLocalCredential` stubs (bearer from the current session)
- [x] 1.3 Map gRPC statuses to typed errors for the new calls (`ALREADY_EXISTS` → already-linked-elsewhere, `FAILED_PRECONDITION` → last-identity, `UNAUTHENTICATED` → re-auth/sign-in). NOTE: `FAILED_PRECONDITION` is overloaded — the default `authErrorMessage` mapping is `authErrUnverified` (email not verified). The unlink call site MUST pass an explicit "only sign-in method" fallback so a refused unlink never reads "email not verified" (design D5).
- [x] 1.4 Extend the in-memory fakes (fake auth + account services / OIDC source) to cover list/link/unlink success and each error

## 2. State (Riverpod 2 + Freezed)

- [x] 2.1 Freezed `ConnectedAccountsState` (identities list, loading, per-action status/error)
- [x] 2.2 `@riverpod` `ConnectedAccountsNotifier`: load via `listIdentities`; `linkGoogle`/`linkApple` (mint `id_token` via `OidcTokenSource` → `linkIdentity` → refetch); `linkEmailPassword`; `unlink(provider, subject)` (refetch); compute "is last identity" and which providers are not yet linked
- [x] 2.3 Unit tests for the notifier: load, link success/cancel, `ALREADY_EXISTS`, unlink success, last-identity blocked, refetch-after-mutation

## 3. UI

- [x] 3.1 `ConnectedAccountsScreen`: list rows (provider + linked-at), per-row unlink (disabled for the last identity with explanation), and link actions for providers not yet present ("Link Google" / "Link Apple" / "Set a password")
- [x] 3.2 Add the entry point from account settings; ensure it is unreachable in guest mode
- [x] 3.3 "Set a password" sub-flow (email + password) wired to `linkEmailPassword`
- [x] 3.4 Widget tests: list render, link tap → success refresh, collision error shown, last-identity unlink disabled, guest has no access

## 4. Error messaging fix

- [x] 4.1 In `auth_messages.dart`, stop mapping all `UNAUTHENTICATED` to "Incorrect email or password."; keep that copy only for the local sign-in flow via an explicit fallback
- [x] 4.2 Add link/unlink-specific messages (link failed, already linked to another account, can't remove only sign-in method) and route OIDC sign-in failures to provider-appropriate copy
- [x] 4.3 Tests asserting the local sign-in still reads "Incorrect email or password." while OIDC/link failures read their own messages

## 5. Sign-in collision: user-driven "sign in to link" (design D7)

> Prerequisite `fix-handle-onboarding-escape` is **archived** (shipping in the
> `handle-onboarding` capability): the escape action and the `DeleteAccount`
> orphan-cleanup path this section reuses already exist.

- [x] 5.1 On the handle-onboarding screen, add an "Already have an account? Sign in to link" option next to the existing escape action (from `handle-onboarding`) (no claim that an account exists; no method disclosed)
- [x] 5.2 Collision-link flow: prompt the user to choose+authenticate an existing method → delete the orphan social account (reuse the abandon/delete path) → sign in to the existing account → `LinkIdentity(socialIdToken)`; enforce delete-before-link ordering
- [x] 5.3 Re-mint the social `id_token` via `OidcTokenSource` if it expired before `LinkIdentity` runs
- [x] 5.4 Tests: happy path (Google→email account, ends on existing account, no new handle/account), wrong existing credentials, and the expired-token re-mint branch

## 6. Re-auth decision (design D6 / Open Question)

- [x] 6.1 Confirmed the v1 decision: **no re-auth gate** for link/unlink (design D6). Linking is additive; the last-identity guard prevents lockout from unlinking; unlink is behind a confirm dialog. Re-auth gating stays an easy follow-up.

## 7. Validation

- [x] 7.1 `flutter analyze` + `dart run custom_lint` clean; `dart format` clean; Rust `cargo fmt --check` + `cargo clippy` clean
- [x] 7.2 `flutter test --exclude-tags golden` green (611 tests); Rust `cargo test -p cymbra-auth` green (26 tests). New code covered: notifier (unit), screen (widget), collision flow (auth_flow), messaging, backend module tests; thin gRPC adapters coverage-excluded as today
- [x] 7.3 Manual smoke on macOS: link Google to an email account, observe `ALREADY_EXISTS` path, unlink, last-identity guard, and the sign-in collision "sign in to link" flow (reuse the grpcurl `link_identity_test.sh` as the backend cross-check) — verified
- [x] 7.4 `openspec validate add-account-identity-linking --strict` passes
