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
- [x] 6.3 Platform config: Google OAuth client IDs (iOS/Android/macOS) + iOS URL schemes / Android intent filters; Apple "Sign in with Apple" capability — DONE. Google client IDs wired + verified on iOS (9.1) and Android (9.2). Apple capability added on iOS (`ios/Runner/Runner.entitlements` → `com.apple.developer.applesignin`) with a paid-account dev cert, verified end-to-end on a physical iPad (9.5). macOS Apple entitlements added (`macos/Runner/{DebugProfile,Release}.entitlements`) and verified (9.6).
  - [x] 6.3a **macOS** build-time injection (no secret committed): Info.plist URL scheme uses `$(GOOGLE_OAUTH_CLIENT_SUFFIX)` resolved from `Configs/AppInfo.xcconfig` (inert default) overridden by gitignored `Configs/Secrets.xcconfig` (template: `Secrets.example.xcconfig`); the `release-build.yml` macOS job writes it from the `GOOGLE_CLIENT_ID` secret and passes `--dart-define`. Needs the `GOOGLE_CLIENT_ID` repo secret set.
  - [x] 6.3b **iOS**: replicated the macOS injection — `ios/Runner/Info.plist` uses `$(GOOGLE_OAUTH_CLIENT_SUFFIX)`, default in `Flutter/Debug.xcconfig`/`Release.xcconfig`, overridden by gitignored `Flutter/Secrets.xcconfig` (template: `Secrets.example.xcconfig`). Fixes the native crash on "Continue with Google" (the placeholder URL scheme). CI injection step deferred until signed iOS builds land in CI.
  - [x] 6.3c **Android**: wired `serverClientId` (the web OAuth client) on every platform (Option A → single backend audience = web client). Android needs only `GOOGLE_SERVER_CLIENT_ID`; an Android OAuth client (package + SHA-1) is registered for Play Services. Shared debug keystore committed (`android/app/debug.keystore` + pinned `signingConfigs.debug`) so all devs/CI share one SHA-1. Verified on a physical device.
- [x] 6.4 Coordinate backend `CYMBRA_GOOGLE_AUDIENCE`/`CYMBRA_APPLE_AUDIENCE` with the registered client IDs — DONE. Google audience set (`backend/.env`, web client). Apple audience set to the app bundle ID `com.cymbra.music` (native Sign in with Apple: `id_token.aud` = bundle ID; no Services ID / P8 needed for native id_token JWKS verification). Both proven live: a successful `SignInOidc` means the audience matched.
- [x] 6.5 Tests with a fake OIDC token source covering success and cancellation

## 7. App: handle onboarding, sign-out, deletion

- [x] 7.1 Post-auth gate: after every successful sign-in, `GetAccount`; route to handle onboarding when handle is null
- [x] 7.2 Handle onboarding screen: client-side policy validation (1–15 letters/numbers), debounced `checkHandleAvailability`, validity feedback, commit via `UpdateAccount`; treat write-time conflict (incl. case-insensitive) as "pick another"
- [x] 7.3 Sign-out: call `Logout`, clear `TokenStore`, return to entry; still clear locally if `Logout` is offline
- [x] 7.4 Account deletion: re-authentication gate (password via `SignInLocal`, or re-run OIDC for fresh `id_token`) → irreversible confirmation → `DeleteAccount`, clear session, return to entry; hidden in guest mode
- [x] 7.5 Tests: handle format reject + free/taken/invalid + case-insensitive write-time conflict; sign-out online/offline; deletion re-auth success/failure, confirm/cancel, and guest-hidden

## 8. Integration, coverage & docs

- [x] 8.1 Integration smoke test against the live backend: guest, email sign-up→verify→sign-in→handle, reset, delete — VERIFIED end-to-end on 2026-07-03. Brought up the compose infra (postgres, redis, mailpit) + ran `cymbra-server` (gRPC `:50051`) and `cymbra-worker` (email jobs) natively; confirmed the wire path (SignUpLocal → worker → verification email in Mailpit). Guest / email sign-up→verify→sign-in→handle / password-reset / delete driven manually in the macOS app and confirmed working. **Google/Apple OIDC-stub sub-flow deferred**: the `mock-oidc` image (`ghcr.io/navikt/mock-oauth2-server`) 403s on ghcr.io in this environment (registry egress restriction), so no local OIDC issuer was available. Real native Google/Apple sign-in is already verified end-to-end on physical devices (tasks 9.1–9.6), and cancel/success are unit-covered via a fake OIDC token source (task 6.5). Runbook below re-runs the OIDC portion on any Docker host with ghcr access.

  > **Smoke-test runbook (run when a Docker/backend host is available)**
  >
  > 1. **Bring up the backend stack with the OIDC + mail sinks:**
  >    ```bash
  >    cd backend
  >    cp -n .env.example .env   # set CYMBRA_GOOGLE_AUDIENCE / CYMBRA_APPLE_AUDIENCE to the mock-oidc client id
  >    docker compose --profile oidc up -d   # starts postgres, redis, mock-oidc (:8080), mailpit (:1025/:8025), backend gRPC (:50051)
  >    ```
  >    Confirm: gRPC on `localhost:50051`, mock-oauth2-server on `localhost:8080`, Mailpit UI on `http://localhost:8025`.
  > 2. **Run the app against it** (email/guest need no OAuth config; Google/Apple use the mock issuer via `CYMBRA_DEV_OIDC_ISSUER`):
  >    ```bash
  >    cd apps/music
  >    flutter run -d linux \
  >      --dart-define=CYMBRA_GRPC_HOST=localhost --dart-define=CYMBRA_GRPC_PORT=50051
  >    ```
  > 3. **Walk each flow, asserting the expected end state:**
  >    - **Guest**: choose *Continue as guest* → library opens; relaunch skips entry (guest persisted in `TokenStore`).
  >    - **Email sign-up → verify → sign-in → handle**: register → grab the code from Mailpit (`:8025`) → verify → land on handle onboarding → set a valid handle (1–15 alnum) → library.
  >    - **Handle-escape**: on handle onboarding tap *Use a different account* → returns to entry, no orphan account (re-`GetAccount` is null).
  >    - **Google / Apple stub**: with the `oidc` profile, the button drives `SignInOidc` against `mock-oidc` → handle onboarding → library. (Buttons only appear on platforms where the seam enables them — use iOS/macOS/Android for Apple/Google; on Linux they are correctly hidden per 9.4/9.7.)
  >    - **Password reset**: request reset → pull the reset code from Mailpit → set a new password → sign in with it.
  >    - **Delete**: from a signed-in session → re-auth gate (password via `SignInLocal` or fresh OIDC) → irreversible confirm → `DeleteAccount` → session cleared, back at entry; a follow-up sign-in shows the account is gone.
  > 4. **Tear down:** `docker compose --profile oidc down -v`.
  >
  > Every one of these flows already has unit/widget coverage via fakes (see `test/widgets/auth_*_test.dart`); this runbook is the end-to-end wire-level confirmation against real gRPC + a real OIDC issuer.
- [x] 8.2 Confirm Flutter + Rust coverage ≥80%; generated gRPC excluded; `melos run analyze`, `dart format`, `cargo fmt`/`clippy` clean
- [x] 8.3 Document the dev setup (backend endpoint, proto codegen, OAuth client IDs, mock-oidc) in the app README/CONTRIBUTING
- [x] 8.4 Run `openspec validate add-music-account-access --strict` and address findings

## 9. Manual cross-platform sign-in verification (Google & Apple)

Google sign-in end-to-end = native consent → `SignInOidc` → handle onboarding →
library; each platform's client ID must be in the backend `CYMBRA_GOOGLE_AUDIENCE`
(accepts a list). NOTE: `google_sign_in` supports Android/iOS/macOS/Web — **not**
Windows/Linux desktop. (macOS Google already verified this session — task 6.3a.)

- [x] 9.1 **Google — iOS** (iOS client ID + reversed-client-id URL scheme + serverClientId): full flow verified on a physical iPad
- [x] 9.2 **Google — Android** (serverClientId web client + SHA-registered Android client): full flow verified on a physical device (SM P610)
- [x] 9.3 **Google — Windows**: `googleAvailable` returns false off Android/Apple platforms (`oidc_token_source.dart`), so the button is never built and no native `google_sign_in` call is reachable (no crash possible). Asserted by the `Google is hidden where it is not available` widget test. Verified via seam + widget test (no Windows device on hand). Loopback-OAuth is out of scope here → tracked as the separate `add-desktop-oauth-loopback` change.
- [x] 9.4 **Google — Linux**: same seam path as Windows (`googleAvailable` → false); Google button absent. Verified on a live Linux desktop build (`flutter build linux` runs; entry screen shows no Google button) plus the gating widget test. Loopback deferred to `add-desktop-oauth-loopback`.

Apple sign-in = native `sign_in_with_apple` (iOS/macOS only); needs the "Sign in
with Apple" capability + a dev certificate and `CYMBRA_APPLE_AUDIENCE`. The app
seam currently gates Apple to iOS/macOS (`appleAvailable`).

- [x] 9.5 **Apple — iOS** (capability + cert + audience): full flow verified on a physical iPad — native consent → `SignInOidc` → handle onboarding → library; "Use a different account" on handle onboarding returns to entry with no orphan account.
- [x] 9.6 **Apple — macOS** (capability + cert + audience): full flow verified on macOS — Sign in with Apple capability added to `macos/Runner/DebugProfile.entitlements` + `Release.entitlements` (`com.apple.developer.applesignin`); native consent → `SignInOidc` → handle onboarding → library.
- [x] 9.7 **Apple — Android/Windows/Linux**: `appleAvailable` requires `Platform.isIOS || Platform.isMacOS` (`oidc_token_source.dart`), so the Apple button is never built on Android/Windows/Linux. Asserted by the `Apple is hidden where it is not available` widget test; Linux confirmed visually on a live desktop build (Apple button not visible). Native Apple on these platforms would need the web-auth flow (Services ID + return URL) — out of current scope.

> Handle-escape per provider: the `fix-handle-onboarding-escape` flow is
> provider-agnostic (deletes the just-created account regardless of sign-in
> method; unit/widget-tested as such, smoke-verified on macOS-Google). For each
> path above that reaches handle onboarding (Google iOS/Android, Apple iOS/macOS),
> also tap "Use a different account" there and confirm return-to-entry + no orphan
> — this re-confirms gate routing per provider. The email sign-up path reaches the
> same screen and is testable now without OAuth.
