# add-lingua-connected-clients — Cymbra Lingua: sign-in, sync and stats in the extension and the app

## Why

`add-lingua-backend` shipped a complete server that is **inert for the user**: the
`lingua` audience, three sync services, the LWW protocol — and no client connecting to
any of it. This change closes the loop: Cymbra ID sign-in in the extension and the Apple
app, the client outbox, the pre-account store merge, and the consolidated stats screen.
The founder's real case — Chrome on macOS + Safari on iOS — finally becomes coherent: a
word learned on the Mac is "known" on the iPhone, a card created on mobile is reviewable
in the desktop side panel. The stack's local-first behaviour remains the default — the
extension and the app work entirely without an account, and syncing is **opt-in** at
sign-in.

**Position in the stack** (12 changes): **12th and last**. **Explicit prerequisites:
`add-lingua-extension-review`** (side panel, cards, statuses — the surfaces that gain the
account UI and the outbox), **`add-lingua-apple`** (container app and Safari extension)
and **`add-lingua-backend`** (audience, services, protocol). `add-lingua-firefox` is
covered by ricochet (same extension code; `browser.identity.launchWebAuthFlow` exists).

## What Changes

- **Sign-in in the extension**: a bearer `TokenPair` over gRPC-web (Connect-ES —
  templated on `apps/back-office/src/lib/transport.ts` + `api.ts`: auth /
  single-flight refresh / session-expiry interceptors), rotating refresh token (reuse
  detection with family revocation already exists server-side), the access token in
  `storage.session` and the refresh token in `storage.local`. Two methods: **Google
  OIDC** through `chrome.identity.launchWebAuthFlow` (the client id has been in the
  `CYMBRA_GOOGLE_AUDIENCE` CSV since `add-lingua-backend`) and the existing
  email/password (`SignInLocal`).
- **Apple and Google on Safari, through the host app**: Safari has no
  `identity.launchWebAuthFlow`, so the host app `apps/lingua-apple` runs Sign in with Apple
  (**mandatory as soon as Google is offered**, App Store rule) and Continue with Google
  natively, and hands a single-use id_token to the Safari extension through its native
  handler. The extension exchanges it through `SignInOidc` like on Chrome and owns its
  session like every other variant. Email sign-in, sign-up, verification, reset and the
  handle already run in the Safari extension (`add-lingua-account-parity`).
- **Opt-in sync, local-first preserved**: without an account, nothing changes; signing
  out stops syncing without touching local state.
- **Client outbox + delta pull**: a local op-log of mutations drained in idempotent
  batches, a cursor pull, and LWW application symmetric to the server's; at first
  sign-in the pre-account local store is **merged** (pushed in full with its original
  timestamps, then the merged snapshot is pulled back).
- **A consolidated stats screen** in the extension and the app, fed by `StatsService`
  when signed in and by local state otherwise; the displayed scope ("all devices" /
  "this device"); a note that agent sessions are excluded.
- **The Claude Code plugin stays local-only**: the `~/.lingua/` store gains no network
  path (its sync is a dedicated later change); the "no network connection" invariant
  stays tested.
- UI vocabulary: the word "lemma" never appears in the new screens (account, sync,
  stats) — say "dictionary form", "distinct words".

## Capabilities

### New Capabilities
- `lingua-sync` (client half): an optional account with local-first preserved, extension
  sign-in (gRPC-web bearer, Google OIDC + email/password, tokens stored by volatility),
  Apple and Google sign-in on Safari through id_tokens obtained natively by the host app,
  the pre-account store merge at first sign-in, and the Claude Code
  plugin left out of sync. _The server half (audience, module, protocol, server-side
  privacy, purge) is the `add-lingua-backend` change._
- `lingua-stats` (client half): the stats screen in the extension and the app —
  consolidated when signed in, local otherwise, with the scope displayed — and the
  jargon-free vocabulary rule. _Server-side storage and the consolidated read are the
  `add-lingua-backend` change._

### Modified Capabilities
_None. The local stack's `lingua-*` capabilities are untouched: their local mode stays
the default and this change adds the connected mode on top. The platform
(`backend-auth`, flags, analytics) is consumed as-is._

## Impact

- **Products**: Lingua (extension: account UI, outbox, stats screen; Apple host app: the
  native Apple and Google sheet for the Safari extension); **Cymbra ID consumed**
  (Google/Apple OIDC, email sign-up, verification and reset, rotating refresh); **Lingua
  backend consumed** (`add-lingua-backend`); Music / Live / back office / site:
  **untouched**.
- **Tree**: `apps/lingua-extension` (Connect-ES transport, account UI, outbox, stats
  screen, the Safari provider source), `apps/lingua-apple` (SwiftUI sign-in sheet, App
  Group id_token handoff, native messaging handler).
- **Env/deploy**: no server code; production configuration gains `com.cymbra.lingua` in
  `CYMBRA_APPLE_AUDIENCE` and the app's Google OAuth client in `CYMBRA_GOOGLE_AUDIENCE`.
  Dev doc: add the dev extension origin to `CYMBRA_ALLOWED_WEB_ORIGINS` in the local
  environment only.
- **New dependencies**: the Google Sign-In SDK in `apps/lingua-apple` (Swift Package
  Manager).
- **CI**: no new unit — `apps/lingua-extension` and `apps/lingua-apple` are already
  watched by their lanes in the stack; vitest extended (session, outbox, stats); the
  TestFlight pass re-run (Sign in with Apple, privacy labels: account data + synced user
  content).
- **Out of scope (later changes)**: Claude Code plugin sync (confidential transcripts —
  local-only restated here), card media/image sync (encrypted, the score-cache pattern),
  the Lingua back-office console and the `lingua` role/scope, TextProfile/book catalogue,
  a Lingua account-management website (the existing site already covers Cymbra ID
  management).
