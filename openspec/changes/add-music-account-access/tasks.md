## 1. Backend: unique handle (cross-tier prerequisite)

- [x] 1.1 Add a `handle` column to the user-account model + a DB migration; enforce a **case-insensitive** UNIQUE constraint (functional unique index on a case-folded form, or a generated normalized column)
- [x] 1.2 Validate the handle policy server-side: 1–15 UTF-8 letters/numbers only; store the display form, compare on a normalized form (Unicode NFC + case-fold) for uniqueness
- [x] 1.3 Accept and validate `handle` on `UpdateAccount`; map the unique-violation to a gRPC conflict (`ALREADY_EXISTS`/`ABORTED`)
- [x] 1.4 Add `CheckHandleAvailability(handle) -> { available }` to `user.proto` and implement it in the user module (advisory only)
- [x] 1.5 Expose `handle` on the `Account` message returned by `GetAccount`
- [x] 1.6 Backend tests for uniqueness at write time + availability check; keep coverage ≥80% and `cargo fmt`/`clippy` clean

## 2. App: dependencies & gRPC codegen

- [x] 2.1 Add `grpc`, `protobuf`, `flutter_secure_storage`, `google_sign_in`, `sign_in_with_apple` to `apps/music/pubspec.yaml`; add `protoc_plugin` for codegen
- [x] 2.2 Add a Melos/script step that generates Dart stubs from `backend/**/proto/*.proto` into `lib/src/grpc/` (gitignored, like `lib/src/rust/`)
- [x] 2.3 Wire the codegen step into CI before analyze/test
- [x] 2.4 Exclude the generated gRPC path from the `very_good_coverage` config (mirror the `lib/src/rust/**` exclusion)

## 3. App: service & session seams (no UI)

- [x] 3.1 Define `TokenStore` seam over `flutter_secure_storage` (read/write/clear token pair + guest flag); `@riverpod` provider
- [x] 3.2 Define abstract `AuthService` (sign-up, verify, resend, sign-in local, sign-in OIDC, refresh, logout, request reset, reset) with a gRPC-backed impl behind `@riverpod`
- [x] 3.3 Define abstract `AccountService` (`getAccount`, `updateAccount`/handle, `checkHandleAvailability`, `deleteAccount`) with gRPC-backed impl behind `@riverpod`
- [x] 3.4 Implement a gRPC interceptor: inject `Bearer` access token, on `UNAUTHENTICATED` do a single `Refresh` + retry, clear session on refresh failure
- [x] 3.5 Add `SessionState` (`@freezed`: `unknown | guest | authenticated | unauthenticated`) and a `SessionNotifier` (`@riverpod`) that hydrates from `TokenStore` at startup and resolves state
- [x] 3.6 Implement guest gating: a `requiresAccount` guard/provider that, for guests, signals "prompt sign-in" instead of calling the backend
- [x] 3.7 Add fakes (`FakeAuthService`, `FakeAccountService`, in-memory `TokenStore`) in `test/support/`; unit tests for `SessionNotifier`, interceptor refresh/retry, and guest gating

## 4. App: launch gating & entry screen

- [x] 4.1 Convert `CymbraApp` to `ConsumerWidget`; drive `home` from `SessionState` (entry vs `LibraryScreen`, with a loading state for `unknown`)
- [x] 4.2 Build the themed account entry screen (`lib/screens/auth/entry_screen.dart`) with the four options, using `CymbraColors`/Material 3
- [x] 4.3 Implement guest path end-to-end: persist choice, open library, expose "leave guest / sign in" affordance
- [x] 4.4 Widget tests: entry renders four options; guest choice routes to library and is persisted; returning session/guest skips entry

## 5. App: email account flow

- [x] 5.1 Email sign-up screen → `SignUpLocal`; client-side password-policy check; handle `ALREADY_EXISTS`
- [x] 5.2 OTP verification screen → `VerifyEmail` + resend via `ResendVerification`; surface `RESOURCE_EXHAUSTED` and invalid/expired code
- [x] 5.3 Email sign-in screen → `SignInLocal(audience="music")`; distinguish wrong-credential, lockout, and unverified (`FAILED_PRECONDITION` → route to verify)
- [x] 5.4 Forgot-password flow → `RequestPasswordReset` (no-enumeration UX) then `ResetPassword`; inform user sessions are signed out
- [x] 5.5 Widget/notifier tests for sign-up, verify+resend, sign-in error states, and reset request+complete

## 6. App: Google & Apple sign-in

- [x] 6.1 Integrate `google_sign_in`: obtain `id_token`, call `SignInOidc(audience="music")`; handle user-cancel as no-op
- [x] 6.2 Integrate `sign_in_with_apple`: obtain `id_token`, call `SignInOidc(audience="music")`; offer on Apple platforms wherever Google is offered
- [ ] 6.3 Platform config: Google OAuth client IDs (iOS/Android/macOS) + iOS URL schemes / Android intent filters; Apple "Sign in with Apple" capability — SCAFFOLDED (buttons gated behind `--dart-define` GOOGLE_CLIENT_ID/APPLE_SIGN_IN_ENABLED; reversed-client-id URL scheme placeholders in iOS/macOS Info.plist; README documents the steps). BLOCKED on the real OAuth client IDs + Apple capability (needs a dev certificate) to finish.
  - [x] 6.3a **macOS** build-time injection (no secret committed): Info.plist URL scheme uses `$(GOOGLE_OAUTH_CLIENT_SUFFIX)` resolved from `Configs/AppInfo.xcconfig` (inert default) overridden by gitignored `Configs/Secrets.xcconfig` (template: `Secrets.example.xcconfig`); the `release-build.yml` macOS job writes it from the `GOOGLE_CLIENT_ID` secret and passes `--dart-define`. Needs the `GOOGLE_CLIENT_ID` repo secret set.
  - [ ] 6.3b **iOS**: replicate the macOS `Secrets.xcconfig` injection in `ios/Runner/Info.plist` (still a literal placeholder); deferred until signed iOS builds land in CI.
  - [ ] 6.3c **Android**: wire `serverClientId` (web OAuth client) for `google_sign_in` — NOT a reversed-client-id intent filter; needs its own client + secret injection.
- [ ] 6.4 Coordinate backend `CYMBRA_GOOGLE_AUDIENCE`/`CYMBRA_APPLE_AUDIENCE` with the registered client IDs — BLOCKED: depends on 6.3 credentials
- [x] 6.5 Tests with a fake OIDC token source covering success and cancellation

## 7. App: handle onboarding, sign-out, deletion

- [x] 7.1 Post-auth gate: after every successful sign-in, `GetAccount`; route to handle onboarding when handle is null
- [x] 7.2 Handle onboarding screen: client-side policy validation (1–15 letters/numbers), debounced `checkHandleAvailability`, validity feedback, commit via `UpdateAccount`; treat write-time conflict (incl. case-insensitive) as "pick another"
- [x] 7.3 Sign-out: call `Logout`, clear `TokenStore`, return to entry; still clear locally if `Logout` is offline
- [x] 7.4 Account deletion: re-authentication gate (password via `SignInLocal`, or re-run OIDC for fresh `id_token`) → irreversible confirmation → `DeleteAccount`, clear session, return to entry; hidden in guest mode
- [x] 7.5 Tests: handle format reject + free/taken/invalid + case-insensitive write-time conflict; sign-out online/offline; deletion re-auth success/failure, confirm/cancel, and guest-hidden

## 8. Integration, coverage & docs

- [ ] 8.1 Integration smoke test against the `mock-oidc` compose profile: guest, email sign-up→verify→sign-in→handle, Google/Apple stub sign-in, reset, delete — BLOCKED: requires the backend + `mock-oidc` compose stack running (not available in this environment); unit/widget coverage of every flow is in place via fakes
- [x] 8.2 Confirm Flutter + Rust coverage ≥80%; generated gRPC excluded; `melos run analyze`, `dart format`, `cargo fmt`/`clippy` clean
- [x] 8.3 Document the dev setup (backend endpoint, proto codegen, OAuth client IDs, mock-oidc) in the app README/CONTRIBUTING
- [x] 8.4 Run `openspec validate add-music-account-access --strict` and address findings
