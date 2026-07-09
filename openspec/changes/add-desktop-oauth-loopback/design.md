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

## D3 resolution (spike outcome — tasks 1.1/1.2)

**Decision: keep the app decision-agnostic; confirm the client type before release.**

Findings:
- **Web client**: Google allows `http://localhost`/`http://127.0.0.1` loopback
  redirect URIs on a *Web application* client, and it can run authorization-code +
  PKCE. But a Web client is **confidential** — Google's token endpoint requires its
  `client_secret` at the code→token exchange. Shipping that secret in a distributed
  desktop binary makes it non-confidential (D3/Risks). If used, `aud` stays the web
  client ⇒ **no backend change** (Option A preserved).
- **Desktop app client**: a **public** client — loopback + PKCE with **no secret**
  (RFC 8252). This is the standard, secret-free desktop pattern. Its `aud` is the
  desktop client id, so the **backend must accept a second Google audience**
  (config + verifier accepted-audience list) — this reopens the multi-audience
  question (D3 option a).

**What we build now:** the loopback flow reads a desktop `client_id` and an
**optional** `client_secret` from `--dart-define`. When a secret is present it is
sent on the token exchange (web-client path, Option A); when absent the exchange is
pure PKCE (desktop-client path). This makes the app work under either choice without
another code change.

**Update (desktop-client path taken):** a Google **Desktop app** client was created
(`cymbra desktop`). Two consequences handled in code:
- Google requires the desktop client's `client_secret` at the token exchange even
  with PKCE — supplied via `DESKTOP_GOOGLE_CLIENT_SECRET`; the exchanger sends it
  when non-empty.
- The desktop token's `aud` = desktop client id, so the **backend now accepts a
  comma-separated audience set** (`CYMBRA_GOOGLE_AUDIENCE=<web>,<desktop>`):
  `OidcProvider`/`OidcProviderCfg` carry `audiences: Vec<String>` and the verifier
  calls `set_audience(&audiences)` (a token matching any one is trusted).

A failed exchange / bad `state` / unopened browser now **throw** `DesktopOauthException`
(surfaced by the UI) instead of returning null — only a real user cancel (timeout or
`error=access_denied`) stays a silent no-op.

**Still required before release (external — cannot be done in code):**
- Task 1.3: register the loopback redirect URIs on the Google Cloud desktop client.
- Set the backend's `CYMBRA_GOOGLE_AUDIENCE` to include the desktop client id.
- Task 5.3: manual end-to-end verification of sign-in + "Link Google" on Windows/Linux.

## macOS resolution (open question — keep native)

**Decision: macOS keeps the native `google_sign_in` flow; do NOT unify it onto the
loopback flow.** The loopback source stays gated to Windows/Linux.

Rationale:
- **Preserves Option A on macOS.** Native macOS uses the iOS client +
  `serverClientId` (web client), so its `id_token` `aud` is the web client — the
  single backend audience. Moving macOS to loopback would switch its `aud` to the
  desktop client and force a backend audience addition on a platform that has no
  problem today.
- **Avoids a macOS App Sandbox entitlement.** The loopback flow binds a *listening*
  socket on `127.0.0.1`, which a sandboxed macOS app needs
  `com.apple.security.network.server` for (extra entitlement, more review surface).
  The native SDK needs no listening socket.
- **Little upside.** `google_sign_in` stays a dependency for iOS/Android regardless,
  so unifying removes only a code path on a working platform — not the dependency.

Revisit only if maintaining the native macOS path becomes costly. The loopback flow
targets the platforms that are actually broken (Windows/Linux).

## Open Questions

- D3 client type: can the **web** client do the loopback flow (keeping Option A), or
  is a **desktop** client required (and thus a backend audience addition)?
- ~~macOS: keep the native flow, or unify macOS onto the loopback flow too?~~
  **Resolved: keep native** (see "macOS resolution" above).
- Do we want a branded local success page, or the minimal default?
