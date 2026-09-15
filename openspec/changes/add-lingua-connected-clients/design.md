# Design — add-lingua-connected-clients

## Context

The local stack settled two things that constrain this change: (1) every surface has a
**versioned local state** whose schemas share the `lingua-core` types — "so that merging
is mechanical" (decisions from `add-lingua-extension-review` and `add-lingua-apple`):
this change is that merge; (2) the extension's transport was already settled during
exploration: **gRPC-web bearer** (Connect-ES, the back-office template), never the
`/web/auth` cookie (SameSite=Strict + exact-origin CORS is hostile to extensions by
design).

The server ships entirely with `add-lingua-backend`: the `lingua` audience in
configuration, `CYMBRA_ALLOWED_WEB_ORIGINS`, the three `cymbra.lingua.v1` services, the
protocol (op-log, LWW, cursor, snapshot) and the purge. The protocol and server-side
privacy decisions (the allow-list) are taken there and are not relitigated here — this
design covers only the **client side**: how you sign in, where the tokens live, how the
outbox and the merge are orchestrated, and the stats screen.

## Goals / Non-Goals

**Goals:**
- A signed-in user finds the same word statuses, the same cards and consolidated stats
  on every device (Chromium/Firefox extension, Safari + the Apple app).
- Without an account nothing changes: the stack's local-first behaviour stays the
  default; syncing is opt-in at sign-in.
- The UI always reads the local store; syncing is an invisible background exchange.

**Non-Goals:**
- Claude Code plugin sync (`~/.lingua/`) — transcripts are confidential; a later change
  with its own design (loopback PKCE CLI auth).
- Card media/images (v1: the schema's `media` field does not sync; encrypted sync comes
  with image capture).
- Anything server-side (protocol, schema, purge, CORS) — shipped by
  `add-lingua-backend`.
- Multiple accounts per device, sharing between users.

## Decisions

### D1 — Extension transport: gRPC-web bearer, tokens split by volatility
Connect-ES + `createGrpcWebTransport`, cloned from the back-office template
(`transport.ts`: an `Authorization: Bearer` auth interceptor, **single-flight** refresh
with a single retry, session-expiry as a last resort; `api.ts`: the `setClientsForTest`
seam). Tokens are stored by volatility: the **access token in `chrome.storage.session`**
(in memory, purged when the browser closes, never on disk); the **refresh token in
`chrome.storage.local`** (the session survives a restart — and it is the one that
rotates and is server-revocable). Rejected alternatives: both in `storage.local` — a
persisted access token is only worth a few minutes yet lingers on disk; both in
`session` — a re-login on every restart, unacceptable for a daily reading extension.

### D2 — OIDC in the extension: `chrome.identity.launchWebAuthFlow`, client id from the CSV
Google only on the extension side: `launchWebAuthFlow` opens the OAuth flow (redirect
`https://<ext-id>.chromiumapp.org/`), the extension gets the `id_token` and calls
`SignInOidc(provider=google, audience=lingua)`. The client id (Web type, chromiumapp.org
redirect) is already in the **`CYMBRA_GOOGLE_AUDIENCE` CSV** (shipped by
`add-lingua-backend`, the exact precedent being the desktop client). Email/password is
the existing `SignInLocal`, with the same reset screens as the site. Firefox:
`browser.identity.launchWebAuthFlow` exists — same code. **No Sign in with Apple in the
extension**: the App Store rule applies to apps only; on Safari, sign-in lives in the
container app (D3).

### D3 — Apple app: native sign-in, Sign in with Apple mandatory
The host app (`apps/lingua-apple`, the minimal app of `add-lingua-apple`) gains the
account. It signs in with **Sign in with Apple** (native `ASAuthorizationController`),
**Continue with Google** (the Google Sign-In SDK, with an iOS/macOS OAuth client for
`com.cymbra.lingua`) and **email/password** — Apple first on iOS, in line with review
expectations; the App Store rule requires Apple as soon as Google is offered. Email
accounts are created and recovered natively too (`SignUpLocal` + `VerifyEmail` with a
code, `RequestPasswordReset` + `ResetPassword`), because `cymbra.app` has no sign-up or
reset page to send people to.

Calls go over **gRPC-web with Connect-Swift**, generated from
`backend/auth-port/proto/auth.proto`: the same protocol and endpoint as the extension
(D1), so there is no second transport to route and secure. Tokens live in the Keychain,
in an access group shared with the Safari extension's native handler (D6). The app holds
**no learning state** — reading, decks, review and sync stay in the extension
(`add-lingua-apple` D4); the session is the one thing it owns.

### D4 — Merging the pre-account store at first sign-in: upload then merge, local stays the display authority
At a device's first sign-in, the local state (statuses, cards, the local stack's stats)
is **pushed in full** as timestamped ops (the original local timestamps are preserved),
and the client then pulls the merged snapshot. The server-side merge is the protocol's
ordinary LWW (`add-lingua-backend`) — first sign-in is not a special case, just a large
outbox. After that the model stays **local-first**: the UI always reads the local store;
syncing is a background exchange (on service-worker wake, after a batch of mutations, on
side-panel open). Signing out stops syncing without touching local state.

### D5 — The Claude Code plugin stays local-only — restated, not forgotten
The `~/.lingua/` store gains **no** network path in this change: transcripts are employer
code, and the "no network connection" invariant of `add-lingua-agent` (the
`lingua-agent-capture` spec) is tested. Its sync will be a dedicated change (loopback
PKCE CLI auth, explicit per-machine opt-in). Accepted consequence: extension and plugin
statuses keep diverging — that was already the local stack's state, and the mechanical
merge stays possible when the day comes (same `lingua-core` types).

### D6 — The Safari extension borrows the app's session; only native code refreshes
This settles the former open question. On Safari the extension keeps **no refresh token
and no sign-in form**. When it needs an access token, its event page asks the extension's
native handler (`browser.runtime.sendNativeMessage` → `SafariWebExtensionHandler`), which
reads the shared Keychain and returns a short-lived access token — refreshing first if it
has expired. A `session.invalidate` message lets the extension report a token the server
rejected.

Refreshing stays native because the server **rotates refresh tokens and revokes the whole
family on a replay** (`backend/auth/src/session.rs`): two holders refreshing the same token
would sign the user out everywhere. The app and the handler run in separate processes, so a
refresh takes an App Group file lock and re-reads the Keychain inside it — whoever comes
second finds the new pair and does not refresh. The extension's `Session` gains a native
source behind the same seam (a capability define, as in `add-lingua-apple` D2); its sync
engine does not change. Rejected: an App Group copy of the tokens for the extension to
read — it leaves the refresh question open, and the handler is only a message away.

### D7 — Safari's network path is proven before it is relied on
The extension's gRPC-web calls leave from Safari's event page, whose origin
(`safari-web-extension://<uuid>`) is random per install and so cannot be listed in
`CYMBRA_ALLOWED_WEB_ORIGINS`. Whether the manifest's host permission exempts the event page
from CORS on Safari, as it does on Chromium, is checked first (task 4.6). If it does not,
the handler forwards the extension's RPCs as opaque HTTP requests: `URLSession` has no
CORS, and the D6 bridge already exists.

## Risks / Trade-offs

- [A double refresh (app and extension) revokes the token family] → a single owner:
  native code only, serialised by an App Group lock (D6), with a test that runs two
  refreshes concurrently.
- [Safari blocks the event page's cross-origin calls] → spike first; the native forward
  is the fallback (D7).
- [Refresh token in `storage.local`: readable by local malware] → the same exposure as
  any browser-profile secret; mitigated by rotation plus reuse detection (the family is
  revoked on the first replay) and an accessible `RevokeAllSessions`.
- [A bulky initial push at first sign-in (years of statuses)] → bounded batches plus
  outbox-offset resumption (carried by the `add-lingua-backend` protocol); op order
  preserves timestamps, so an interruption resumes without corruption.
- [`chrome-extension://<id>` origin: the id differs between dev (unpacked) and store] →
  the published id is stable (key in the manifest); in dev, the local origin is added to
  `CYMBRA_ALLOWED_WEB_ORIGINS` in the dev environment only. Firefox
  (`moz-extension://<uuid>`, random per install): to be confirmed at integration, the
  documented fallback being a fixed origin id via `browser_specific_settings`.
- [Extension ↔ Claude Code plugin divergence maintained] → accepted and restated (D5);
  documented in the stats UI ("excluding agent sessions").
- [Wrong client clock: a skewed device "wins" LWW conflicts] → tie-break and clamping
  server-side (`add-lingua-backend`); the worst case is still fixable with one click (set
  the status again).

## Migration Plan

1. Prerequisite: `add-lingua-backend` deployed and provisioned (audience, origins,
   `CYMBRA_LINGUA_DATABASE_URL` active).
2. Extension first (account UI + outbox + stats) — the founder's Chrome-on-macOS
   dogfooding.
3. The Apple host app next: the Safari network spike (D7) first, then native sign-in and
   the session bridge (D6); internal TestFlight against the dev backend, then App Store
   review — Sign in with Apple present, privacy labels up to date.
4. Rollback: signing out (or removing `CYMBRA_LINGUA_DATABASE_URL` server-side) drops the
   clients back to local-only — their default mode; the local schemas do not migrate
   destructively, so a "synced" client keeps working on its own.

## Open Questions

- The exact cadence of background sync in the extension (on service-worker wake +
  `chrome.alarms`? an op threshold?) — to be measured during dogfooding, with no impact
  on the protocol.
- Should a "Sync now" button be exposed, or should it stay silent (a discreet indicator
  only)? Leaning: an indicator plus a manual action in settings, never friction while
  reading.
