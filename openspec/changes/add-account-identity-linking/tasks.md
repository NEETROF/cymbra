## 1. Service seams + gRPC adapters

- [ ] 1.1 Add `listIdentities()` to the `AccountService` seam returning a Freezed `LinkedIdentity` list (`provider`, `subject`, `linkedAt`); implement in `GrpcAccountService` via the generated `ListIdentities` stub
- [ ] 1.2 Add `linkIdentity(idToken)` and `unlinkIdentity(provider, subject)` to the `AuthService` seam; implement in `GrpcAuthService` via the generated `LinkIdentity`/`UnlinkIdentity` stubs (bearer from the current session)
- [ ] 1.3 Map gRPC statuses to typed errors for the new calls (`ALREADY_EXISTS` → already-linked-elsewhere, `FAILED_PRECONDITION` → last-identity, `UNAUTHENTICATED` → re-auth/sign-in)
- [ ] 1.4 Extend the in-memory fakes (fake auth + account services / OIDC source) to cover list/link/unlink success and each error

## 2. State (Riverpod 2 + Freezed)

- [ ] 2.1 Freezed `ConnectedAccountsState` (identities list, loading, per-action status/error)
- [ ] 2.2 `@riverpod` `ConnectedAccountsNotifier`: load via `listIdentities`; `linkGoogle`/`linkApple` (mint `id_token` via `OidcTokenSource` → `linkIdentity` → refetch); `linkEmailPassword`; `unlink(provider, subject)` (refetch); compute "is last identity" and which providers are not yet linked
- [ ] 2.3 Unit tests for the notifier: load, link success/cancel, `ALREADY_EXISTS`, unlink success, last-identity blocked, refetch-after-mutation

## 3. UI

- [ ] 3.1 `ConnectedAccountsScreen`: list rows (provider + linked-at), per-row unlink (disabled for the last identity with explanation), and link actions for providers not yet present ("Link Google" / "Link Apple" / "Set a password")
- [ ] 3.2 Add the entry point from account settings; ensure it is unreachable in guest mode
- [ ] 3.3 "Set a password" sub-flow (email + password) wired to `linkEmailPassword`
- [ ] 3.4 Widget tests: list render, link tap → success refresh, collision error shown, last-identity unlink disabled, guest has no access

## 4. Error messaging fix

- [ ] 4.1 In `auth_messages.dart`, stop mapping all `UNAUTHENTICATED` to "Incorrect email or password."; keep that copy only for the local sign-in flow via an explicit fallback
- [ ] 4.2 Add link/unlink-specific messages (link failed, already linked to another account, can't remove only sign-in method) and route OIDC sign-in failures to provider-appropriate copy
- [ ] 4.3 Tests asserting the local sign-in still reads "Incorrect email or password." while OIDC/link failures read their own messages

## 5. Sign-in collision: user-driven "sign in to link" (design D7)

- [ ] 5.1 On the handle-onboarding screen, add an "Already have an account? Sign in to link" option next to the escape action from `fix-handle-onboarding-escape` (no claim that an account exists; no method disclosed)
- [ ] 5.2 Collision-link flow: prompt the user to choose+authenticate an existing method → delete the orphan social account (reuse the abandon/delete path) → sign in to the existing account → `LinkIdentity(socialIdToken)`; enforce delete-before-link ordering
- [ ] 5.3 Re-mint the social `id_token` via `OidcTokenSource` if it expired before `LinkIdentity` runs
- [ ] 5.4 Tests: happy path (Google→email account, ends on existing account, no new handle/account), wrong existing credentials, and the expired-token re-mint branch

## 6. Re-auth decision (design D6 / Open Question)

- [ ] 6.1 Confirm the v1 decision (no re-auth gate for link/unlink) or, if reversed, add a recent-auth prompt before unlink mirroring the delete-account flow; document the outcome in design.md

## 7. Validation

- [ ] 7.1 `flutter analyze` + `dart run custom_lint` clean; `dart format` clean
- [ ] 7.2 `flutter test --coverage --exclude-tags golden` green and overall line coverage ≥ 80%
- [ ] 7.3 Manual smoke on macOS: link Google to an email account, observe `ALREADY_EXISTS` path, unlink, last-identity guard, and the sign-in collision "sign in to link" flow (reuse the grpcurl `link_identity_test.sh` as the backend cross-check)
- [ ] 7.4 `openspec validate add-account-identity-linking --strict` passes
