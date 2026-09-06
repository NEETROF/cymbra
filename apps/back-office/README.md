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
yarn gen:wasm   # build the notation renderer wasm (needs wasm-pack + wasm32 target)
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
- `yarn gen:wasm` — build the notation renderer wasm from `crates/musicxml-wasm` into
  `src/wasm/pkg/` (gitignored). Needs `wasm-pack` + `wasm32-unknown-unknown`.
- `yarn test` — Vitest component/store tests (its own gate, outside the Flutter/Rust CI).
- `yarn build` — type-check + production build (run `yarn gen` + `yarn gen:wasm` first).

## Debugging gRPC-web calls

gRPC-web always returns **HTTP 200** — the real status is the `grpc-status` trailer
(`0` = OK, else an error; e.g. `16` = unauthenticated), and the Network tab's
Response is just the length-prefixed **protobuf wire bytes** (the hex/ASCII dump),
not readable JSON. So don't debug from the Network tab. Aids wired in
`src/lib/transport.ts`:

- **Console trace** (the reliable one): every call logs a collapsed group with its
  method and **decoded request/response JSON** — `✓ <Service>/<Method>` on success,
  `✗ … status <Code> (<n>): <message>` on failure. No extension needed.
- **gRPC-Web Developer Tools** Chrome extension
  ([SafetyCulture](https://github.com/SafetyCulture/grpc-web-devtools)): optional
  DevTools panel decoding each call. The interceptor that feeds it
  (`src/lib/grpc-devtools.ts`) posts the events it listens for; no extension →
  harmless no-op. It lives in a separate **"gRPC Web" DevTools tab** (not Network),
  and you must reload the page after opening it. Dev build only.

### Tracing on the deployed site (`bo.cymbra.app`)

The console trace is **always on in dev** and **opt-in in a production build**. It is
strictly off for every user unless an operator turns it on in their own browser:

```js
localStorage.setItem("cymbra:grpc-trace", "1"); // then reload
// localStorage.removeItem("cymbra:grpc-trace"); // + reload to turn it back off
```

The flag is read once at startup, so toggling needs a reload; when it's on in prod,
the console prints a one-line `[cymbra] gRPC trace ON …` banner. This is the safe way
to inspect live traffic — it needs **no CORS/cookie changes on prod**. (Pointing a
local `yarn dev` at the prod backend would require adding `localhost` to the prod CORS
allowlist, which is a lasting security weakening — don't.)

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
signed-in non-moderator → access-denied; `/users` requires `admin`. This is UX only —
**every RPC is independently role-guarded server-side**, so the gate can't be bypassed.

### Cookie sessions (no token in JS storage)

The refresh token is **never** exposed to page JavaScript, and the access token is
**memory-only** (nothing in `localStorage`/`sessionStorage`/non-`HttpOnly` cookie), so
an XSS can't exfiltrate a replayable session (change: `add-web-auth-cookies`).

- Sign-in / refresh / sign-out go over the **web-auth HTTP surface** (`VITE_WEB_AUTH_URL`,
  the backend's HTTP port), not gRPC. The server sets/rotates/clears the refresh token
  in an `HttpOnly; Secure; SameSite=Strict; Path=/web/auth` cookie and returns the access
  token in the JSON body. See `src/lib/web-auth.ts` (injectable seam, like `lib/api.ts`)
  over the shared **`packages/web-auth`** package (`@cymbra/web-auth`, a Yarn `portal:`):
  the web-auth client and the Google / Apple sign-in composables live there, one
  implementation for this console and the public site (`apps/site`).
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
account's sessions from that account's page, `/users/{user_id}` (change:
`add-session-management`). All of
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
   and optionally the SSO client ids (see **Social sign-in** below).
3. Sets repo **secrets**: `CLOUDFLARE_API_TOKEN` (scope "Cloudflare Pages: Edit") and
   `CLOUDFLARE_ACCOUNT_ID`.

The console is **versioned like the other modules** via release-please
(`component: back-office`, tag `back-office-vX.Y.Z`). Deploy is **release-driven, not
merge-driven**: the workflow triggers on a `back-office-v*` tag (cut when the
back-office Release PR is merged) or a manual **Run workflow** — never on every merge to
`main`. The same-site constraint above is a **hard** requirement: `bo.cymbra.app` and
`VITE_WEB_AUTH_URL` must share `cymbra.app`, or the refresh cookie is not first-party.

## Social sign-in (Google & Apple)

Both are **optional and inert until their client id is set** — with none configured the
email/password form is the sole path (the buttons don't render). Each provider mints an
id_token in the browser that the app exchanges via `SignInOidc`; the backend picks the
provider from the token's issuer and verifies its `aud` against the matching
`CYMBRA_*_AUDIENCE` (both accept a comma-separated set).

- **Google** — set `VITE_GOOGLE_CLIENT_ID` to the Google **Web** OAuth client id (the
  same value as the backend's `CYMBRA_GOOGLE_AUDIENCE`). Add `https://bo.cymbra.app` (and
  `https://cymbra.app` for the site's sign-in) to that client's _Authorized JavaScript
  origins_ in Google Cloud Console.
- **Apple** — set `VITE_APPLE_CLIENT_ID` to a **Services ID** (e.g. `com.cymbra.bo.web`),
  **not** the app bundle id, and optionally `VITE_APPLE_REDIRECT_URI` (defaults to the SPA
  origin — we use `https://bo.cymbra.app/signin`). In the Apple Developer portal, on that
  Services ID: enable "Sign in with Apple" → Configure → pick the primary App ID
  (`com.cymbra.music`) → register the domains `bo.cymbra.app`, `cymbra.app` and the Return
  URLs `https://bo.cymbra.app/signin`, `https://cymbra.app/redeem`, `https://cymbra.app/account`
  (comma-delimited lists; the site reuses the same Services ID) → Done → Continue → Save. Apple's
  current flow needs **no** domain-association file / `.well-known` hosting. Add the
  Services ID to the backend's `CYMBRA_APPLE_AUDIENCE` (alongside the app bundle,
  comma-separated) so web tokens verify — no backend code change needed.

## Moderator onboarding

The access model is: **admins** grant/revoke roles; **moderators** work the review
queue. Bootstrapping a fresh environment (sequence this with the catalog-moderation
backend rollout so the queue is populated and the hub isn't empty indefinitely):

1. **Seed the first `music/admin`** with the ops bootstrap — `backend/scripts/seed_admin.sh`
   (documented in that change's task 1.4). There is no self-service path to admin by
   design; the first one is seeded out-of-band.
2. That admin signs in to the console and, from **`/users`**, opens each teammate's
   account and grants `moderator` (or `admin`) in the `music` scope — no user id to
   type. Every grant/revoke is written to the append-only `role_grants` audit table
   and shown in that account's role history.
3. **Moderators** sign in → the queue (`/`) lists pending scores in review priority;
   they open a row, preview it, and **Accept**/**Reject** (calls `SetModerationStatus`,
   recording `reviewed_by`/`reviewed_at`). Accepted scores become servable in the app hub.

Role changes take effect on the moderator's **next token refresh** (roles ride in the
access token), not instantly — see the residual-window note above.

## Preview (notation)

`ScorePreview.vue` renders each score's notation **read-only**, faithfully to the app,
by reusing the app's own layout engine compiled to WebAssembly. This fulfils the
`moderation-console` "Read-only score preview" requirement and closes
`add-moderation-back-office` task 4.8 (change: `add-wasm-notation-preview`).

Pipeline:

1. **`crates/musicxml-wasm`** — a thin `wasm-bindgen` wrapper over
   `cymbra-musicxml-core` exposing one read-only entry point, `render(bytes, width)`,
   which runs the app's `parse` + `layout_systems` and returns the geometry
   (`{ document, systems }`). It depends only on the pure core crate — never on
   `rust_lib_music` (cpal/midir/frb are native-only).
2. **`yarn gen:wasm`** (`tool/gen_wasm.sh`) — `wasm-pack build --target web` into the
   gitignored `src/wasm/pkg/`. Needs the wasm toolchain:
   `rustup target add wasm32-unknown-unknown` + `wasm-pack`. Like `yarn gen`, CI builds
   it (the Cloudflare Pages image has no Rust toolchain).
3. **`composables/useScoreRenderer.ts`** — the isolated renderer seam: lazy-`import()`s
   and instantiates the wasm module once (a separate bundle chunk, so nothing pays for
   it until a preview is opened), then paints the geometry with the SVG SMuFL painter
   (`lib/notation/painter.ts` + `smufl.ts`), mirroring the app's `partition_painter.dart`.
   The render lifecycle is an `Async<T>`; any load/instantiate/render failure degrades to
   a placeholder — never a page error. Swapping the wasm renderer for a pure-JS fallback
   is a one-file change behind `lib/notation/wasm.ts`.

The Bravura SMuFL font (SIL OFL 1.1, the app's exact `Bravura.otf`) is served
same-origin from `public/fonts/` via an `@font-face` in `styles.css`, under
`font-src 'self'`. Instantiating wasm needs the narrow **`wasm-unsafe-eval`** token in
`script-src` (set in the Vite CSP meta plugin) — it does not permit JS `eval()`.

### Playback (Play/Pause + animated playhead)

A **Play/Pause** control (no other interaction) plays the score's audio and sweeps a
playhead over the notation, highlighting the sounding notes — the same engine the app's
play mode uses:

- **`crates/audio-wasm`** — wraps the app's pure-Rust `rustysynth` synth to render the
  whole score to one interleaved-stereo PCM buffer (`render(scoreBytes, sf2, sampleRate)`),
  driven by the shared schedule from `cymbra-musicxml-core`. Depends only on the two pure
  crates; **Web Audio** (`AudioBufferSourceNode`) is the output sink here, replacing the
  app's native `cpal`. Built by `yarn gen:wasm` into `src/wasm/pkg-audio/`.
- **Timing** lives in the core crate: `schedule(bytes)` (exposed from `musicxml-wasm`)
  mirrors the app's `notationToTimedNotes` — onset-sorted notes + per-measure start times
  - tempo. `composables/useScorePlayer.ts` owns Web Audio + a `requestAnimationFrame`
    clock (`elapsedMs`); `composables/usePlayhead.ts` positions the cursor, toggles the
    `.playing` highlight on the `data-note`-tagged heads, and auto-scrolls. Both the
    measure→cursor maths (`measureAt`) and the sounding-notes maths (`playingNoteIds`) are
    pure and unit-tested.
- **SoundFont**: the app's exact `UprightPianoKW-20220221.sf2` (CC0, ~57 MB), served by
  the **backend delivery route** `GET /soundfonts/{id}` (change `add-soundfont-delivery`)
  from a private OVH bucket — _not_ bundled (57 MB > Cloudflare Pages' 25 MiB/file limit).
  `lib/audio/soundfont.ts` fetches it **on demand** (only when a moderator hits Play) with
  the caller's **access token** (`Authorization: Bearer`), and persists it in the **Cache
  API** so it downloads at most once and is never unloaded. The URL is
  `${VITE_WEB_AUTH_URL}/soundfonts/upright-piano-kw` by default (override with
  `VITE_SOUNDFONT_URL`); the route's origin is already allowed by the app's CSP `connect-src`.
- A small **spinner** in the transport (`ScorePreview`) shows while that first-play
  download is in flight (`audio` = loading).
- Playback degrades gracefully: no `AudioContext`, a failed SoundFont fetch, or a render
  error surface as a small "Audio unavailable" note, never a thrown error or a broken page.

> **Deploy dependency.** Playback in prod needs `add-soundfont-delivery` deployed (the
> private `cymbra-soundfonts` bucket + the `/soundfonts/*` route) so the SoundFont is
> reachable — this resolves the old Cloudflare Pages 25 MiB blocker. In dev, point
> `VITE_WEB_AUTH_URL` (or `VITE_SOUNDFONT_URL`) at a backend running the route with the
> bucket configured. The rest of the console deploys unaffected.

v1 draws staves, clefs, key/time signatures, barlines, note heads, stems, flags,
beams, accidentals, augmentation dots, rests and ledger lines. Expression/dynamics
directions, lyrics, ties, slurs and tuplet brackets are best-effort follow-ups (not
part of the preview contract).

## Not yet wired (deferred)

- Full Google OIDC button (the token-exchange path is wired; the GIS button needs a
  configured client id).
