# Tasks — add-lingua-account-parity

## 1. Apple spike and manual configuration

- [ ] 1.1 [manual] On the site's Apple Services ID (`PUBLIC_APPLE_CLIENT_ID`), add the return URLs `https://figfjglfdiffocldficbimecjnhnkhkh.chromiumapp.org/` (Chromium dev id) and the Firefox URL read from `identity.getRedirectURL()` (`https://<hash>.extensions.allizom.org/`)
- [ ] 1.2 [manual] Spike (design D3): run a hand-built scope-less authorize URL (`response_type=code id_token`, `response_mode=fragment`, `state`, `nonce`) through `launchWebAuthFlow` on the dev id; confirm the id_token arrives in the fragment, that `SignInOidc(audience="lingua")` accepts it against the dev backend, and that the resolved account is the one a Music Apple sign-in uses (compare identities in the back office `/users`). Record the outcome in design.md — D3 confirmed, or D4 activated (then §5 applies)
- [ ] 1.3 [manual] Check the running backend's `CYMBRA_APPLE_AUDIENCE` contains the Services ID (`docker inspect`, not `printenv`); release builds set `LINGUA_APPLE_CLIENT_ID`

## 2. Session and background

- [ ] 2.1 `src/state/auth-errors.ts`: pure `authErrorFromCode` (Connect code → `unauthenticated | alreadyExists | rateLimited | failedPrecondition | invalidArgument | unavailable | unknown`, `DEADLINE_EXCEEDED` → `unavailable`), mirroring Music's mapping; vitest table test
- [ ] 2.2 `Session`: add `signUp`, `verifyEmail`, `resendVerification`, `requestPasswordReset`, `resetPassword` (no tokens, locale passed through) and `signInWithApple` over a new `getAppleIdToken` dependency; failures surface as categories; vitest with a fake `AuthService` client
- [ ] 2.3 Apple request helpers (pure): authorize-URL builder (Services ID, redirect URL, no scope, `code id_token`, `fragment`, random `state` + `nonce`) and fragment parser that rejects a `state` mismatch and ignores `code`; wire `getAppleIdToken` in `background.ts`; vitest
- [ ] 2.4 Cancel detection for `launchWebAuthFlow` (Google and Apple): a user-closed window resolves as "cancelled" — no RPC, nothing written under `SIGNIN_ERROR_KEY`; vitest for both providers
- [ ] 2.5 Background messages `account:signUp`, `account:verifyEmail`, `account:resendVerification`, `account:requestPasswordReset`, `account:resetPassword`, `account:signInApple`, `account:providers` (`{google, apple}` = client id configured × `identity.launchWebAuthFlow` present); every successful sign-in schedules a sync; replies carry categories, never error strings
- [ ] 2.6 `build.mjs` + `env.d.ts`: `LINGUA_APPLE_CLIENT_ID` → `__APPLE_CLIENT_ID__` (empty by default = Apple hidden); README build section documents it next to `LINGUA_GOOGLE_CLIENT_ID`

## 3. Account page

- [ ] 3.1 `src/account/` (`account.html`, `account.css`, `account.ts`) added to the build (entry + static copies, both variants), Cymbra tokens only; views sign-in / sign-up / code / forgot / reset / signed-in driven by a pure view-state reducer (vitest)
- [ ] 3.2 Sign-up → code → automatic `SignInLocal`: password held only in page memory and dropped after use; pending email in `storage.session`; a reload resumes on the code step and a verification then returns to sign-in with the email prefilled; resend action; vitest asserts no storage area ever holds the password
- [ ] 3.3 Forgot password → reset (code + new password) → sign-in with the email prefilled; the request confirmation is identical whether or not the account exists
- [ ] 3.4 Unverified sign-in (`failedPrecondition`) on the page moves to the code step; deep link (`account.html#verify?email=…`, `#signup`, `#forgot`) opens the matching view
- [ ] 3.5 Copy per design D7 (context × category), French like the rest of the extension; vitest lint: no surface renders a Connect error's `message`, and no "lemme"/"lemma" in the new strings

## 4. Entry points

- [ ] 4.1 Popup: "Continuer avec Apple" button (Apple window tears the popup down → reuse the persisted-error path), Google/Apple hidden per `account:providers`, links "Créer un compte" / "Mot de passe oublié ?" open the account page; an unverified sign-in opens the page on the code step with the email only
- [ ] 4.2 Onboarding: a last, skippable step offering account creation (opens `account.html#signup`) or "Plus tard"; skipping changes nothing else

## 5. Apple relay fallback (only if 1.2 activates D4 — otherwise mark N/A)

- [ ] 5.1 `backend/server`: `POST /web/auth/apple/relay` (form-encoded `id_token` + `state`), target from `state` exact-matched against `CYMBRA_EXTENSION_REDIRECT_URIS` (CSV, empty → 404), unknown target → 400, `303` to `<target>#id_token=…&state=…` with `Cache-Control: no-store` + `Referrer-Policy: no-referrer`, body and `Location` never logged; `.env.example` entry; tests for each case
- [ ] 5.2 Extension: Apple request switches to `response_mode=form_post` with the relay as `redirect_uri` and `state` carrying `identity.getRedirectURL()`; fragment parsing unchanged; vitest
- [ ] 5.3 [manual] Relay return URL on the Services ID; `CYMBRA_EXTENSION_REDIRECT_URIS` set on the backend with the Chromium and Firefox redirect URLs

## 6. Verification

- [ ] 6.1 `apps/lingua-extension`: lint, vitest (coverage ≥ 80 %), `node build.mjs` for both variants green; `python3 scripts/check_ci_units.py --list` still watches every touched unit (plus `cargo fmt`/`clippy -D warnings`/tests if §5 ran)
- [ ] 6.2 [manual] Chrome on macOS against a real backend: sign-up → code from the mailbox → signed in → sync runs; forgotten password end to end; Apple with an Apple ID already used in Music lands on the same account; Google unchanged
- [ ] 6.3 [manual] Firefox desktop: Google, Apple and email; Firefox for Android: no provider buttons, email sign-up/sign-in/reset work
- [ ] 6.4 `openspec validate add-lingua-account-parity --strict`
