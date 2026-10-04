# add-lingua-connected-accounts — Cymbra Lingua: link sign-in methods from the extension, as in Cymbra Music

## Why

A reader who created their Cymbra account with Google cannot sign in to Lingua where the
browser has no Google flow — Firefox for Android, so the Boox e-ink tablet the reader uses
for books. Cymbra ID resolves a Google sign-in by `(provider, subject)`, never by email, so
nothing links the Google account to the reader's address. On that browser:

- « Créer un compte » with the same email silently creates a **second, empty** Cymbra
  account;
- « Mot de passe oublié ? » sends nothing — the reset only exists for a password that was
  set — and says so in the same words as for any address, by design (no enumeration).

The only way out today is Cymbra Music's **Connected accounts** screen (« Définir un mot
de passe »), found in dogfooding on 2026-10-04. The extension has no equivalent:
`add-lingua-account-parity` left linking out of scope. The backend already serves every
RPC it needs, for any audience.

## What Changes

- **Connected accounts in the extension**, on the account page (a tab, which survives the
  reader leaving for their mailbox and a provider's window taking the focus), reached
  from Réglages › Données › Compte and from the account page's signed-in view:
  - the sign-in methods linked to the account (`UserService.ListIdentities`): Google,
    Apple, email and password (with its address), each with the date it was linked;
  - **« Définir un mot de passe »** when the account has none (`AuthService.SetLocalCredential`),
    then the emailed code on the same page (`VerifyEmail`); the reader stays signed in, and
    the email and password then sign in on any browser — Firefox for Android included;
  - **« Lier Google » / « Lier Apple »** where the browser can run that provider's flow
    (`identity.launchWebAuthFlow` and a configured client id, as for sign-in), through
    `AuthService.LinkIdentity` with a freshly minted id_token;
  - **« Retirer »** a method (`AuthService.UnlinkIdentity`), confirmed first, and never the
    last one (anti-lockout).
- **Errors in the reader's words** for the linking cases: a provider already linked to
  another Cymbra account, an address already used by an account, the last method, a weak
  password, a wrong or expired code, an expired session. No account is ever merged.
- **The Compte block hints at it** when signed out: signing in on a browser without the
  method the account was created with goes through linking it elsewhere first.

## Capabilities

### New Capabilities

_None._ The behaviour is Cymbra ID's `account-linking` (legacy for `id-*`), which this change
**consumes** without changing it.

### Modified Capabilities

- `lingua-account` (introduced by `add-lingua-account-parity`, still in flight): ADDED
  requirements for connected accounts in the extension — where they are reached, which
  methods each browser can link, the set-password code step on the account page, the
  guards and the error copy. This change therefore archives after `add-lingua-account-parity`
  (`archiveAfter` in `.openspec.yaml`).

## Impact

- **Products**:
  - **Lingua** — new: the connected-accounts view of the account page, its entry points, the
    background messages and the session's link flow. Consumed: the existing provider flow
    (`launchWebAuthFlow`), the account page and its code step, the Compte block of
    Réglages (from the settings sub-tabs change, PR #660).
  - **ID** — consumed only: `UserService.ListIdentities`, `AuthService.LinkIdentity`,
    `UnlinkIdentity`, `SetLocalCredential`, `VerifyEmail`. They take any authenticated
    caller (`caller(&req)`), whatever the token's audience: **no backend, proto or
    migration change**. The `account-linking` rules (verify-before-bind, last-identity
    guard, no merge) are enforced server-side and apply as they are.
  - **Music / Live / back office / site** — untouched.
- **Tree**: `apps/lingua-extension` only — `src/account/` (flow, view, copy, messages,
  host, profile), `src/state/session.ts`, `src/background.ts`, `src/reading/account-setting.ts`.
- **Order**: implementation lands after PR #660 (the Compte block it links from) is merged.
- **CI**: no new unit — `apps/lingua-extension` is watched by `lingua-extension-check`.
- **Out of scope**:
  - Linking Google or Apple **in Safari**: its providers run in the host app and come back
    as a sign-in; a link hand-off is a follow-up. Safari lists, sets a password and removes.
  - Music's « sign in to link » collision flow after a social sign-in (`account-linking`'s
    last requirement): the extension creates no account through a provider without the
    reader choosing it, and the orphan clean-up stays Music's.
  - Changing the account's email, deleting the account (the site has it).
