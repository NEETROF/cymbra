# Cymbra moderation back office (`bo.cymbra.app`)

A client-rendered Vue 3 + Vite SPA where moderators/admins review catalog scores
and accept/reject them, and admins grant/revoke roles. It talks to the backend over
**gRPC-web** (tonic-web) using generated Connect clients — no REST. Part of the
`add-moderation-back-office` change (frontend slice).

## Develop

```bash
cd apps/back-office
yarn install
yarn gen        # generate TS gRPC stubs from the backend protos (needs protoc)
cp .env.example .env   # set VITE_GRPC_WEB_URL + VITE_WEB_AUTH_URL to your backend
yarn dev
```

The backend must run with `CYMBRA_BACK_OFFICE_ORIGINS` including this app's origin
(e.g. `http://localhost:5173`) so CORS allows both the gRPC-web calls and the
credentialed web-auth cookie calls, and a moderator/admin account must exist (see
`backend/scripts/seed_admin.sh`). For plain-HTTP localhost set
`CYMBRA_WEB_AUTH_COOKIE_SECURE=false` (see [Cookie sessions](#cookie-sessions-no-token-in-js-storage)).

## Scripts

- `yarn gen` — regenerate `src/gen/*` from `backend/*/proto/*.proto` (gitignored).
- `yarn test` — Vitest component/store tests (its own gate, outside the Flutter/Rust CI).
- `yarn build` — type-check + production build.

## Debugging gRPC-web calls

gRPC-web always returns **HTTP 200** — the real status is the `grpc-status` trailer
(`0` = OK, else an error; e.g. `16` = unauthenticated), so Chrome's Network tab
hides failures. Two dev-only aids (wired in `src/lib/transport.ts`, `import.meta.env.DEV`):

- **Console**: every failed call logs one line — `gRPC <Method> → <Code> (<n>): <message>`.
- **gRPC-Web Developer Tools** Chrome extension
  ([SafetyCulture](https://github.com/SafetyCulture/grpc-web-devtools)): install it to
  get a DevTools panel decoding each call's request/response/status. The interceptor
  that feeds it (`src/lib/grpc-devtools.ts`) posts the events it listens for; no
  extension → harmless no-op.

User-facing errors never show raw codes: `humanError` (`src/lib/errors.ts`) maps
`ConnectError` codes to short messages and logs the real cause.

## Architecture

- **Components never call the API.** Only Pinia stores (`src/stores/`) import
  `api()`; views/components depend on stores. The gRPC-web clients live behind
  `src/lib/api.ts` (injectable — tests swap fakes via `setClientsForTest`).
- **Async state is a discriminated union, matched with `ts-pattern`.** Every remote
  resource is one `Async<T>` value (`idle | loading | success | error`, see
  `src/lib/async.ts`) — impossible states (loading + error at once) can't be
  represented. Views fold it into a template view-model with
  `match(...).exhaustive()`, so forgetting a state is a compile error. Errors land
  in the union (`{ status: "error" }`), not exceptions.

## Auth & access

Sign-in targets the `music` audience so `music`-scoped roles (`moderator`/`admin`)
ride in the access token. The router gates the console: unauthenticated → sign-in;
signed-in non-moderator → access-denied; `/roles` requires `admin`. This is UX only —
**every RPC is independently role-guarded server-side**, so the gate can't be bypassed.

### Cookie sessions (no token in JS storage)

The refresh token is **never** exposed to page JavaScript, and the access token is
**memory-only** (nothing in `localStorage`/`sessionStorage`/non-`HttpOnly` cookie), so
an XSS can't exfiltrate a replayable session (change: `add-web-auth-cookies`).

- Sign-in / refresh / sign-out go over the **web-auth HTTP surface** (`VITE_WEB_AUTH_URL`,
  the backend's HTTP port), not gRPC. The server sets/rotates/clears the refresh token
  in an `HttpOnly; Secure; SameSite=Strict; Path=/web/auth` cookie and returns the access
  token in the JSON body. See `src/lib/web-auth.ts` (injectable seam, like `lib/api.ts`).
- On load the SPA calls `/web/auth/refresh` (credentialed) to **re-mint** an access token
  from the cookie — a reload stays signed in with nothing persisted. On a gRPC
  `UNAUTHENTICATED` the transport refreshes once via the cookie and retries; only if that
  refresh fails does it sign out.
- **CSRF**: the cookie is `SameSite=Strict` and every web-auth call sends a custom
  `X-Cymbra-Web` header (and JSON content-type) that forces a CORS preflight a cross-site
  `<form>` can't satisfy; CORS echoes the exact origin with credentials (never `*`).

### Deployment constraint — same-site + reverse proxy

The API and this SPA **must share a registrable domain** (e.g. `api.cymbra.app` +
`bo.cymbra.app` under `cymbra.app`, with `CYMBRA_WEB_AUTH_COOKIE_DOMAIN=cymbra.app`) so
the refresh cookie is first-party and survives third-party-cookie blocking (Safari ITP,
Firefox, Chrome). A split-domain deploy breaks the cookie — this is a hard constraint.
Dev uses `localhost` (cookies are shared across ports; set
`CYMBRA_WEB_AUTH_COOKIE_SECURE=false` for plain-HTTP localhost).

The host fronting the SPA should also send, as response **headers** (complementing
the build-time CSP `<meta>`) — on Cloudflare Pages these ship via
[`public/_headers`](public/_headers):

- `Content-Security-Policy: frame-ancestors 'none'` (header form — clickjacking defence
  the meta tag can't express); the `default-src`/`connect-src` policy stays in the
  build-time `<meta>` (it is env-aware — `connect-src` is pinned to `VITE_GRPC_WEB_URL`).
- `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload` (HSTS), so the
  `Secure` cookie is never attempted over plain HTTP.

### Session management & revocation

The **Sessions** view (`/sessions`) lists this account's active sessions and lets the
user revoke one device or **sign out everywhere**; an admin can revoke a compromised
account's sessions from the Roles directory (change: `add-session-management`). All of
this goes through the authenticated gRPC `AuthService` (`ListSessions` / `RevokeSession`
/ `RevokeAllSessions` / admin `RevokeAccountSessions`); "this device" is flagged by
matching the access token's `sid` claim. Sign-out-everywhere also calls the existing
`/web/auth/logout` to clear this browser's cookie.

**Residual window (important):** revocation takes effect immediately at the **refresh**
layer — a revoked session can no longer mint new access tokens. But an **access token
already issued stays valid until it expires** (~15 min), because it is a stateless JWT
verified offline (no per-request DB check). So a revoked/compromised session's in-flight
calls keep working until the short TTL lapses; there is no instant cut-off without
shortening the access-token TTL or adding token introspection (out of scope). The admin
revoke is recorded durably in `auth.session_revocation_audit`.

## Deploy (Cloudflare Pages)

`bo.cymbra.app` ships to **Cloudflare Pages**, like the marketing site.
[`back-office-deploy.yml`](../../.github/workflows/back-office-deploy.yml) builds in CI
(the SPA needs `protoc` for `yarn gen`, which the Pages build image can't run) and
uploads the static `dist/` with `wrangler pages deploy`. `public/_redirects` gives the
SPA its history fallback and `public/_headers` the prod security headers above.

The workflow is **dormant until configured**: its deploy job is gated on the
`CF_PAGES_PROJECT` repo variable, so it is skipped (reported success) until a maintainer:

1. Creates a Cloudflare Pages project and maps the custom domain `bo.cymbra.app`.
2. Sets repo **variables** (Settings → Secrets and variables → Actions → Variables):
   `CF_PAGES_PROJECT`, `VITE_GRPC_WEB_URL` (e.g. `https://api.cymbra.app`),
   `VITE_WEB_AUTH_URL` (same registrable domain — see the same-site constraint above),
   and optionally `VITE_GOOGLE_CLIENT_ID`.
3. Sets repo **secrets**: `CLOUDFLARE_API_TOKEN` (scope "Cloudflare Pages: Edit") and
   `CLOUDFLARE_ACCOUNT_ID`.

Then every push to `main` touching `apps/back-office/**` (or a manual **Run workflow**)
builds and deploys. The same-site constraint above is a **hard** requirement:
`bo.cymbra.app` and `VITE_WEB_AUTH_URL` must share `cymbra.app`, or the refresh cookie
is not first-party.

## Moderator onboarding

The access model is: **admins** grant/revoke roles; **moderators** work the review
queue. Bootstrapping a fresh environment (sequence this with the catalog-moderation
backend rollout so the queue is populated and the hub isn't empty indefinitely):

1. **Seed the first `music/admin`** with the ops bootstrap — `backend/scripts/seed_admin.sh`
   (documented in that change's task 1.4). There is no self-service path to admin by
   design; the first one is seeded out-of-band.
2. That admin signs in to the console and, from **`/roles`**, grants `moderator` (or
   `admin`) in the `music` scope to each teammate by user id. Every grant/revoke is
   written to the append-only `role_grants` audit table and shown in the Roles view.
3. **Moderators** sign in → the queue (`/`) lists pending scores in review priority;
   they open a row, preview it, and **Accept**/**Reject** (calls `SetModerationStatus`,
   recording `reviewed_by`/`reviewed_at`). Accepted scores become servable in the app hub.

Role changes take effect on the moderator's **next token refresh** (roles ride in the
access token), not instantly — see the residual-window note above.

## Preview (notation)

`ScorePreview.vue` is the isolated preview seam. Today it shows metadata and confirms
the fetched bytes; the notation renderer (compile the app's Rust `layout_systems` to
wasm + a JS/SVG SMuFL painter, so it matches the app exactly) is a deferred module that
drops in behind this component — or is swapped for a JS fallback if the wasm cost is
too high.

## Not yet wired (deferred)

- The wasm notation renderer.
- Full Google OIDC button (the token-exchange path is wired; the GIS button needs a
  configured client id).
