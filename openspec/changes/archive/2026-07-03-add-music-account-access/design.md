## Context

Cymbra Music (`apps/music/`) is a Flutter app that today has **no networking and
no user concept** — `LibraryScreen` is the unconditional `home`, state is Riverpod
2 + Freezed (codegen), and native capabilities (MIDI/audio) live behind injectable
service seams (e.g. `lib/services/midi_service.dart`).

The **Cymbra ID** backend (`backend/`, from change `add-cymbra-id`) is a
**gRPC-only** modular monolith. Relevant facts that shape this design:

- Two public services: `cymbra.auth.v1.AuthService` (sign-up/verify/sign-in/
  refresh/logout/reset) and `cymbra.user.v1.UserService` (`GetAccount`,
  `UpdateAccount`, `ListIdentities`, `DeleteAccount`).
- Sessions are **audience-scoped**; the app always uses audience `music`. Access
  token = short-lived EdDSA JWT (~15m); refresh token = opaque, rotated (~30d).
- OIDC is **token-in, not redirect**: the client obtains a Google/Apple `id_token`
  and posts it to `SignInOidc`; the backend verifies it against the provider JWKS.
  New accounts are auto-provisioned on first `SignInOidc` (no "is new" flag).
- `SignUpLocal` takes only email+password (no display name). The profile field
  today is `display_name` (optional, **non-unique**).
- Email verification and password reset are **token-by-email**; we elected to have
  the user type the code (OTP), not deep-link.

This is a cross-tier change: the unique-handle requirement forces a small backend
evolution because `display_name` is non-unique today.

## Goals / Non-Goals

**Goals:**
- Make account entry the launch experience with four options (Google, Apple,
  email, guest), on theme, with a returning user skipping straight in.
- Integrate the three online methods against Cymbra ID over native gRPC.
- Uniform post-auth handle onboarding (unique handle) for new users across all
  three methods.
- Full local lifecycle: sign-up + OTP verify, forgot-password OTP reset, sign-out,
  account deletion.
- Keep all new state/widgets testable to ≥80% by hiding gRPC + native SDKs behind
  injectable Riverpod seams.

**Non-Goals:**
- Identity linking (`LinkIdentity`/`UnlinkIdentity`) and multi-identity UI.
- The `live` audience and any cross-app session sharing.
- Building any actual online feature; guest gating is scaffolded, not exercised.
- Magic-link / deep-link return paths (explicitly deferred in favor of OTP).
- Offline queueing of auth actions beyond silent refresh on reconnect.

## Decisions

### D1. Native gRPC client behind an injectable `AuthService` seam
Generate Dart stubs from `backend/auth-port/proto/auth.proto` and
`backend/user-port/proto/user.proto` with `protoc` + `protoc_plugin`; wrap them in
an abstract `AuthService`/`AccountService` exposed via `@riverpod`, mirroring
`midi_service.dart`. Production impl owns the `ClientChannel`; tests override the
provider with an in-memory fake.
- *Why:* matches the mandated DI pattern and keeps the coverage gate satisfiable
  without a live backend or generated code in the unit suite.
- *Alternatives:* a REST/grpc-web gateway (rejected — adds a backend surface out of
  scope); calling generated stubs directly from notifiers (rejected — untestable,
  violates the seam convention).

### D2. Session-driven home via an auth/session notifier
`CymbraApp` becomes a `ConsumerWidget`; `home` switches on a `SessionState`
(`@freezed`: `unknown | guest | authenticated | unauthenticated`) produced by an
`@riverpod` `SessionNotifier`. At startup the notifier hydrates from secure storage
(stored guest choice or token pair → attempt silent `Refresh`) and resolves to one
of the states; the entry screen renders only for `unauthenticated`.
- *Why:* the spec requires returning users (and guests) to skip the entry screen;
  a single source-of-truth notifier keeps routing declarative.
- *Alternatives:* `go_router` redirects (heavier than needed; app uses plain
  `Navigator`); a `FutureBuilder` in `main` (loses Riverpod testability).

### D3. Token storage + auto-refresh in a gRPC interceptor
Tokens live in `flutter_secure_storage` behind a `TokenStore` seam. A gRPC client
interceptor injects `authorization: Bearer <access>` and, on `UNAUTHENTICATED`,
performs a single `Refresh` + retry; a failed refresh clears the store and flips
`SessionState` to `unauthenticated`.
- *Why:* centralizes the 15-minute access-token churn so feature code never sees
  it; one retry avoids loops.
- *Alternatives:* per-call refresh logic (duplicated, error-prone); proactive
  timer-based refresh (more code, clock-skew sensitive) — can be added later.

### D4. Uniform post-auth handle onboarding gate
Handle collection is **always** after authentication (because `SignUpLocal`/
`SignInOidc` don't accept it and there's no "is new" flag): on every successful
sign-in the `SessionNotifier` calls `GetAccount`; a null handle routes to a blocking
handle screen that debounces a `CheckHandleAvailability` call and commits with
`UpdateAccount`, treating a write-time uniqueness conflict as "pick another".
- *Why:* one code path for all four entry methods; relies on server truth, not a
  client "new user" guess.
- *Alternatives:* collecting pseudo on the email sign-up form (impossible — backend
  rejects the field; and wouldn't cover OIDC).

### D5. Backend evolution for unique handles (cross-tier, this change)
Add a `handle` to the user-account model with a DB **UNIQUE** constraint, expose
`CheckHandleAvailability(handle)`, and accept/validate `handle` on `UpdateAccount`
(map the unique-violation to gRPC `ALREADY_EXISTS`/`ABORTED`). Uniqueness is
guaranteed by the constraint at write time; the check RPC is advisory only.

**Handle policy (decided):** 1–15 characters, UTF-8 **letters and numbers only**
(Unicode letters/digits, no spaces/punctuation/symbols). Uniqueness is
**case-insensitive**: store the user-entered handle for display, but enforce the
UNIQUE constraint and the availability check on a **normalized** form — Unicode
**NFC** + case-fold — via a generated/normalized column or a functional unique
index, so handles that differ only by case (or by NFC-equivalent code points)
collide. NFC is essentially free and removes the most common accidental
duplicates; a full confusables/homoglyph guard (e.g. `Alıce` vs `Alice`) is **not**
built here — see Risks. Validation runs identically client- and server-side.
- *Why:* the user chose a unique handle; `display_name` cannot provide it. Case-
  insensitive comparison prevents look-alike impersonation (`Alice` vs `alice`).
- *Alternatives:* client-only "uniqueness" (rejected — racy, unenforceable);
  reusing `display_name` as-is (rejected — non-unique).

### D8. Re-authentication before account deletion
`DeleteAccount` SHALL be gated behind a fresh re-authentication step: an email
user re-enters their password (verified via `SignInLocal`), and a Google/Apple
user re-runs the native sign-in to produce a fresh `id_token`. Only after the
re-auth succeeds is the irreversible confirmation enabled and `DeleteAccount`
called.
- *Why:* deletion is destructive and irreversible; a short-lived access token (or
  an unlocked, unattended device) should not be enough to wipe an account.
- *Trade-off:* one extra step; acceptable for a rare, destructive action. The
  backend doesn't mandate it, so this is enforced entirely client-side.

### D6. OTP (typed code) for verification and reset
The verification and reset screens take the emailed code as user input and call
`VerifyEmail`/`ResetPassword`. No universal-links/App-Links setup.
- *Why:* zero deep-link infra, works on every platform, fastest correct path.
- *Trade-off:* slightly heavier UX than a tappable link; revisit later behind D2's
  routing if desired.

### D7. Guest mode as a first-class session state with a gating helper
Guest is a persisted `SessionState.guest`; online-bound features check a
`requiresAccount` guard (provider) that, for guests, surfaces a sign-in prompt
instead of calling the backend. No Cymbra ID channel is opened in guest mode.
- *Why:* the spec forbids any backend access for guests; a single guard centralizes
  the rule for the (future) online features.

## Risks / Trade-offs

- **Cross-tier coupling (handle uniqueness)** → The app flow can't fully ship until
  the backend gains the handle field + `CheckHandleAvailability`. Mitigation:
  sequence backend tasks first; the `AccountService` seam lets the app compile and
  be tested against a fake before the RPC exists.
- **OAuth audience misconfig** → Google/Apple `id_token`s carry per-platform client
  IDs; if the backend `CYMBRA_GOOGLE_AUDIENCE`/`CYMBRA_APPLE_AUDIENCE` don't include
  them, `SignInOidc` fails with `UNAUTHENTICATED`. Mitigation: document required
  client IDs per platform; verify in an integration smoke test against mock-oidc.
- **gRPC on all target platforms** → grpc-dart needs HTTP/2; fine on iOS/Android/
  macOS/desktop (the app's targets), not web. Mitigation: app has no web target;
  if web is added, introduce grpc-web then.
- **Auth wall hurting an offline app** → forcing a login screen first can deter
  users. Mitigation: prominent guest option + persisted choice so it's shown once.
- **Secure storage availability** → Keychain/Keystore can fail (e.g. no device
  lock). Mitigation: treat storage failure as "no session" and fall back to the
  entry screen rather than crashing.
- **Coverage gate vs generated stubs** → generated gRPC Dart must be excluded from
  coverage like `lib/src/rust/**`. Mitigation: add the generated path to the
  `very_good_coverage` excludes and keep logic out of adapters.
- **Handle homoglyph impersonation** (accepted, low priority) → NFC + case-fold does
  not stop visually-confusable handles across scripts (`Alıce`/`Alice`, Cyrillic
  `а`). Accepted for now since handles are display labels, not security principals
  (auth is by identity/token, not by handle). Mitigation hook: the normalization
  lives in one place (the generated/normalized column), so a confusables-skeleton
  pass (e.g. Unicode UTS-39) can be layered there later without touching the app.

## Migration Plan

1. **Backend first**: add `handle` column + UNIQUE constraint + migration,
   `CheckHandleAvailability` RPC, `handle` on `UpdateAccount`; keep `display_name`.
2. Add app dependencies + proto codegen step (Melos script + CI).
3. Land the `AuthService`/`AccountService`/`TokenStore` seams + `SessionNotifier`
   with fakes and tests (no UI yet).
4. Build the entry screen and gate `main.dart` on `SessionState` (guest path works
   end-to-end first, since it needs no backend).
5. Wire email, then Google, then Apple flows; add OTP verify/reset, handle
   onboarding, sign-out, deletion.
6. Platform config (Google client IDs, Apple capability, iOS/Android SDK wiring);
   integration smoke test against the `mock-oidc` compose profile.

**Rollback:** the change is additive behind the new launch gate; reverting to
`home: LibraryScreen()` restores prior behavior. The backend migration is additive
(nullable `handle` + constraint) and safe to leave in place.

## Open Questions

- Production gRPC endpoint + TLS termination details (dev uses plaintext localhost).
