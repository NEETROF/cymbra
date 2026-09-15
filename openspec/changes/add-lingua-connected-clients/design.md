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
the existing `SignInLocal`. Firefox desktop: `browser.identity.launchWebAuthFlow`
exists — same code. ~~No Sign in with Apple in the extension~~ — **amended by
`add-lingua-account-parity`**: the App Store rule indeed applies to apps only, but an
Apple-only Cymbra account (created in Music) has no other way in, so the extension also
offers Sign in with Apple (scope-less `launchWebAuthFlow`), plus email sign-up,
verification and password reset. The method list is specified by `lingua-account`. On
Safari, the same flows run in the extension; only Apple and Google come from the host app
(D3).

### D3 — On Safari, the host app provides Apple and Google; the extension keeps its session
Safari has no `identity.launchWebAuthFlow`, so the extension cannot run a provider flow.
Everything else already runs in the Safari extension exactly as on Chrome — email sign-in,
sign-up, verification, password reset and the handle (`add-lingua-account-parity`) — and
its network path is direct (D7). The host app (`apps/lingua-apple`) therefore fills only
the provider gap. It runs **Sign in with Apple** (native `ASAuthorizationController`) and
**Continue with Google** (the authorization-code flow with PKCE in
`ASWebAuthenticationSession`, on an iOS OAuth client for `com.cymbra.lingua`), Apple first —
the App Store rule requires Apple as soon as Google is offered — and hands the resulting
**id_token** to the extension (D6). Google needs no SDK: an iOS client has no secret, the
session intercepts its reversed-client-id redirect, and one code exchange yields the
id_token — without the Google Sign-In SDK's keychain state, which already broke Music's
macOS sign-in after a re-signature. The client id is a build setting; without one, the app
reports Google as unavailable and the Safari extension hides its button. The extension calls
`SignInOidc(audience="lingua")` with it, the same exchange as a Chrome sign-in, and owns the
session like every variant (D1). The app holds neither learning state nor a session.

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

### D6 — The id_token handoff between the host app and the Safari extension
On Safari, « Continuer avec Apple » and « Continuer avec Google » open the host app through
its URL scheme, naming the provider. The app shows the native sheet and, on success, writes
the id_token once to the App Group container, then tells the reader to go back to Safari.
The extension's event page asks its native handler for a pending token (`auth.takeIdToken`,
over `browser.runtime.sendNativeMessage` → `SafariWebExtensionHandler`) when the popup or
the account page opens and when the event page wakes. The handler returns it once, deletes
it, and discards one older than five minutes (an Apple id_token lives ten). The extension
then runs the ordinary `SignInOidc` and schedules a sync.

Rejected: the app owning the session and lending access tokens to the extension (this
change's earlier D6). Once `add-lingua-account-parity` shipped the account flows in every
extension, it would have duplicated those screens natively and needed a cross-process
refresh lock, because the server revokes a whole token family when a refresh token is
replayed (`backend/auth/src/session.rs`). With the extension as the only session owner,
that risk does not arise. Rejected too: the native handler presenting the sheet — a Safari
web extension's handler has no UI.

### D7 — Safari's network path is direct, proven by a spike
The extension's gRPC-web calls leave from Safari's event page, whose origin
(`safari-web-extension://<uuid>`) is random per install and so cannot be listed in
`CYMBRA_ALLOWED_WEB_ORIGINS`. The spike (task 4.6, iOS 26.5 simulator) called
`AuthService.Refresh` with a bogus token from the event page against
`https://api.cymbra.app`: the server's answer came back readable (HTTP 200,
`grpc-status: 16`) although the response carried no `Access-Control-Allow-Origin`. Safari
exempts the event page from CORS through the manifest's host permission, as Chromium does,
so the extension calls the backend directly. Forwarding the RPCs through the native handler
(`URLSession` has no CORS) stays the fallback should a later Safari change this; the manual
pass (task 4.8) re-checks it on macOS and on a device.

## Risks / Trade-offs

- [An id_token left behind in the App Group] → read once and deleted by the handler,
  discarded after five minutes, and readable only by the app and its extension (D6).
- [Apple and Google on Safari take two hops, Safari → app → Safari] → accepted: email needs
  none and the app says when to go back; provider detection turns the in-extension flow
  back on by itself if Safari ever ships `identity.launchWebAuthFlow`.
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
3. The Apple host app next: the Safari network spike (D7), then the native Apple and
   Google sheet and the id_token handoff (D3, D6); internal TestFlight against the dev
   backend, then App Store review — Sign in with Apple present, privacy labels up to date.
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
