## Context

`add-music-account-access` shipped sign-in (Google/Apple/email) and made linking an
explicit Non-Goal. Accounts are keyed by `(provider, subject)` and the app never
merges by email, so a user who has an email account and later signs in with Google
silently gets a **second** account. The backend already exposes the fix —
`AuthService.LinkIdentity(id_token)`, `AuthService.UnlinkIdentity(provider,
subject)`, `UserService.ListIdentities()` — but the app wires none of it. This
change adds the app surface only; **no server work**.

Existing seams to build on: `AuthService` / `AccountService` (injectable, gRPC-backed
in `grpc_client.dart`), `OidcTokenSource` (native Google/Apple `id_token`s behind a
fake-able seam), `SessionNotifier` (current session/bearer), and the
delete-account screen as a precedent for sensitive account actions.

## Goals / Non-Goals

**Goals:**
- Let a signed-in user see connected identities and link/unlink Google, Apple, and
  email/password against the existing RPCs.
- Decide and specify the `ALREADY_EXISTS` collision behavior.
- Fix the misleading "Incorrect email or password." message on non-local
  `UNAUTHENTICATED` failures.
- Keep native SDK/gRPC behind injectable seams; ≥80% Flutter coverage with fakes.

**Non-Goals:**
- Cross-account **merge** (combining two existing accounts' data).
- Auto-linking by email at sign-in time.
- Any backend/proto change; the `live` audience.

## Decisions

### D1 — New `account-linking` surface reusing existing seams
A `ConnectedAccountsScreen` reached from account settings, backed by a `@riverpod`
notifier + Freezed state holding the `ListIdentities` result and per-action status.
Add `listIdentities`/`linkIdentity`/`unlinkIdentity` to the `AccountService`/`AuthService`
seams and their `Grpc*` adapters (calling the already-generated stubs). *Alternative:*
call gRPC stubs directly from the widget — rejected; breaks the testable-seam
convention and the coverage exclusion for FFI/gRPC.

### D2 — Reuse `OidcTokenSource` to mint link tokens
Linking Google/Apple reuses the same `googleIdToken()`/`appleIdToken()` the sign-in
flow uses; `LinkIdentity` is called with the current session bearer (so the identity
attaches to the logged-in user, not a new one). No new SDK surface. Fakes already
exist for the source.

### D3 — Collision (`ALREADY_EXISTS`) → clear error, no merge (v1)
When the social identity already owns another account, show a dedicated message
("This Google account is already linked to another Cymbra account.") and change
nothing. *Alternative considered:* an account-merge flow — rejected for v1: it needs
new server support, a data-ownership/transfer model, and re-auth on both accounts;
it is a separate, larger change. Documented as the intended future extension.

### D4 — Last-identity guard enforced on both ends
The client disables the unlink action when only one identity remains and explains
why; it also maps a server `FAILED_PRECONDITION` to the same message in case state
is stale. Unlink needs `(provider, subject)`, both taken from `ListIdentities`
(subjects are opaque and never displayed — provider + linked-at only).

### D5 — Provider-appropriate error messages
`auth_messages.dart` stops mapping *all* `UNAUTHENTICATED` to the email-credential
copy. The local sign-in screen keeps "Incorrect email or password." by passing it as
an explicit fallback; OIDC sign-in and link/unlink pass their own fallbacks
(link failed / already linked elsewhere / can't remove only sign-in method). This
also repairs the existing Google sign-in failure UX.

### D6 — No re-auth gate for v1 link/unlink
Unlike account deletion (D8 in the prior change), link/unlink does not require a
recent-auth step in v1: linking is additive, and the last-identity guard prevents
lockout from unlinking. *Alternative:* gate unlink behind re-auth (a credential
removal is sensitive) — deferred; revisit if abuse on an unlocked device is a
concern. Captured as an Open Question.

### D7 — User-driven "sign in to link" at the sign-in collision point
After a social sign-in that lands on handle onboarding, offer "Already have an account?
Sign in to link." The flow is **user-driven**: the user chooses their existing method
and re-authenticates, proving ownership of the existing account. The app then (1)
deletes the just-created orphan social account (reusing the abandon/delete path from
`fix-handle-onboarding-escape`, freeing `(provider, subject)`), (2) signs in to the
existing account, and (3) calls `LinkIdentity` with the still-valid social `id_token`
to attach the identity. Net result: one account with both identities.

Ordering matters — the orphan must be deleted before `LinkIdentity`, otherwise the
social identity is still owned by the orphan and the link returns `ALREADY_EXISTS`.

*Alternatives considered:*
- **Reveal the existing account's method** ("you already have a password account") —
  rejected for v1: requires persisting the OIDC email and a new backend lookup, and is
  an account-enumeration surface (only mitigated by the token proving email control). A
  user-driven flow needs none of that — the user supplies the method by choosing it.
- **Auto-link by email at sign-in** — rejected: classic pre-hijacking risk; owning the
  email is not the same as controlling the existing account. Re-auth into the existing
  account is required.

## Risks / Trade-offs

- **Two-account dead-end** → A user who already created a standalone Google account
  can't link it and gets only an error. Mitigation: clear messaging now; merge as a
  named future change. Surfaced directly by the Option-1 grpcurl test.
- **No re-auth on unlink** → A walk-up attacker on an unlocked, signed-in device
  could remove a credential. Mitigation: last-identity guard limits blast radius;
  re-auth gating is an easy follow-up (D6 Open Question).
- **Apple relay/email + opaque subjects** → Don't display subjects; rely on provider
  labels. Unlink uses the `(provider, subject)` from `ListIdentities`, not derived
  values.
- **Stale list after link/unlink** → Always re-fetch `ListIdentities` after a
  mutating action rather than mutating local state optimistically.

## Migration Plan

Purely additive: a new screen + seam methods + message changes. No schema, proto, or
startup-contract change; nothing to roll back beyond reverting the app code. Ships
independently of any online feature. Requires the backend's `CYMBRA_GOOGLE_AUDIENCE`
(and `CYMBRA_APPLE_AUDIENCE` for Apple) configured — same prerequisite as sign-in.

## Open Questions

- Should unlink (or all link/unlink) require a recent-auth gate before v1 ships? (D6)
- ~~Discoverability hook at the sign-in collision point?~~ **Resolved (D7):** yes, a
  user-driven "Sign in to link" option, with no method disclosure and no email lookup.
- The social `id_token` must still be valid when `LinkIdentity` runs (after re-auth into
  the existing account). If re-auth is slow and the token expires, re-mint it via the
  OIDC source before linking — confirm UX for that edge.
- Timing/scope of the future cross-account **merge** change.
