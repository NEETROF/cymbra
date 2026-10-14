# add-lingua-app-email-signin — design

## Context

See proposal.md — Why. The state this builds on, as `add-lingua-review-translation-fallback` D8
shipped it (#948):

- **The app's page** (`apps/lingua-extension/src/app/`, served under `lingua-app://app/` by
  `ViewController.swift`) mounts the extension's Révision. Its own session is `src/app/account.ts`:
  the extension's `Session` class over a storage area whose refresh token goes to the Keychain
  through the bridge (`auth.readRefresh`, `auth.writeRefresh`, `auth.clear`) and whose access token
  stays in memory. The transport is the extension's gRPC-web one, direct, with
  `Origin: lingua-app://app` — already in `CYMBRA_ALLOWED_WEB_ORIGINS` in production.
- **Signing in** is the bridge's `auth.signIn`: `ViewController.presentAppSignIn` presents
  `AppSignInView` (`Shared (App)/SignInView.swift`) — Apple first, then Google — and answers the page
  with `AppSessionMessage.signInReply`: `{ provider, idToken }`, or `{}` when the sheet was closed.
  The page exchanges the id_token with `Session.signInWithIdToken` (`SignInOidc`, `lingua`
  audience). It is asked on the first press of « Traduction en ligne » without a session, and from
  the account line after a sign-out from the app.
- **The account line** (`src/app/view.ts`, `renderAccountLine`): signed in, « Connecté à Cymbra pour
  la traduction en ligne — Se déconnecter »; signed out from the app, « Se connecter pour la
  traduction en ligne »; otherwise, where online translation is offered, the note « Dans l'app, la
  traduction en ligne demande de te connecter avec Apple ou Google. » (`signInNote`, fr/en/es).
- **The extension's email sign-in** lives in two places that share one controller's messages:
  - `src/account/flow.ts` (`AccountFlow`) and `src/account/view.ts` (`renderAccount`): the account
    page — sign-in, sign-up, the verification-code step, « Mot de passe oublié » → code + new
    password, the handle step, connected accounts, erasure. The flow sends `AccountMessage`s
    (`account:signInLocal`, `account:verifyEmail`, `account:resendVerification`,
    `account:requestPasswordReset`, `account:resetPassword`, …); the background answers them with
    `handleAccountMessage` (`src/account/host.ts`) over its `Session`. Replies carry error
    *categories* (`AuthErrorKind`, `src/state/auth-errors.ts`), worded by `errorCopy(context, kind)`
    (`src/account/copy.ts`) in the interface language — `unauthenticated` → « Email ou mot de passe
    incorrect. », `rateLimited` → « Trop de tentatives. Réessaie dans quelques minutes. »,
    `unavailable` → « Impossible de joindre Cymbra. Vérifie ta connexion et réessaie. »; an
    unverified address (`failedPrecondition`) moves to the code step with the password held in
    memory so a valid code signs straight in.
  - `src/reading/account-setting.ts`: the compact « Compte » block of Réglages, which sends the same
    `account:signInLocal` and opens the account page for every other step.
  The fields already carry `autocomplete` hints: `username`, `current-password`, `new-password`,
  `one-time-code`.

**Server — verified, no change.** `backend/auth-port/proto/auth.proto`: `SignInLocalRequest { email,
password, audience }`. `backend/auth/src/module.rs` `sign_in_local` checks the audience against
`allowed_audiences` (the same check `sign_in_oidc` makes, which the app already passes with
`lingua`), applies the three sign-in counters of `fix-auth-lockout-dos` (per email and address, per
address, per email), answers `Unauthenticated` for wrong credentials, `FailedPrecondition` for an
unverified address and `ResourceExhausted` at a limit, then issues a token pair for the audience.
`VerifyEmail`, `ResendVerification`, `RequestPasswordReset` and `ResetPassword` take no bearer and
no audience. They are the same `AuthService` the app already calls over gRPC-web, so Caddy's routes
and CORS already pass them. Nothing in Cymbra ID has to move.

## Goals / Non-Goals

**Goals:**
- A reader with an email account opens the app's session with it, in the app, on iPhone, iPad and
  Mac.
- One implementation of the email flow: the extension's controller, view, copy and errors.
- The password never leaves the page except to Cymbra ID, and is never stored.

**Non-Goals:**
- Creating an account in the app (see D4).
- Linking methods, setting a password or the handle step in the app: the app's session serves online
  translation only.
- Associated domains or `webcredentials` (see D5).
- Any change to the extension's own sign-in behaviour, or to the server.

## Decisions

### D1 — The form lives in the page, and it is the extension's

Three places were weighed:
- **A native SwiftUI form in the sheet** — a second implementation of the flow, its errors and its
  copy in Swift; and either a native gRPC client in the app or the password crossing the bridge to
  the page. Rejected: two flows drift (the Lingua settings had a copy that missed every addition),
  and the bridge would carry a password it has no reason to see.
- **The page shows its own chooser, with the form inline** — the native Apple button and Google's
  web flow are native; the page would have to call back into the app for them, splitting one choice
  across two surfaces. Rejected.
- **The sheet keeps the choice, the page holds the form** — chosen. The sheet the reader already knows
  gains « Continuer avec un email »; the form is the extension's account page code, mounted in the
  app's page. The password is typed into the page and sent by the page's transport to Cymbra ID, the
  path every other request of the app's session takes.

### D2 — The sheet's third entry answers `{ method: "email" }`

`AppSignInView` gains a third button below Google, styled as a secondary choice (French UI copy:
« Continuer avec un email »; English: « Continue with email »; Spanish: « Continuar con un correo »),
its label in `SignInCopy` (fr/en/es). « email », not « e-mail »: the spelling of the extension's
account copy the panel then shows (« Email ou mot de passe incorrect. »). Pressing it closes the sheet and `auth.signIn` answers
`{ method: "email" }` (`AppSessionMessage.signInReply` gains that case; a closed sheet still answers
`{}`). On the Mac the sheet's frame grows to hold the third button.

In the page, `appAccount.signIn()` reads the reply: an id_token is exchanged as today; `method:
"email"` opens the email panel and the promise settles when the panel does — `true` once signed in,
`false` when the reader closes it. So the first press of « Traduction en ligne » goes on to
translate after an email sign-in exactly as after an Apple one, and the account line's link behaves
the same.

The panel is a modal region of `app.html` over the review (a `<dialog>`), with the account page's
styles and a « Annuler » (French UI copy) that closes it.

### D3 — The extension's `AccountFlow` and `renderAccount`, in an app scope

The panel is `AccountFlow` + `renderAccount`, not the Réglages block: the block hands every step but
the first to the account page — a tab the app does not have — whereas the flow carries the code step
and the reset in one place.

`AccountFlow` gains an option (`scope: "app"`, default the extension's full page) that:
- limits the views to `signin`, `verify`, `forgot` and `reset`;
- leaves out « Créer un compte » on the sign-in view, and sends the code step's « Utiliser un autre
  email » back to sign-in instead of sign-up;
- shows no provider buttons (the sheet ran that choice);
- ends on a successful sign-in with a callback instead of reading the profile: no handle step, no
  signed-in view — the app's session is for online translation, and an account's handle is the
  extension's business.

The flow's `send` is `handleAccountMessage` called in the page over the app's `Session` — the
dispatch the background uses, so the categories and their mapping are the same code. Its deps are
reduced to what the scope sends: providers answer none, there is no hand-off, and the profile port
is never reached (a stub that throws keeps that visible in tests). The locale the requests carry
(`AccountLanguage.locale` of `send-lingua-browser-locale-on-account-emails`) is the interface
language the reader chose in the extension when the shared state names it, else the device's
language tag; the account's stored language wins server-side anyway.

The pending email (the code step's address) lives in the page's memory while the panel is open —
the extension keeps it in `storage.session`, which the page does not have, and the page's own storage
under a custom scheme is not guaranteed (D8 of the fallback change). The password, as in the
extension, is held in memory only for the code step.

### D4 — No account creation in the app

The app signs in to an account; it does not create one with an email. Readers new to Cymbra keep
Apple or Google in the sheet (which the app already offers) or the extension's account page. Creating
an email account in the app would bring the verification of a fresh address, the handle step and
the abandonment rule (a handle-less account is deleted) into an app whose session serves one
feature, for readers who by construction already have the extension. A follow-up can add it if the
dogfood asks for it.

### D5 — Autofill: the hints, measured; no associated domain in this change

The fields keep the extension's hints (`autocomplete="username"`, `"current-password"`,
`"one-time-code"`, `"new-password"`), and `type="email"` / `inputmode="numeric"`.

What iOS and macOS do with them in a `WKWebView` page under a custom scheme is not known yet: Password
AutoFill matches saved credentials to a domain, and `lingua-app://app` has none. Where an email
reader's password was saved also varies: by Safari under the Safari extension's page, or under
`cymbra.app` when they signed in on the site. Associating the app with `cymbra.app`
(`webcredentials:cymbra.app` in both app targets' entitlements, an `apple-app-site-association` file
served by the site with a JSON content type, new provisioning profiles) is a cross-product change
whose effect on a custom-scheme web view is itself unverified. So this change ships the hints only,
and the dogfood (tasks 5.2, 5.3) records what each platform offers: the QuickType « Mots de passe »
key, a domain suggestion, one-time codes from Mail. Typing the password, or picking it from the
Passwords list, is the accepted fallback. If the dogfood finds autofill missing and typing a burden,
associated domains are a follow-up change of their own.

**Outcome on iPhone (dogfood 2026-10-14, iPhone 15 Pro Max, build 535b409d9):** iOS offered no autofill
at all in the app's web view: no « Mots de passe » key, no suggestion, no code from Mail. Guillaume
judged it unimportant: typing is accepted, and associated domains are not needed now. The Mac
(task 5.3) is still to be measured.

### D6 — Coming back from the mailbox keeps the step

The app reloads its page when it returns to the foreground (`linguaForeground`), unless a review is
under way, to read the extension's latest state. A reader fetching a code in Mail would come back to
a reloaded page, the panel gone with its pending email and password. `linguaForeground` therefore
also skips the reload while the email panel is open; the page reloads at the next foreground after
the panel closes. If the app was terminated meanwhile, the reader starts again at sign-in: an
address still unverified brings them back to the code step (the server answers
`FailedPrecondition`), where « Renvoyer le code » is at hand.

### D7 — The deltas modify the in-flight change's text, archived after it

The two requirements this changes are the text `add-lingua-review-translation-fallback` writes into
`lingua-sync` and `lingua-apple-app`; that change is merged but not archived, so `openspec/specs/`
still holds the older text (« The container app SHALL hold neither a session nor learning state »).
ADDED requirements were considered instead and rejected: an added requirement saying the app may
sign in with an email would contradict the in-flight one saying « the email flows SHALL stay in the
extension ». So the deltas are MODIFIED, written on the fallback change's text, and
`.openspec.yaml` sets `archiveAfter: add-lingua-review-translation-fallback`, which
`scripts/openspec_archive_order.py` enforces. The email sign-in's own behaviour is an ADDED
requirement of `lingua-apple-app`, independent of the order.

### D8 — The note and the annex

The note « Dans l'app, la traduction en ligne demande de te connecter avec Apple ou Google. » only
existed because an email account had no way in. It goes, with its three translations (`signInNote`);
where it was shown, the account line shows the sign-in link already used after a sign-out (« Se
connecter pour la traduction en ligne »), so the reader can sign in before pressing the button too.

The privacy annex says the in-app sign-in is « avec Apple ou Google » (French), « with Apple or
Google » (English), « con Apple o Google » (Spanish). It becomes « avec Apple, Google ou votre email et
votre mot de passe », « with Apple, Google or your email and password », « con Apple, Google o tu
correo y tu contraseña ». What is collected does not
change — the email and password already reach Cymbra ID from the extension — so the App Store
privacy answers do not move; `lingua-privacy`'s requirement names no method and is not modified.

### D9 — The line names the account signed in

Dogfood on a real iPhone (2026-10-14): after an email sign-in the line only said « Connecté à Cymbra
pour la traduction en ligne », not which account — and the app's session may be a different account
from the extension's. The line now names it for every method: « Connecté en tant que @handle pour la
traduction en ligne — Se déconnecter » (English « Signed in as @handle for online translation »,
Spanish « Sesión iniciada como @handle para la traducción en línea »).

The handle is read the way the extension's account page reads it: `UserService.GetAccount`, with the
app's own bearer over the same gRPC-web transport (origin `lingua-app://app`, already allowed). It is
a separate read in `src/app/account.ts`, not the panel's profile port, which stays a refusal (D3: no
handle step in the app). Without a handle (an account the reaper has not reached yet), the line falls
back to an email address: the one an email sign-in just used (no request), else the `local`
identity's subject from `UserService.ListIdentities`; without either, or when the read fails, it keeps
the plain line. It is read once after each sign-in and on opening with a session, kept in the page's
memory only, and forgotten on sign-out: nothing new is stored, so `lingua-privacy` does not move.
The name is the reader's data: a text node only.

## Risks / Trade-offs

- [Autofill may offer nothing in the app's web view] → measured in the dogfood on iPhone and Mac;
  typing or the Passwords list is the fallback; associated domains are the follow-up (D5).
- [The sign-in lockout is shared with the extension on the same network] → the server keys its
  counters by email and address, so failed attempts in Safari and in the app add up. That is the
  lockout working as designed; the copy (« Trop de tentatives… ») is the same on both.
- [The flow gains a scope option, a branch the extension never takes] → the scope only removes
  views and links; the specs of `flow.ts` and `view.ts` cover both scopes, and the extension's
  existing specs stay as they are.
- [A page reload loses the pending email] → the reload is skipped while the panel is open (D6);
  after a termination the server's `FailedPrecondition` brings the reader back to the code step.
- [Apple review: sign-in with a password in an app offering Sign in with Apple] → Sign in with Apple
  stays first and equivalent (guideline 4.8 is about offering it, which the app does); no account
  creation is added (D4), so no new account-deletion obligation arises from this change.

## Migration Plan

Ship with the next app build: the page is bundled in the app, so the sheet and the page always change together. Then the annex on the site. No server deployment, no configuration. Rollback: an app build without
the entry; nothing is stored that an older build would misread.
