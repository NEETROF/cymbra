# Tasks — add-lingua-connected-clients

## 1. Extension — account and session

- [x] 1.1 Connect-ES gRPC-web transport in `apps/lingua-extension`: an adapted clone of `apps/back-office/src/lib/transport.ts` + `api.ts` (auth / single-flight refresh / session-expiry interceptors, test seam), backend URL from a build variable
- [x] 1.2 Token storage: access token in `chrome.storage.session`, refresh token in `chrome.storage.local`; session resumption at startup (refresh → a new `TokenPair`); vitest coverage of the cycle
- [x] 1.3 Account UI (icon popup + settings page, Cymbra tokens): "Continue with Google" (`chrome.identity.launchWebAuthFlow` → `SignInOidc`), email/password (`SignInLocal`), signed-in state (email maskable), sign-out (`Logout` + token purge, local state intact) — icon-popup account section done; the standalone settings page + email display fold into the §3 stats page; sign-up, verification, password reset and Sign in with Apple move to `add-lingua-account-parity`
- [x] 1.4 Dev doc: the `chrome-extension://` origin is stable via the manifest key; add the dev origin to `CYMBRA_ALLOWED_WEB_ORIGINS` in the local environment only

## 2. Extension — outbox and synchronisation

- [x] 2.1 Push — **as shipped, no outbox**: every sync is a full idempotent upload of the local statuses, declared level and deck (LWW server-side, replaying a batch is a no-op), in bounded batches of 500 with the device id generated at install (`src/sync/sync.ts` header, `getOrCreateDeviceId`). The versioned op-log of the original plan was not built: the full upload makes first sign-in an ordinary sync and needs no offset resumption
- [x] 2.2 Cursor pull + application to the local store (client-side LWW symmetric to the server's); the existing cross-tab propagation (`storage.onChanged`) triggered by pulled changes
- [x] 2.3 First-sign-in merge: full push of the pre-account state (original timestamps preserved), then a pull of the merged snapshot — not a special case since 2.1 uploads everything on every sync; the two-device convergence is covered by the server's `backend/lingua/tests/convergence.rs`, the client side by the restore → apply → backup tests of `test/sync.spec.ts`
- [x] 2.4 Background orchestration: sync on service-worker wake, after a batch of mutations and on side-panel open; a discreet sync-state indicator + a manual action in settings; no request at all while signed out
- [x] 2.5 Stats: local aggregation per day → `UpsertDailyStats` with the device_id; vitest coverage. **As shipped the language is the literal `en`** (`src/state/dailystats.ts`, `sync.ts` pushStats): the per-language key comes with the studied-language programme

## 3. Extension — stats screen

- [x] 3.1 Stats screen (extension page, Cymbra tokens): series per day × language (words learned, reviews, exposures), 7/30/90-day ranges; signed in = consolidated `GetStats` (scope "all devices"), otherwise local aggregates (scope "this device"); a note that agent sessions are excluded
- [x] 3.2 UI-string lint extended to the new screens (account, sync, stats): no occurrence of "lemma" (say "dictionary form", "distinct words")

## 4. Safari — Apple and Google through the host app

- [x] 4.1 Host app sign-in sheet (SwiftUI, Cymbra styling, French copy): Sign in with Apple first (`ASAuthorizationController`) and Continue with Google (`ASWebAuthenticationSession` with PKCE on the iOS OAuth client, no SDK), opened through the app's URL scheme with the requested provider; on success the id_token is written once to the App Group and the reader is told to go back to Safari; a cancel shows nothing, a failure is worded per provider, never as a password error
- [x] 4.2 `SafariWebExtensionHandler`: `auth.takeIdToken` returns the pending id_token once, deletes it, and discards one older than five minutes; `nativeMessaging` permission in the safari manifest only; XCTest for the single read and the expiry
- [x] 4.3 Extension on Safari: Apple and Google offered through a Safari provider source behind `availableProviders` (capability define); their buttons open the host app through its URL scheme; the popup, the account page and the event page's wake collect a pending id_token and sign in through the existing `Session` (`SignInOidc`, sync scheduled, error categories); vitest for the collection and the expiry path
- [x] 4.4 Spike first: a gRPC-web call from Safari's event page to the backend (CORS for the per-install `safari-web-extension://` origin); if Safari blocks it, forward the extension's RPCs through the native handler
- [x] 4.5 Configuration: `com.cymbra.lingua` added to `CYMBRA_APPLE_AUDIENCE`; a Google OAuth iOS client for `com.cymbra.lingua` added to `CYMBRA_GOOGLE_AUDIENCE` and set as the project's `LINGUA_GOOGLE_CLIENT_ID`; Sign in with Apple and App Group entitlements on the app and the extension
- [x] 4.6 Manual check on Safari (Mac and iPhone): email sign-up, verification, reset and handle in the extension; Apple and Google through the host app land on the same Cymbra account as on Chrome; a word marked in Chrome on the Mac visible in Safari on the iPhone, a card created in Safari reviewable on desktop, consolidated stats correct
- [x] 4.7 Internal TestFlight against production; App Store review pass re-run (Sign in with Apple present, privacy labels updated: account data + synced user content)
- [x] 4.8 **After publication (from `add-lingua-account-parity`)**: create a `com.cymbra.lingua.web` Services ID grouped under `com.cymbra.lingua`, register the browser extension's return URLs on it (`https://figfjglfdiffocldficbimecjnhnkhkh.chromiumapp.org/`, `https://4c0d4d892737d7bb66cf280aafecc32150587ff4.extensions.allizom.org/`), add it to `CYMBRA_APPLE_AUDIENCE` and build the extension with `LINGUA_APPLE_CLIENT_ID=com.cymbra.lingua.web` — Apple's consent screen then says « Cymbra Lingua » instead of « Cymbra Music » (it shows the primary App ID's published app); no account is lost, Apple's user identifier being team-scoped

## 5. Gates and finishing

- [x] 5.1 vitest green on `apps/lingua-extension`; `python3 scripts/check_ci_units.py --list` confirms every touched unit is still watched
- [x] 5.2 Final `openspec validate add-lingua-connected-clients --strict` + spec updates if implementation moved a contract
