## Why

Google (and Apple) sign-in is **hidden on Windows/Linux desktop**: the app uses the
native `google_sign_in` / `sign_in_with_apple` plugins, which have **no Windows/Linux
implementation**. So desktop users there can only use email/guest — and the
account-linking flow (`add-account-identity-linking`) can't offer "Link Google" there
either. Yet desktop Google sign-in is entirely possible via the standard **browser
loopback OAuth flow** (RFC 8252) that many desktop apps use. Adding it unlocks
Google sign-in **and** linking on Windows/Linux without touching the backend.

## What Changes

- **Add a browser-based loopback OAuth (PKCE) flow** for Google on desktop
  (Windows/Linux): open the **system browser**, run a tiny local `127.0.0.1`
  HTTP server to catch the redirect, exchange the authorization code (PKCE/S256)
  for tokens, and extract the `id_token`.
- **Wire it behind the existing `OidcTokenSource` seam** as a desktop
  implementation, selected on Windows/Linux. macOS/iOS/Android keep their working
  native flow unchanged.
- **`googleAvailable` becomes true on Windows/Linux** when configured → the Google
  entry button **and** the "Link Google" action (account-linking) appear there.
- **Preserve Option A** (single backend audience = the web client): the loopback
  flow targets the **web client** so the `id_token` audience stays the web client —
  **no backend change** (pending the client-type spike, see design).
- **Apple on desktop is a non-goal** here (its web flow needs a Services ID + a
  hosted return URL — heavier; Google desktop first).

## Capabilities

### New Capabilities
- `desktop-oidc-loopback`: Browser-based OAuth (authorization code + PKCE, loopback
  redirect) to obtain a Google `id_token` on desktop platforms where the native SDK
  is unavailable, behind the injectable OIDC seam — enabling Google sign-in and
  linking on Windows/Linux.

### Modified Capabilities
<!-- account-access / account-linking are not yet archived into openspec/specs/, so
     there is no delta target. The platform-availability gating that this change
     flips on for desktop is specified in account-linking ("Link actions follow
     platform availability") and the sign-in entry buttons inherit the same gate. -->

## Impact

- **App** (`apps/music/`): a `DesktopOidcTokenSource` (or a desktop branch of the
  OIDC seam) implementing the loopback flow; platform selection so Windows/Linux use
  it; `googleAvailable` updated to be true on desktop when configured. Reuse the
  existing `signInOidc` path — only the *token acquisition* differs.
- **New dependencies**: `url_launcher` (open the system browser), `crypto` (PKCE
  S256), an HTTP client for the token exchange (`http`); a local `dart:io`
  `HttpServer` on an ephemeral `127.0.0.1` port. (`flutter_appauth` is not used —
  it lacks solid Windows/Linux support.)
- **Google Cloud**: register loopback redirect URIs (`http://127.0.0.1` /
  `http://localhost`) on the OAuth client used by the flow.
- **Backend**: none **if** the flow keeps `aud = web client` (Option A). If Google
  forces a separate "Desktop app" client (different `aud`), the backend must accept
  that audience too — captured as the key spike/decision in design.
- **Tests/coverage**: ≥80% — keep PKCE/state/redirect-parsing logic in pure,
  host-testable helpers; the browser/HttpServer glue sits behind the seam and is
  covered with a fake (no real browser).
- **Out of scope**: Apple on desktop (web flow); mobile changes; any change to the
  native macOS/iOS/Android flow.
