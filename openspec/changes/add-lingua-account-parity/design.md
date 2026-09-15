# Design — add-lingua-account-parity

## Context

State of the extension after `add-lingua-connected-clients` §1–§3:

- `src/state/session.ts` owns the token pair (access in `storage.session`, refresh in
  `storage.local`) and exposes `signInLocal`, `signInWithGoogle`, `refresh`, `signOut`.
- `src/background.ts` is the single session owner. The popup drives it over runtime
  messages (`account:state|signInGoogle|signInLocal|signOut`). Google runs
  `chrome.identity.launchWebAuthFlow` with an OpenID implicit request
  (`response_type=id_token`) and reads the id_token from the redirect fragment.
- The popup (`src/popup/`) hosts the only account UI: a Google button and an
  email/password `<details>`. Because Chrome destroys a browser-action popup whenever it
  loses focus, a Google sign-in failure is persisted in `storage.session`
  (`SIGNIN_ERROR_KEY`) and surfaced on the next open.
- The Chromium dev id is pinned (`figfjglfdiffocldficbimecjnhnkhkh`) by a committed dev
  key; Firefox pins `lingua@cymbra.app`.

Backend facts this design relies on (verified in code):

- `AuthService` already serves `SignUpLocal{email,password,locale}`,
  `VerifyEmail{token}`, `ResendVerification{email,locale}`,
  `RequestPasswordReset{email,locale}`, `ResetPassword{token,new_password}`.
  Verification and reset emails carry a **code**, not a link, and are branded "Cymbra".
- A local sign-in with an unverified email fails with `FAILED_PRECONDITION`; a duplicate
  local email with `ALREADY_EXISTS`; a weak password with `INVALID_ARGUMENT`; lockout and
  email throttling with `RESOURCE_EXHAUSTED`; a reset request never reveals whether the
  email exists.
- `sign_in_oidc` verifies the id_token, then calls
  `resolve_or_provision(provider, subject)` — **the email claim is never read**. The
  verifier accepts any `aud` listed in `CYMBRA_APPLE_AUDIENCE` (a CSV).

Platform facts:

- Apple's authorize endpoint requires `response_mode=form_post` when `name` or `email` is
  requested; with no scope, `fragment` is accepted. `launchWebAuthFlow` only exposes the
  final redirect URL, so a `form_post` body is unreachable.
- Apple's `sub` is stable per developer team across bundle id and Services ID: the id
  Music's native flow gets is the id the extension gets.
- `identity.launchWebAuthFlow` exists on Chrome and Firefox desktop, **not** on Firefox for
  Android nor Safari (MDN browser-compat-data).

## Goals / Non-Goals

**Goals**
- A reader can create, verify, recover and use a Cymbra account entirely from the
  extension, with email, Google or Apple, and land on the same account they use in Music.
- No regression of local-first: every account step is optional and skippable.
- One session owner (the background); UI surfaces never call the API.

**Non-Goals**
- Identity linking, collision merge (Music's D7), account deletion from the extension — the
  handle step's "use another account" deleting a handle-less account (D9) is the only one.
- Localising the extension UI (French-only today); only emails follow the browser locale.
- Safari (container-app sign-in, `add-lingua-apple`).
- Nonce binding of OIDC id_tokens server-side (no `nonce` field in `SignInOidcRequest`;
  pre-existing, tracked as a risk below).

## Decisions

### D1 — Account flows live in a full extension page, not in the popup
A new `account.html` (Chromium and Firefox, opened with `tabs.create`) hosts the views
*sign-in*, *sign-up*, *code*, *forgot*, *reset* and *signed-in*. The popup keeps quick
sign-in (Google, Apple, email) plus two links — "Créer un compte" and "Mot de passe
oublié ?" — that open the page on the matching view.

Why: the code step makes the reader leave the browser for their mailbox; a popup is
destroyed at that exact moment and would lose the form. A tab survives focus changes on
both browsers.

Alternatives rejected: **the popup with persisted form state** — it would have to persist
the password to resume an auto sign-in, which D6 forbids, and would still flicker closed
on every focus change; **the side panel** — Chromium only (Firefox has no lateral panel in
this extension), so it would fork the flow per browser.

### D2 — The background stays the only API caller
The page and the popup send messages; the background calls `AuthService` through the
existing `api()` client. New messages: `account:signUp`, `account:verifyEmail`,
`account:resendVerification`, `account:requestPasswordReset`, `account:resetPassword`,
`account:signInApple`, `account:providers`. `Session` gains the matching methods
(`signUp`, `verifyEmail`, … — the non-sign-in ones hold no tokens) and a
`getAppleIdToken` dependency next to `getGoogleIdToken`, so all of it stays testable with
a fake client. A successful sign-in (including the automatic one after verification)
schedules a sync, as today.

Every reply carries a **category**, not a message string (D7), so the background never
decides copy and no surface can show a raw error by accident.

### D3 — Sign in with Apple: scope-less implicit request in the fragment
`getAppleIdToken` opens:

```
https://appleid.apple.com/auth/authorize
  ?client_id=<Services ID>            (build-time LINGUA_APPLE_CLIENT_ID)
  &redirect_uri=<identity.getRedirectURL()>
  &response_type=code id_token        (Apple does not accept id_token alone)
  &response_mode=fragment
  &state=<random>  &nonce=<random>
```

with **no `scope`**. It checks `state` on return, reads `id_token` from the fragment,
discards `code` (never exchanged — that would need a client secret the extension must not
hold), and calls `SignInOidc(id_token, audience="lingua")`.

Why it works for us: Cymbra ID keys OIDC accounts on `(apple, sub)` and ignores email, and
`sub` is per team — so a Music user who signed up with Apple is found, and a brand-new
Apple user is provisioned without an email, exactly as the native flow would.

The Services ID is **the site's** (`PUBLIC_APPLE_CLIENT_ID`), which is already in
`CYMBRA_APPLE_AUDIENCE` for the site to work. Two return URLs are added on it:
`https://<chromium-id>.chromiumapp.org/` and Firefox's
`https://<hash>.extensions.allizom.org/` (read from `identity.getRedirectURL()`).

Gate: **task 1 is a spike** confirming Apple accepts these return URLs (Apple no longer
requires a domain-verification file, but nothing documents a domain we do not own).
Alternatives rejected: **`form_post` + email scope** — the body never reaches the
extension; **Apple's JS SDK popup in an extension page** — its origin must be a registered
`https` return URL, which `chrome-extension://` cannot be.

### D4 — Fallback only if the spike fails: an allow-listed relay under `/web/auth/*`
If Apple refuses the extension return URLs, the Services ID gets one return URL on the
backend, `https://<api host>/web/auth/apple/relay`. The extension then requests
`response_mode=form_post` (still no scope) with that `redirect_uri`, and a `state` that
encodes its own `getRedirectURL()`. The relay (Axum, in `backend/server`, next to the web
auth cookie routes already routed by the Caddyfile's `/web/auth/*`):

1. accepts only `POST application/x-www-form-urlencoded` with `id_token` and `state`;
2. extracts the target from `state` and **exact-matches** it against an allow-list
   (`CYMBRA_EXTENSION_REDIRECT_URIS`, CSV, empty by default → relay disabled with 404);
3. answers `303 See Other` to `<target>#id_token=…&state=…` with `Cache-Control: no-store`
   and `Referrer-Policy: no-referrer`, and never logs the body or the `Location` header.

`launchWebAuthFlow` completes on the final `chromiumapp.org` navigation, as it already
does after Google's redirects. The relay does not verify the token — `SignInOidc` does.

Why the allow-list is exact and mandatory: a relay that redirects anywhere is an id_token
exfiltration primitive. Rejected alternative: a Cloudflare Pages Function on the site —
it would put auth code in a static site with no tests or allow-list config of its own.

### D5 — Provider availability = configured × identity API present
`account:providers` returns `{ google, apple }` where each is true only when its client id
was injected at build time **and** `chrome.identity?.launchWebAuthFlow` is a function at
runtime. Buttons are hidden, not disabled, when false — the same rule as Music, which
hides Google on Linux and Apple off Apple platforms. This replaces today's behaviour where
an unconfigured Google build shows a button that errors.

Feature detection is preferred to `__TARGET__` / `getPlatformInfo()`: Firefox desktop and
Android share the `firefox` build, and detection stays correct if Mozilla ships the API.

### D6 — Credentials are never persisted
- The password typed at sign-up is kept **only in the account page's memory** until the
  code is verified, then used once for `SignInLocal` and dropped. It is never written to
  any `storage` area and never logged.
- The *pending verification email* (not the password) is kept in `storage.session` so a
  reload or a reopened page resumes on the code step; after a reload the password is gone,
  so a successful verification returns to sign-in with the email prefilled (Music's
  `OtpVerifyScreen` does the same when it has no password).
- An unverified sign-in from the **popup** opens the account page on the code step with
  the email only; the password is not handed across.

### D7 — Errors are categories mapped to copy in the surfaces
A pure `authErrorFromCode(code)` mirrors Music's mapping (Connect/gRPC code → `unauthenticated
| alreadyExists | rateLimited | failedPrecondition | invalidArgument | unavailable |
unknown`, `DEADLINE_EXCEEDED` → `unavailable`). The surfaces own the copy, per context:

| Context | Category | Shown |
|---|---|---|
| email sign-in | unauthenticated | wrong email or password |
| email sign-in | failedPrecondition | → code step (no error) |
| Google / Apple | unauthenticated | the provider sign-in failed — never the password copy |
| sign-up | alreadyExists | an account already uses this email + sign-in / forgot links |
| sign-up / reset | invalidArgument | password too weak (policy) |
| code / reset | invalidArgument | invalid or expired code + resend |
| any | rateLimited | too many attempts, try later |
| any | unavailable | cannot reach Cymbra, try again |

A user-cancelled `launchWebAuthFlow` (window closed) is a **cancel**, not a failure: no
RPC, no message, nothing persisted under `SIGNIN_ERROR_KEY`. A vitest lint asserts no
surface renders `error.message` from a Connect error.

### D8 — Onboarding offers, never requires
The onboarding tab (first launch, CEFR level) gains a last, skippable step: "Crée un compte
pour retrouver tes mots sur tous tes appareils" → opens `account.html#signup`, or
"Plus tard". Nothing else in the extension changes when it is skipped.

### D9 — Every account leaves the extension with a handle
Cymbra ID's orphan reaper (`orphan_reap`, hourly, grace `CYMBRA_ORPHAN_REAP_GRACE` = 24 h)
deletes every account whose handle is still null; it exists so Music's abandoned
onboardings do not pile up (`handle-onboarding`). An account created from the extension —
by email, or at a first Google/Apple sign-in — never had a handle, so it was deleted the
next day, while its `auth` credential survived in another schema and blocked the email.
Found on the first production test, after the proposal had put the handle out of scope.

The extension therefore applies Music's gate. After every sign-in, and whenever the account
page opens signed in, it reads the account (`UserService.GetAccount`). Without a handle it
lands on « Choisis ton pseudo »: the 1–15 letters/digits policy checked locally,
availability asked once typing pauses (a slower answer overtaken by newer typing is
dropped), `UpdateAccount` with the account's current version and its other fields sent
back unchanged, `ALREADY_EXISTS` / `ABORTED` shown as taken. « Utiliser un autre compte »
follows Music's rule: a handle-less account is deleted (`DeleteAccount`), an account with a
handle is only signed out, and an account whose profile cannot be read is never deleted on
a guess. The popup shows `@handle`, or « Choisir mon pseudo » while it is missing.

Rejected: sparing accounts with a Lingua session in the reaper — it bends an `id-*` rule for
one product, and a Lingua-only account would still reach Music without a handle.

## Risks / Trade-offs

- **Apple refuses the extension return URLs** → D4 relay, already designed; the spike is
  task 1 so nothing downstream is built on a false assumption.
- **A relay that leaks id_tokens** → exact allow-list, disabled by default, POST only,
  no-store, no logging of body/Location; covered by tests (unknown target → 400, empty
  allow-list → 404, headers asserted).
- **Apple accounts without an email** (scope-less request) → nothing in Cymbra ID or Lingua
  needs the email for an OIDC account today; if a later feature does, it must fetch it
  through linking, not assume the claim.
- **Email sign-up with an address that already belongs to a Google/Apple-only account**
  creates a second account (the backend only rejects an existing *local* credential) →
  same behaviour as Music without its collision flow; out of scope, noted for a later
  `id-*` change.
- **OIDC nonce not bound server-side** (pre-existing) → the extension still generates and
  checks `state`; server-side nonce binding stays a separate Cymbra ID change.
- **Firefox Android readers can only use email** → explicit in the UI (no dead buttons);
  acceptable given ~2 % share.
- **Unverified sign-in from the popup loses the password** → one extra sign-in after
  verification, in exchange for never moving a password across surfaces.

## Migration Plan

1. Spike (manual): add the two return URLs on the site's Services ID, run a hand-built
   authorize URL through `launchWebAuthFlow` on the dev id, exchange the id_token against
   the dev backend with `audience=lingua`, and compare `sub` with a Music Apple account.
   Record the outcome here (D3 confirmed, or D4 activated).
2. Ship behind configuration: an empty `LINGUA_APPLE_CLIENT_ID` hides Apple, so the
   extension can merge before Apple is configured; email flows work as soon as the build
   points at a backend.
3. Rollback: rebuild without `LINGUA_APPLE_CLIENT_ID` (Apple hidden); the email flows only
   call RPCs that already exist, so there is no server state to undo. The relay, if built,
   is disabled by emptying `CYMBRA_EXTENSION_REDIRECT_URIS`.

## Open Questions

- Spike outcome (D3 vs D4) — resolved by task 1.
- Should the signed-in view show the account email? The access token does not carry it; it
  would need `AccountService.GetAccount`. Deferred unless dogfooding asks for it.
