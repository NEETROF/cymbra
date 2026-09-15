# Tasks — add-lingua-account-parity

## 1. Apple spike and manual configuration

- [x] 1.1 [manual] On the site's Apple Services ID (`PUBLIC_APPLE_CLIENT_ID` = `com.cymbra.bo.web`), add the return URLs `https://figfjglfdiffocldficbimecjnhnkhkh.chromiumapp.org/` (Chromium dev id) and the Firefox URL read from `identity.getRedirectURL()` (`https://<hash>.extensions.allizom.org/`) — Chromium and Firefox domains + return URLs registered and saved 2026-09-15
- [x] 1.2 [manual] Spike (design D3): run a hand-built scope-less authorize URL (`response_type=code id_token`, `response_mode=fragment`, `state`, `nonce`) through `launchWebAuthFlow` on the dev id; confirm the id_token arrives in the fragment, that `SignInOidc(audience="lingua")` accepts it against the dev backend, and that the resolved account is the one a Music Apple sign-in uses (compare identities in the back office `/users`). Record the outcome in design.md — D3 confirmed, or D4 activated (then §5 applies) — **D3 confirmed 2026-09-15** against production: Apple accepted the chromiumapp.org return URL, answered in the fragment, and `SignInOidc(audience="lingua")` signed in from Chrome on macOS
- [x] 1.3 [manual] Check the running backend's `CYMBRA_APPLE_AUDIENCE` contains the Services ID (`docker inspect`, not `printenv`); release builds set `LINGUA_APPLE_CLIENT_ID` — proven by the production Apple sign-in (the verifier only accepts listed audiences); production also needed `lingua` added to `CYMBRA_ALLOWED_AUDIENCES` (done 2026-09-15)

## 2. Session and background

- [x] 2.1 `src/state/auth-errors.ts`: pure `authErrorFromCode` (Connect code → `unauthenticated | alreadyExists | rateLimited | failedPrecondition | invalidArgument | notFound | unavailable | unknown`, `DEADLINE_EXCEEDED` → `unavailable`, a failed fetch → `unavailable`), mirroring Music's mapping; vitest table test
- [x] 2.2 `Session`: add `signUp`, `verifyEmail`, `resendVerification`, `requestPasswordReset`, `resetPassword` (no tokens, locale passed through) and `signInWithProvider` over `getGoogleIdToken` / a new `getAppleIdToken` dependency; failures surface as categories (`AccountError`), a provider failure is persisted as `{provider, kind}`; vitest with a fake `AuthService` client
- [x] 2.3 Apple request helpers (pure, `src/state/oidc.ts`): authorize-URL builders (Apple: Services ID, redirect URL, no scope, `code id_token`, `fragment`; Google moved here, both with random `state` + `nonce`) and a fragment parser that rejects a `state` mismatch and ignores `code`; `getAppleIdToken` wired in `background.ts`; vitest
- [x] 2.4 Cancel detection for `launchWebAuthFlow` (Google and Apple): a user-closed window or a provider cancel error resolves as "cancelled" — no RPC, nothing written under `SIGNIN_ERROR_KEY`; vitest for both providers
- [x] 2.5 Background messages `account:signUp`, `account:verifyEmail`, `account:resendVerification`, `account:requestPasswordReset`, `account:resetPassword`, `account:signInApple`, `account:providers` (`{google, apple}` = client id configured × `identity.launchWebAuthFlow` present), dispatched by the unit-tested `src/account/host.ts`; every successful sign-in schedules a sync; replies carry categories, never error strings (`account:state` now answers `{ok, state}`, the stats view follows)
- [x] 2.6 `build.mjs` + `env.d.ts` + `vitest.config.ts`: `LINGUA_APPLE_CLIENT_ID` → `__APPLE_CLIENT_ID__` (empty by default = Apple hidden); README account section documents it next to `LINGUA_GOOGLE_CLIENT_ID`

## 3. Account page

- [x] 3.1 `src/account/` (`account.html`, `account.css`, `account.ts`) added to the build (entry + static copies, both variants), Cymbra tokens only; views sign-in / sign-up / code / forgot / reset / signed-in driven by a controller (`flow.ts`) and a pure renderer (`view.ts`), both unit-tested like the review controller/view
- [x] 3.2 Sign-up → code → automatic `SignInLocal`: password held only in page memory and dropped after use; pending email in `storage.session`; a reload resumes on the code step and a verification then returns to sign-in with the email prefilled; resend action; vitest asserts the pending store never receives the password
- [x] 3.3 Forgot password → reset (code + new password) → sign-in with the email prefilled; the request confirmation is identical whether or not the account exists
- [x] 3.4 Unverified sign-in (`failedPrecondition`) on the page moves to the code step; deep link (`account.html#verify`, `#signup`, `#forgot`) opens the matching view — the email travels in `storage.session` (pending email), never in the URL
- [x] 3.5 Copy per design D7 (context × category), French like the rest of the extension (`src/account/copy.ts`); vitest lint: no account surface reads an error's `message` and the background replies with categories; no "lemme"/"lemma" in the new strings

## 4. Entry points

- [x] 4.1 Popup: "Continuer avec Apple" button (Apple window tears the popup down → reuse the persisted-error path, now a category), Google/Apple hidden per `account:providers`, links "Créer un compte" / "Mot de passe oublié ?" open the account page; an unverified sign-in opens the page on the code step with the email only
- [x] 4.2 Onboarding: a skippable section offering account creation (opens `account.html#signup`) or "Plus tard"; hidden when already signed in; skipping changes nothing else

## 5. Apple relay fallback (only if 1.2 activates D4 — otherwise mark N/A)

- [x] 5.1 N/A — D3 confirmed (task 1.2), no relay needed. Was: `backend/server`: `POST /web/auth/apple/relay` (form-encoded `id_token` + `state`), target from `state` exact-matched against `CYMBRA_EXTENSION_REDIRECT_URIS` (CSV, empty → 404), unknown target → 400, `303` to `<target>#id_token=…&state=…` with `Cache-Control: no-store` + `Referrer-Policy: no-referrer`, body and `Location` never logged; `.env.example` entry; tests for each case
- [x] 5.2 N/A — D3 confirmed. Was: Extension: Apple request switches to `response_mode=form_post` with the relay as `redirect_uri` and `state` carrying `identity.getRedirectURL()`; fragment parsing unchanged; vitest
- [x] 5.3 N/A — D3 confirmed. Was: [manual] Relay return URL on the Services ID; `CYMBRA_EXTENSION_REDIRECT_URIS` set on the backend with the Chromium and Firefox redirect URLs

## 6. Verification

- [x] 6.1 `apps/lingua-extension`: lint, format, typecheck, vitest (new modules 96–100 % line coverage; the extension has no global coverage gate in CI) and `node build.mjs` for both variants green; `python3 scripts/check_ci_units.py --list` still watches every touched unit (no new unit)
- [x] 6.2 [manual] Chrome on macOS against a real backend: sign-up → code from the mailbox → signed in → sync runs; forgotten password end to end; Apple with an Apple ID already used in Music lands on the same account; Google unchanged — validated against production 2026-09-15 (sign-up + code + automatic sign-in, forgotten password, Google, Apple, handle step); sync not exercised: the `lingua` backend module is not deployed in production yet (`add-lingua-connected-clients`)
- [x] 6.3 [manual] Firefox desktop: Google, Apple and email; Firefox for Android: no provider buttons, email sign-up/sign-in/reset work — Firefox desktop validated against production 2026-09-15 (email, Google, Apple; redirect `https://4c0d4d892737d7bb66cf280aafecc32150587ff4.extensions.allizom.org/` registered on the Google client and `com.cymbra.bo.web`); Firefox for Android validated the same day (email only; the per-email sign-in lockout — 5 failures, 15 min — behaved as specified)
- [x] 6.4 `openspec validate add-lingua-account-parity --strict`

## 7. Handle step (found in production: handle-less accounts are reaped after 24 h)

- [x] 7.1 `UserService` client (`tool/gen_proto.sh` + `net/transport.ts`) and the `account/profile.ts` port — `GetAccount`, `CheckHandleAvailability`, `UpdateAccount` with the current version and the other fields sent back, `DeleteAccount` — with categorized errors (`ABORTED` → `conflict`); vitest
- [x] 7.2 Host messages `account:profile`, `account:checkHandle`, `account:setHandle` (fresh read, then write), `account:abandon` (delete only a readable handle-less account, then sign out); vitest
- [x] 7.3 Account page handle step: gate after every sign-in and on opening signed in, local policy, debounced availability with overtaken answers dropped, save, taken/conflict, abandon, no navigation away, focus and caret kept across re-renders; signed-in view shows `@handle`; vitest
- [x] 7.4 Popup: `@handle` in the account row, « Choisir mon pseudo » opening the handle step while it is missing
- [x] 7.5 CI: `lingua-extension-check` watches the proto directories the extension generates from (auth-port, user-port, lingua)
- [ ] 7.6 [manual] Production: a new email account picks a handle and is still there the next day; an existing handle-less account is asked at sign-in; a Music account with a handle is not asked
