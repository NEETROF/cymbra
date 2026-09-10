# Tasks — add-lingua-connected-clients

## 1. Extension — account and session

- [ ] 1.1 Connect-ES gRPC-web transport in `apps/lingua-extension`: an adapted clone of `apps/back-office/src/lib/transport.ts` + `api.ts` (auth / single-flight refresh / session-expiry interceptors, test seam), backend URL from a build variable
- [ ] 1.2 Token storage: access token in `chrome.storage.session`, refresh token in `chrome.storage.local`; session resumption at startup (refresh → a new `TokenPair`); vitest coverage of the cycle
- [ ] 1.3 Account UI (icon popup + settings page, Cymbra tokens): "Continue with Google" (`chrome.identity.launchWebAuthFlow` → `SignInOidc`), email/password (`SignInLocal`), signed-in state (email maskable), sign-out (`Logout` + token purge, local state intact)
- [ ] 1.4 Dev doc: the `chrome-extension://` origin is stable via the manifest key; add the dev origin to `CYMBRA_ALLOWED_WEB_ORIGINS` in the local environment only

## 2. Extension — outbox and synchronisation

- [ ] 2.1 Local outbox (a versioned op-log in `chrome.storage.local`): every status/card mutation enqueues a timestamped op (device_id generated at install); drained in batches with offset resumption
- [ ] 2.2 Cursor pull + application to the local store (client-side LWW symmetric to the server's); the existing cross-tab propagation (`storage.onChanged`) triggered by pulled changes
- [ ] 2.3 First-sign-in merge: full push of the pre-account state (original timestamps preserved), then a pull of the merged snapshot; test the "two devices with disjoint local states" scenario
- [ ] 2.4 Background orchestration: sync on service-worker wake, after a batch of mutations and on side-panel open; a discreet sync-state indicator + a manual action in settings; no request at all while signed out
- [ ] 2.5 Stats: local aggregation per (day, language) → `UpsertDailyStats` with the device_id; vitest coverage (re-push idempotence)

## 3. Extension — stats screen

- [ ] 3.1 Stats screen (extension page, Cymbra tokens): series per day × language (words learned, reviews, exposures), 7/30/90-day ranges; signed in = consolidated `GetStats` (scope "all devices"), otherwise local aggregates (scope "this device"); a note that agent sessions are excluded
- [ ] 3.2 UI-string lint extended to the new screens (account, sync, stats): no occurrence of "lemma" (say "dictionary form", "distinct words")

## 4. Apple app — native sign-in and sync

- [ ] 4.1 Native sign-in screen (SwiftUI, Cymbra styling): Sign in with Apple (`ASAuthorizationController` → Apple `SignInOidc`) + "Continue with Google" + email/password, over native tonic with the `lingua` audience; tokens in the Keychain
- [ ] 4.2 App ↔ Safari extension session sharing through the App Group (one account per device): the extension syncs under the app's session, no OAuth flow inside Safari; settle tokens-per-handler vs an App Group copy (the design's open question) and document it
- [ ] 4.3 App sync: the same outbox/pull/merge as the extension (shared `lingua-core` types) for statuses, cards and stats; first sign-in merges the device's local state
- [ ] 4.4 The app's stats screen (same rules as 3.1); manual cross-device check: a word marked on the Mac visible on the iPhone, an iOS card reviewable on desktop, consolidated stats correct
- [ ] 4.5 Internal TestFlight against the dev backend; App Store review pass re-run (Sign in with Apple present, privacy labels updated: account data + synced user content)

## 5. Gates and finishing

- [ ] 5.1 vitest green on `apps/lingua-extension`; `python3 scripts/check_ci_units.py --list` confirms every touched unit is still watched
- [ ] 5.2 Final `openspec validate add-lingua-connected-clients --strict` + spec updates if implementation moved a contract
