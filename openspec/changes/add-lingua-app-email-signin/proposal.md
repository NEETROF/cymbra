# add-lingua-app-email-signin — sign in to the Cymbra Lingua app with an email and a password

## Why

Dogfood of the Cymbra Lingua app on a real iPhone (2026-10-14): the app now holds its own Cymbra
session for « Traduction en ligne » in its review (`add-lingua-review-translation-fallback` D8,
#948), but it can only be opened with the Apple or Google sheet. A reader whose account was created
with an email and a password — in the extension or in another Cymbra app — cannot sign in to the app at all;
the app only says so (French UI copy: « Dans l'app, la traduction en ligne demande de te connecter
avec Apple ou Google. »). Online translation is a premium feature: the readers most likely to have
it are the ones who made an account, and the extension's account page creates email accounts. Guillaume
chose to add email + password sign-in to the app, reusing the extension's own email sign-in rather
than writing a second one.

## What Changes

- **The app's sign-in sheet gains a third entry**, below Apple and Google (French UI copy:
  « Continuer avec un email »). Choosing it closes the sheet and the app's page shows the email
  sign-in.
- **The email sign-in is the extension's own**, in the app's page: the same form, the same flow and
  the same copy as the extension's account page — email and password, the verification-code step
  for an address not yet verified, « Mot de passe oublié ? » with its emailed code and new password.
  It talks to Cymbra ID through the app's own session (`SignInLocal` for the `lingua` audience, the
  same as the extension), never through the extension's.
- **Account creation stays out of the app.** The app's email form signs in to an existing account;
  a new reader keeps Apple or Google in the app, or creates an email account in the extension. The
  « Créer un compte » link is not shown in the app.
- **Errors are the extension's localised ones** (French, English, Spanish): wrong email or password,
  too many attempts, Cymbra unreachable, a bad code — never a raw technical error.
- **Autofill**: the fields keep the extension's `autocomplete` hints (`username`,
  `current-password`, `one-time-code`, `new-password`). Whether iOS and macOS offer saved passwords
  in the app's web view is measured in the dogfood; no associated domain is added by this change.
- **The password is never stored**: it lives in the page's memory only for the verification step,
  exactly as in the extension; what the app keeps is still only the refresh token, in its Keychain.
  Sign-out is unchanged.
- **Coming back from the mailbox keeps the step**: the app no longer reloads its page on returning to
  the foreground while the email sign-in is open, so the code step is still there.
- **The in-app note goes**: where the app used to say that online translation needs Apple or Google,
  it now offers to sign in (French UI copy: « Se connecter pour la traduction en ligne »), the
  entry already shown after a sign-out from the app.
- **The privacy annex** (French, English, Spanish) no longer says the in-app sign-in is « with Apple
  or Google »: it names the email and password too. What is collected does not change.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-sync`: « Apple and Google sign-in on Safari through the host app » — the container app's
  own session may also be opened with an email and a password; email account creation stays in the
  extension.
- `lingua-apple-app`: added — the app signs in with an email and a password through the extension's
  own flow; modified — « The app reviews the deck the extension keeps » names the email sign-in's
  requests among the only network requests the app makes.

Both modified requirements are the text `add-lingua-review-translation-fallback` writes; that change
is not archived yet, so this one sets `archiveAfter: add-lingua-review-translation-fallback` in
`.openspec.yaml` and modifies its text (design D7). `lingua-privacy` is not modified: its requirement
says the app needs the reader « to sign in inside the app » without naming the methods; only the
site's wording moves.

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The Apple container app (`apps/lingua-apple`, iPhone, iPad,
  Mac): its sign-in sheet and the page it serves (the `app` build of `apps/lingua-extension`). The
  extension's account code (`src/account/*`) is reused, not changed in behaviour: it gains an option
  the app uses to leave out account creation and the handle step.
- **Cymbra ID — consumed, unchanged.** `AuthService.SignInLocal`, `VerifyEmail`,
  `ResendVerification`, `RequestPasswordReset` and `ResetPassword` already serve the `lingua`
  audience; the sign-in lockout and limits (`fix-auth-lockout-dos`) apply as they do to the
  extension. The page's origin `lingua-app://app` is already allowed. **No server change.**
- **Public site — content only.** `apps/site`: one sentence of the Lingua annex of the privacy
  policy, in French, English and Spanish.
- Music, Live, the back office and the agent plugin are untouched.

**Code.** `apps/lingua-extension/src/app/*` (`account.ts`, `app.ts`, `app.html`, `view.ts`, a new
email panel), `src/account/flow.ts` and `view.ts` (an app scope), `src/i18n/{fr,en,es}/app.ts`;
`apps/lingua-apple/Shared (App)/SignInView.swift`, `ViewController.swift`,
`LinguaSignIn/Sources/LinguaSignIn/AppSessionStore.swift`, `SignInFlow.swift`;
`apps/site/src/pages/confidentialite.md`, `en/privacy.md`, `es/privacidad.md`.

**Data.** None. No new stored item; the email waiting for its code is kept in the page while the
panel is open, never the password.

**CI.** `lingua-extension-check`, `lingua-apple-build` and `site-check` already watch these paths;
no new unit.
