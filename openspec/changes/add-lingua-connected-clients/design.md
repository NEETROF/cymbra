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
The container app signs in over **native tonic** (`SignInOidc`/`SignInLocal` — no
gRPC-web: it is an app, not a browser), with tokens in the Keychain and the refresh
shared with the Safari extension through the App Group (a single account per device; the
extension consumes the app's session through the native handler — no OAuth flow inside
Safari). As soon as a third-party login (Google) is offered on iOS, **Sign in with Apple
must be offered too** (App Store guideline): the backend already supports it
(`CYMBRA_APPLE_AUDIENCE`) and the app uses the native `ASAuthorizationController`. Button
order: Apple first on iOS, in line with review expectations.

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

## Risks / Trade-offs

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
3. The Apple app next (internal TestFlight against the dev backend, then App Store review
   — Sign in with Apple present, privacy labels up to date).
4. Rollback: signing out (or removing `CYMBRA_LINGUA_DATABASE_URL` server-side) drops the
   clients back to local-only — their default mode; the local schemas do not migrate
   destructively, so a "synced" client keeps working on its own.

## Open Questions

- The exact cadence of background sync in the extension (on service-worker wake +
  `chrome.alarms`? an op threshold?) — to be measured during dogfooding, with no impact
  on the protocol.
- Sharing the session between the app and the Safari extension through the App Group:
  does the extension fetch the tokens through the native handler on every call, or is
  there an App Group copy with invalidation? To be settled at implementation (the
  contract — a single account per Apple device — does not move).
- Should a "Sync now" button be exposed, or should it stay silent (a discreet indicator
  only)? Leaning: an indicator plus a manual action in settings, never friction while
  reading.
