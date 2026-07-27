## Why

The web back office keeps **both** the access token and the long-lived (30-day)
refresh token in `localStorage`, which is readable by any JavaScript on the page. A
single XSS therefore exfiltrates the refresh token → **persistent, offline-replayable
account takeover** of a moderator/admin. Moving the refresh token out of JS reach
(an `HttpOnly` cookie) removes that entire class of impact; the access token becomes
memory-only so nothing high-value survives a reload.

## What Changes

- Add a small **web-auth HTTP surface** (a thin gateway over the existing auth
  module) for browser clients: sign-in, refresh, and sign-out endpoints that
  **set/rotate/clear the refresh token in an `HttpOnly; Secure; SameSite=Strict`
  cookie** (scoped to the auth path) and return the **access token in the JSON body**.
  The refresh token never transits JavaScript.
- The **back office** stops persisting tokens: the access token is held **in memory
  only**, obtained/renewed via the credentialed refresh endpoint (on load and on a
  401). `localStorage` no longer holds any token.
- **CORS** for the web-auth surface: `Access-Control-Allow-Credentials: true` with an
  exact allowed-origin echo (reuse the existing back-office origins allow-list).
- **CSRF** protection on the cookie endpoints (`SameSite=Strict` + a
  non-simple/JSON content-type that forces a preflight; optional double-submit token).
- **Native clients unchanged**: the Flutter/desktop apps keep the gRPC `TokenPair`
  flow with tokens in platform secure storage (already out of a web JS context).

## Capabilities

### New Capabilities
- `web-auth-session`: cookie-based browser sessions for Cymbra web clients — the
  refresh token lives in an `HttpOnly` cookie managed server-side; the access token
  is delivered in the response body for in-memory use only; sign-in/refresh/sign-out
  and their CSRF protection.

### Modified Capabilities
<!-- None. The gRPC AuthService (SignInLocal/SignInOidc/Refresh/Logout → TokenPair)
     stays as-is for native clients; this change ADDS a parallel web surface over the
     same auth module. `backend-auth` session/rotation requirements are unchanged. -->

## Impact

- **Backend**: a new HTTP route group (on the existing HTTP server) that wraps the
  auth module for local + OIDC sign-in, refresh (reads the cookie), and logout
  (revokes the session + clears the cookie); cookie construction/attributes; CORS
  credentials; CSRF check. No change to token signing, rotation, or session storage
  (`backend-auth`).
- **Back office**: `stores/auth.ts` (in-memory access token, no `localStorage`),
  `lib/transport.ts` (credentialed transport; refresh via the cookie endpoint instead
  of the JS refresh RPC), `main.ts` wiring, sign-in/sign-out views.
- **Config/infra**: cookie `Domain`/`Secure` per environment; the API and back office
  must be **same-site** (e.g. `api.cymbra.app` + `bo.cymbra.app` under `cymbra.app`)
  so the cookie is first-party (not blocked by third-party-cookie policies).
- **Docs**: the reverse proxy should also send the header-form CSP (`frame-ancestors`)
  and HSTS (complements the build-time CSP meta already added).
