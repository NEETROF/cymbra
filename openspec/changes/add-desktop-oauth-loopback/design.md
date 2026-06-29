## Context

The app obtains Google/Apple `id_token`s via native plugins behind the
`OidcTokenSource` seam (`googleIdToken`/`appleIdToken`, gated by
`googleAvailable`/`appleAvailable`). Those plugins have no Windows/Linux support, so
desktop gating hides Google/Apple there. The backend trusts a **single Google
audience = the web client** (Option A, from `add-music-account-access`). The standard
way to do Google sign-in on desktop is the **OAuth 2.0 authorization code + PKCE
loopback** flow (RFC 8252): system browser + a `127.0.0.1` redirect the app listens
on. Only the *token acquisition* changes; the downstream `SignInOidc` /
`LinkIdentity` paths are unchanged.

## Goals / Non-Goals

**Goals:**
- Google sign-in **and** linking on Windows/Linux via the browser loopback flow.
- Keep the backend's single audience (Option A) — ideally no backend change.
- Keep the native flow on macOS/iOS/Android untouched.

**Non-Goals:**
- Apple on desktop (its web flow needs a Services ID + a hosted return URL).
- Refresh-token management for Google — we need the `id_token` once; the backend
  issues and refreshes its own session.
- Replacing the native flow on platforms where it already works.

## Decisions

### D1 — Authorization code + PKCE, loopback redirect (RFC 8252)
On desktop: generate a PKCE verifier/challenge (S256) + a `state`; start a `dart:io`
`HttpServer` on an ephemeral `127.0.0.1` port; open Google's authorization URL in the
**system browser** (`url_launcher`); the browser redirects to
`http://127.0.0.1:<port>/...` which the server captures; validate `state`; exchange
`code` + verifier at Google's token endpoint; read the `id_token`. Show a small
"you can close this tab" page; handle timeout/cancel.

### D2 — Behind the seam: `DesktopOidcTokenSource`, selected on Windows/Linux
Add a desktop implementation of `OidcTokenSource` (or a desktop branch). Platform
selection: Windows/Linux → loopback; macOS/iOS/Android → existing native source.
`googleAvailable` becomes true on Windows/Linux when configured. The loopback source
implements `googleIdToken({forceChooser})` (browser `prompt=select_account` for
`forceChooser`) and `signOut()` (clear any cached state); Apple stays unavailable on
desktop.

### D3 — Audience: keep `aud = web client` (Option A) — the key spike
The `id_token` audience equals the OAuth **client_id used in the auth request**. To
preserve the single backend audience, the loopback flow should use the **web client**
(register `http://127.0.0.1`/`localhost` redirect URIs on it). *Open risk:* Google may
require a **"Desktop app"** client for loopback, whose `aud` differs from the web
client. If so, either (a) the backend accepts a second Google audience (small config +
verifier change — note this reopens the multi-audience question), or (b) accept a
desktop-specific audience. **Spike this first** (D-spike) before building the flow.

### D4 — Pure core, fake-able glue
Keep PKCE (verifier/challenge/S256), the authorization-URL builder, `state`
generation/validation, and redirect-query parsing in **pure, host-testable**
functions. The browser launch + `HttpServer` + HTTP token exchange sit behind a thin
adapter, covered by a fake in tests (no real browser/network).

### D5 — Security
PKCE S256 (no client secret needed for the public/native flow); `state` for CSRF;
loopback only on `127.0.0.1` (not `0.0.0.0`); short timeout; single-use server shut
down after the redirect; never log the code/verifier/token.

## Risks / Trade-offs

- **Client-type vs audience (D3)** → spike before implementing; worst case a small
  backend multi-audience change.
- **Confidential web-client secret on desktop** → if a web client requires a secret,
  shipping it in a desktop app is non-confidential; prefer a public/desktop client
  with PKCE, which loops back to the audience question (D3).
- **Browser/redirect UX** → user may close the browser; handle cancel/timeout
  cleanly and free the port.
- **Port/firewall** → ephemeral loopback port on `127.0.0.1`; no inbound LAN, so no
  firewall prompt expected.

## Migration Plan

Additive and desktop-only: new desktop OIDC source + deps + Google Cloud redirect
URIs. macOS/iOS/Android unchanged. Ships independently; flipping `googleAvailable` on
desktop is what surfaces the button/link action. Rollback = revert the desktop source
(Google hides again on Windows/Linux).

## Open Questions

- D3 client type: can the **web** client do the loopback flow (keeping Option A), or
  is a **desktop** client required (and thus a backend audience addition)?
- macOS: keep the native flow, or unify macOS onto the loopback flow too?
- Do we want a branded local success page, or the minimal default?
