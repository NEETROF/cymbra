## Context

The auth module already issues a `TokenPair { access_token, refresh_token }` from
`SignInLocal`/`SignInOidc`/`Refresh` over gRPC, with sliding, rotated refresh tokens
and durable, revocable sessions (`backend-auth`). Native apps store these in platform
secure storage — fine. The **web** back office stores both in `localStorage`, exposing
the 30-day refresh token to any XSS. This change adds a browser-specific delivery of
the refresh token (an `HttpOnly` cookie) without touching token semantics.

## Goals / Non-Goals

**Goals:**
- Refresh token unreadable by JS (`HttpOnly` cookie); access token memory-only.
- Reuse the existing auth module (signing, rotation, revocation) unchanged.
- Keep the gRPC data plane (score/user services) untouched — still Bearer access token.
- CSRF-safe cookie endpoints.

**Non-Goals:**
- Changing token signing, TTLs, rotation, or session storage (`backend-auth`).
- Moving the whole gRPC API behind cookies (only the auth handshake uses cookies).
- Changing native (Flutter/desktop) auth.
- A full BFF/session-proxy for all API calls.

## Decisions

**1. A dedicated HTTP auth surface for the web, not gRPC-web `Set-Cookie`.**
Setting/reading cookies from a gRPC-web handler is awkward (framed binary body, trailer
semantics). Instead expose a few plain HTTP endpoints on the existing HTTP server
(`/web/auth/signin`, `/web/auth/refresh`, `/web/auth/logout`) that wrap the auth module
and speak JSON + `Set-Cookie`. The gRPC data plane is unchanged. Alternative
considered: emit `Set-Cookie` from the gRPC handlers via a tower layer — rejected as
fragile and coupling the data transport to session concerns.

**2. Refresh token in an `HttpOnly; Secure; SameSite=Strict` cookie, scoped by path.**
`Path=/web/auth` so it's only sent to the auth endpoints (not every request);
`Max-Age` = refresh TTL; `Domain` set per-env to the shared parent (e.g. `cymbra.app`).
The access token is returned in the JSON body (never a cookie) for in-memory use.

**3. Same-site topology is required.**
The API and back office must share a registrable domain (`api.cymbra.app` +
`bo.cymbra.app`) so the cookie is first-party and survives third-party-cookie blocking
(Safari ITP, Firefox, Chrome). Documented as a deployment constraint; dev uses
`localhost` (cookies shared across ports).

**4. Access token stays in memory (a store value), never persisted.**
On app load the SPA calls `/web/auth/refresh` (credentialed) to mint an access token
from the cookie; on a gRPC 401 it refreshes once and retries (the interceptor already
exists — it just calls the cookie endpoint instead of the JS refresh RPC). A reload
loses the access token but silently re-mints it from the cookie.

**5. CSRF defense-in-depth.**
`SameSite=Strict` blocks cross-site sending of the cookie. The endpoints require a
JSON content-type / custom header → forces a CORS preflight a simple `<form>` can't
satisfy. Optionally a double-submit CSRF token for the refresh/logout endpoints.
CORS: `Allow-Credentials: true` + exact origin from the existing allow-list (never `*`).

**6. Rotation reuse detection is inherited.**
Refresh rotation + reuse detection live in the auth module already; the cookie is just
a carrier. A stolen-then-rotated cookie triggers the same revocation path.

## Risks / Trade-offs

- **Cross-site cookie blocking** → mitigated by the same-site domain requirement
  (decision 3); a split-domain deploy would break it, so this is a hard constraint.
- **CSRF** → `SameSite=Strict` + preflighted JSON endpoints + optional double-submit;
  documented threat model.
- **XSS still allows in-page abuse** → `HttpOnly` stops *exfiltration/persistence*, not
  in-page requests during a live XSS; the CSP + `no-v-html` work (already shipped) plus
  the short access-token TTL bound the residual risk.
- **Two auth surfaces (gRPC for native, HTTP for web)** → small duplication, but both
  are thin wrappers over one auth module; kept intentionally minimal.
- **Multi-tab logout** → clearing the cookie invalidates refresh everywhere; open tabs
  keep their in-memory access token until it expires (bounded by the ~15-min TTL).
