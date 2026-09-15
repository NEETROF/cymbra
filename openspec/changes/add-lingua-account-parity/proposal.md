# add-lingua-account-parity — Cymbra Lingua: create an account, verify it, reset it, and sign in with Apple from the extension

## Why

The extension can sign in to an **existing** Cymbra account (Google, email/password) but
cannot **create** one: there is no sign-up, no email verification, no password reset, and
no Sign in with Apple. A new reader has to install Cymbra Music (or use the site) just to
get an account, an unverified account is told "wrong email or password", and anyone who
created their Cymbra account with Apple on Music — often behind "Hide My Email" — cannot
sign in to Lingua at all. Music already has the full set; the backend already serves every
RPC it needs. This change brings the extension to parity.

Apple was excluded from the extension by `add-lingua-connected-clients` (design D2: "the
App Store rule applies to apps only"). That reasoning holds for the *rule*, but misses
**account continuity**: an Apple-only Cymbra account has no other way in. And the usual
blocker — Apple forces `form_post` when email/name scopes are requested, which
`launchWebAuthFlow` cannot read — does not apply to us: Cymbra ID resolves an OIDC sign-in
by `(provider, subject)` only and never reads the email claim, so Apple can be asked for an
id_token **with no scope**, returned in the URL fragment exactly like Google's.

## What Changes

- **Account creation by email** in the extension: `SignUpLocal` (with the browser's UI
  locale, so the email is in the reader's language), password-policy and duplicate-email
  errors shown in plain words.
- **Email verification by code**: the emailed code is entered in the extension
  (`VerifyEmail`), with a resend (`ResendVerification`); a successful verification signs
  the reader in directly when the password is still held in memory, otherwise returns to
  sign-in. A local sign-in rejected because the email is unverified leads to the code
  step instead of a "wrong password" message.
- **Forgotten password**: request (`RequestPasswordReset`, same confirmation whether or not
  the email exists) then code + new password (`ResetPassword`), then back to sign-in.
- **Sign in with Apple** through `launchWebAuthFlow` (no scope, `response_mode=fragment`,
  Services ID as client id) → `SignInOidc(audience="lingua")`. The same Apple ID lands on
  the same Cymbra account as in Music. Gated by a spike validating that Apple accepts the
  extension's redirect URL on the Services ID; if it does not, an allow-listed relay on
  the Cymbra backend receives Apple's `form_post` and hands the id_token to the extension.
- **A full-page account surface** (`account.html`, opened in a tab) hosts sign-up, code,
  and reset: the browser-action popup is destroyed whenever it loses focus — i.e. the
  moment the reader switches to their mailbox for the code. The popup keeps quick sign-in
  (Google, Apple, email) and links to the page; the onboarding tab offers account
  creation as an optional, skippable step. Local-first is untouched: no wall, ever.
- **Provider availability per browser**: Google and Apple are shown only when configured in
  the build **and** the browser exposes `identity.launchWebAuthFlow`. Firefox for Android
  has no identity API → email only there. Safari stays out of scope (its sign-in lives in
  the container app, `add-lingua-apple`).
- **Errors in plain words**: every auth failure maps to a category and a user-facing
  message; a raw gRPC/Connect string never reaches the UI, and a provider failure is never
  worded as a password error.
- **A handle before the account is kept** (found on the first production test): Cymbra
  ID's orphan reaper deletes handle-less accounts after 24 h, so — like Music — every
  sign-in to an account without a handle leads to "Choisis ton pseudo" (availability
  checked live, saved with `UpdateAccount`); leaving deletes the handle-less account. The
  popup shows `@handle`, or a call to choose one.
- **Amends `add-lingua-connected-clients`** (in flight, not archived): design D2 and the
  "Extension sign-in over gRPC-web bearer" requirement no longer enumerate "two methods";
  they defer to `lingua-account` for the method list.

## Capabilities

### New Capabilities
- `lingua-account`: account lifecycle in the Lingua browser extension — email sign-up,
  verification by code (resend, sign-in after verification, unverified sign-in routed to
  the code step), password reset by code, Sign in with Apple via `launchWebAuthFlow`,
  provider availability per browser, the full-page account surface with popup/onboarding
  entry points, credentials never persisted, and plain-language errors. _Server behaviour
  (sign-up, verification, reset, OIDC, policy, throttling) is consumed from `backend-auth`
  / `user-account` unchanged; the session transport and token storage are
  `add-lingua-connected-clients`' `lingua-sync`._

### Modified Capabilities
_None in `openspec/specs/`. `lingua-sync` is not archived yet: its in-flight requirement
text is amended in place inside `add-lingua-connected-clients` (see What Changes)._

## Impact

- **Products**:
  - **Lingua** (new): extension account page, popup/onboarding entry points, Apple flow,
    session methods, error mapping.
  - **Cymbra ID** (consumed as-is): `SignUpLocal`, `VerifyEmail`, `ResendVerification`,
    `RequestPasswordReset`, `ResetPassword`, `SignInLocal`, `SignInOidc` (Google + Apple),
    `UserService` (`GetAccount`, `CheckHandleAvailability`, `UpdateAccount`,
    `DeleteAccount`) for the handle,
    transactional emails already branded "Cymbra" (not Music). **No proto change.**
  - **Music / Live / back office / site**: untouched. The site's Apple **Services ID** is
    reused (return URLs added on Apple's side, no site code change).
- **Tree**: `apps/lingua-extension` (`src/state/session.ts`, `src/background.ts`, new
  `src/account/`, `src/popup/`, `src/onboarding/`, `build.mjs`, `env.d.ts`). Only if the
  spike fails: a relay route in `backend/server` under the existing `/web/auth/*` path
  (already routed by the Caddyfile).
- **Build/env**: new build variable `LINGUA_APPLE_CLIENT_ID` (Services ID). Manual,
  outside the repo: Apple Services ID return URLs for the Chromium and Firefox redirect
  URLs; `CYMBRA_APPLE_AUDIENCE` must contain the Services ID (already required by the
  site); the Google client / `CYMBRA_GOOGLE_AUDIENCE` / `CYMBRA_ALLOWED_WEB_ORIGINS` items
  from `add-lingua-connected-clients` still apply.
- **CI**: no new unit — `apps/lingua-extension` is watched by `lingua-extension-check`
  (and `backend/server` by the backend lanes if the relay is needed); vitest extended.
- **Out of scope**: linking identities / merging a social account into
  an existing one (Music's collision flow), account deletion (the site already has it),
  extension UI localisation (the extension is French-only today — only the emails follow
  the browser language), Safari.
