## Why

A Cymbra account is keyed by `(provider, subject)`, and the app never merges by
email — so a user who signed up with email and later "Continue with Google" ends
up with **two separate accounts** (separate handles, separate future data). The
backend already exposes the primitives to fix this (`LinkIdentity`,
`UnlinkIdentity`, `ListIdentities`), but the app exposes none of them, and
linking was an explicit Non-Goal of `add-music-account-access`. This change makes
it a goal: let a signed-in user attach additional sign-in methods to their single
account and see what's connected.

## What Changes

- **New "Connected accounts" screen** (reached from account settings): lists the
  identities linked to the current account (provider + when linked, via
  `ListIdentities`), and offers per-provider actions.
- **Link an identity**: *Link Google* / *Link Apple* mint a fresh `id_token` via
  the existing `OidcTokenSource` and call `LinkIdentity`; *Set a password* links a
  local (email+password) credential where one isn't present.
- **Unlink an identity**, with the backend's anti-lockout guard surfaced in the
  UI: the **last remaining** identity cannot be unlinked (action disabled +
  explained), matching the server's `FAILED_PRECONDITION`.
- **Collision behavior (the key decision)**: when the chosen social identity
  already owns a *different* account, `LinkIdentity` returns `ALREADY_EXISTS`. For
  v1 we **surface a clear, dedicated error and do not merge** ("This Google
  account is already linked to another Cymbra account."). Cross-account *merge* is
  called out as a future extension, explicitly out of scope here.
- **Error-message fix**: OIDC/link failures currently fall through to the email
  sign-in copy "Incorrect email or password." (`auth_messages.dart` maps all
  `UNAUTHENTICATED` to it). Linking/sign-in surface **provider-appropriate**
  messages (link failed / already linked elsewhere / cannot unlink last identity).
- **User-driven "link at sign-in" from the collision point**: after a Google/Apple
  sign-in that lands on handle onboarding, offer "Already have an account? Sign in to
  link." The user picks *their* existing method and re-authenticates; the app then
  deletes the just-created orphan social account, signs in to the existing account, and
  calls `LinkIdentity` to attach the social identity. This is **user-driven** — the app
  never reveals that an account exists or which method it uses (no enumeration), and it
  uses only existing RPCs.
- **Service seam wiring**: add `linkIdentity` / `unlinkIdentity` / `listIdentities`
  to the injectable `AuthService` / `AccountService` seams and their gRPC adapters,
  with fakes for tests. No backend changes — the RPCs already exist.

## Capabilities

### New Capabilities
- `account-linking`: Managing the set of sign-in identities bound to a single
  account — listing connected identities, linking Google/Apple/email-password,
  unlinking with a last-identity lockout guard, the `ALREADY_EXISTS` collision
  behavior, optional recent-auth gating, and provider-appropriate error messaging.

### Modified Capabilities
<!-- account-access / account-management (from add-music-account-access) are not
     yet archived into openspec/specs/, so there is no delta target here. The
     UNAUTHENTICATED error-message fix that lives in those flows is captured as a
     requirement of account-linking and in design.md/tasks.md. -->

## Impact

- **Affected app code** (`apps/music/`):
  - New `lib/screens/account/connected_accounts_screen.dart` (+ an entry point in
    account settings) and a `@riverpod` notifier + Freezed state for the linked
    identities list and link/unlink actions.
  - `lib/services/auth_service.dart` + `lib/services/account_service.dart` seams
    gain `linkIdentity`/`unlinkIdentity`/`listIdentities`; `grpc_client.dart`
    adapters call the existing generated stubs (`linkIdentity`, `unlinkIdentity`,
    `listIdentities`).
  - `lib/screens/auth/auth_messages.dart`: stop using the email-credential copy
    for non-local `UNAUTHENTICATED`; add link/unlink-specific messages.
  - Reuse `OidcTokenSource` for Google/Apple `id_token`s; native SDKs stay behind
    the injectable seam.
- **No new dependencies. No backend changes** — `LinkIdentity`, `UnlinkIdentity`,
  `ListIdentities` already ship in Cymbra ID.
- **Re-auth consideration**: decide in design.md whether link/unlink needs a
  recent-auth gate (mirroring the delete-account decision D8).
- **Tests/coverage**: ≥80% (Flutter) maintained; new state/widgets covered with
  fakes, the thin gRPC/SDK adapters coverage-excluded as today.
- **Depends on** `fix-handle-onboarding-escape` for the handle-screen escape action and
  orphan-account deletion; this change adds the "sign in to link" option alongside that
  escape and reuses its abandon/delete-orphan path before calling `LinkIdentity`.
- **Out of scope**: cross-account *merge*, auto-linking by email, revealing an existing
  account's sign-in method, the `live` audience, any server-side change.
