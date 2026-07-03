## Why

The Cymbra Music app is fully offline today and has no notion of a user. The new
**Cymbra ID** backend (gRPC, audience-scoped sessions) is live, so the app needs
an account layer to unlock future online services (sync, sharing, multi-device).
We want the very first thing a new user sees to be a welcoming, on-brand entry
screen that lets them sign in with Google, Apple, or email — or skip entirely and
try the app with no account. Doing this now, before any online feature ships,
lets us build those features behind a clean, already-authenticated seam instead
of retrofitting auth later.

## What Changes

- **New launch screen = account entry.** On first launch (or when there is no
  valid session) the app opens on a themed entry screen with four choices:
  *Continue with Google*, *Continue with Apple*, *Continue with email*, and
  *Continue without an account (guest)*.
- **Three online sign-in methods** against Cymbra ID's gRPC `AuthService`, all
  scoped to audience `music`:
  - **Email + password**: sign-up → email verification (OTP code) → sign-in.
  - **Google**: native `google_sign_in` → send `id_token` to `SignInOidc`.
  - **Apple**: native `sign_in_with_apple` → send `id_token` to `SignInOidc`.
- **Unique-handle onboarding.** After *any* successful sign-in, if the account
  has no handle yet (new user), the app forces a "choose your handle" screen with
  live availability checking. The handle is **unique** — this requires a backend
  evolution (see Impact).
- **Guest mode.** A local-only session with **no** calls to Cymbra ID and no
  access to any online (backend-bound) service. The guest/account choice is
  persisted so the entry screen is not re-imposed on every launch.
- **Account lifecycle**: email verification + resend, *forgot password* (OTP
  reset), sign-out (`Logout`), and **account deletion** (`DeleteAccount`, with
  explicit confirmation).
- **Session plumbing**: secure token storage (`flutter_secure_storage`), silent
  refresh on `UNAUTHENTICATED`/expiry via `Refresh`, and a returning user with a
  valid session skips the entry screen entirely.
- **New networking layer**: native gRPC client (`grpc` + `protobuf`) generated
  from Cymbra ID's `auth.proto` / `user.proto`, behind an injectable Riverpod
  service seam so widgets/state stay testable without the backend. **BREAKING**
  for the app's startup contract: `LibraryScreen` is no longer the unconditional
  home.

## Capabilities

### New Capabilities
- `account-access`: The app's authentication entry experience and session
  lifecycle — the launch entry screen, the four entry points (Google, Apple,
  email, guest), guest-mode gating of online services, secure session storage,
  silent token refresh, and sign-out.
- `account-management`: Account creation and stewardship — email sign-up with
  OTP verification, *forgot password* via OTP reset, unique-handle onboarding and
  profile, and account deletion.

### Modified Capabilities
<!-- The backend `user-account` capability (from change `add-cymbra-id`) is not yet
     archived into openspec/specs/, so it is not listed as a delta here. The
     required backend evolution for unique handles is captured under Impact and in
     design.md / tasks.md as an explicit cross-tier dependency. -->

## Impact

- **Affected app code** (`apps/music/`):
  - `lib/main.dart`: `CymbraApp` becomes a `ConsumerWidget`; home is now driven by
    a session/auth provider (entry screen vs `LibraryScreen`).
  - New `lib/screens/auth/*` (entry, email sign-in/sign-up, OTP, forgot-password,
    handle onboarding), `lib/state/auth_*` (Freezed data + `@riverpod` notifier),
    `lib/services/auth_service.dart` + gRPC-backed implementation, `lib/services/
    token_store.dart`.
- **New dependencies**: `grpc`, `protobuf`, `protoc_plugin` (dev/codegen),
  `flutter_secure_storage`, `google_sign_in`, `sign_in_with_apple`. New build
  step: generate Dart stubs from the backend `.proto` files.
- **Backend dependency (cross-tier, this change)**: Cymbra ID's user module must
  gain (1) a `handle` field on `Account`, (2) a **UNIQUE** constraint on it, and
  (3) a `CheckHandleAvailability` RPC. Tracked in tasks; coordinate before the app
  flow can ship.
- **Platform config**: Google OAuth client IDs (iOS/Android/macOS) registered and
  set as the backend's `CYMBRA_GOOGLE_AUDIENCE`; Apple "Sign in with Apple"
  capability + `CYMBRA_APPLE_AUDIENCE`; iOS URL schemes / Android intent filters
  for the Google SDK.
- **Tests/coverage**: ≥80% maintained. The gRPC channel + native sign-in SDKs sit
  behind injectable seams (like `midi_service.dart`) so the new state/widgets are
  covered with fakes; the thin FFI/SDK adapters are coverage-excluded.
- **Non-goals**: identity linking (`LinkIdentity`/`UnlinkIdentity`), the `live`
  audience, and any concrete online feature — gating is scaffolded but no online
  service is built here.
