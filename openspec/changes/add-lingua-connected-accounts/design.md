# Design — add-lingua-connected-accounts

## Context

State of the extension this builds on:

- **The session lives in the background** (`src/state/session.ts`, `src/background.ts`).
  Surfaces reach it over runtime messages (`account:*`, `src/account/messages.ts`),
  dispatched by `handleAccountMessage` (`src/account/host.ts`). Replies carry an
  `AuthErrorKind` category, never a message (`src/state/auth-errors.ts`).
- **Providers**: `Session.signInWithProvider` mints an id_token through
  `identity.launchWebAuthFlow` (implicit OpenID request, token read from the fragment),
  then calls `SignInOidc`. `availableProviders` offers a provider only with a client id
  and `launchWebAuthFlow`. Safari has no `launchWebAuthFlow`: its providers run in the
  host app and come back through a hand-off as a **sign-in**.
- **The account page** (`account.html`, `src/account/flow.ts` + `view.ts`) hosts the
  steps that need a page: sign-up, the email code, the reset, the handle, « Gérer mes
  données ». It is a tab, so it survives the reader leaving for their mailbox and a
  provider's window taking the focus. The pending email of a code step sits in
  `storage.session` (`PENDING_EMAIL_KEY`); a password never does.
- **Réglages › Données › Compte** (`src/reading/account-setting.ts`, PR #660) signs in
  and out in every host of Réglages, and opens the account page for the rest
  (`openPage("account.html#…")`, which the background performs, so it works from the
  in-page drawer too).

Backend facts (verified in code, unchanged by this change):

- `UserService.ListIdentities` → `[{ provider: "local"|"google"|"apple", subject,
  linked_at }]`; for `local`, the subject is the email.
- `AuthService.LinkIdentity{id_token}`, `UnlinkIdentity{provider, subject}`,
  `SetLocalCredential{email, password, locale}` all take the caller from the bearer
  (`caller(&req)`) with **no audience check**: a `lingua` session may call them.
- `LinkIdentity` → `ALREADY_EXISTS` when the identity belongs to another account.
  `UnlinkIdentity` → `FAILED_PRECONDITION` for the last identity. `SetLocalCredential` →
  `INVALID_ARGUMENT` for a weak password, `ALREADY_EXISTS` when the account already has a
  password or the address already has a credential; it **parks** the submission under the
  emailed token and binds it only in `VerifyEmail` (which may then fail `ALREADY_EXISTS`
  if the address was taken meanwhile).
- `ResendVerification` looks up a stored credential by email: a parked set-password has
  none, so **it cannot be resent** — a new submission sends a new code.

## Goals / Non-Goals

**Goals:** the extension reaches parity with Music's Connected accounts for listing,
setting a password, linking Google/Apple where the browser can, and removing — on one
page, behind the existing message seam, with no backend change.

**Non-Goals:** linking in Safari (hand-off), Music's « sign in to link » collision flow,
changing the email, any account merge.

## Decisions

### D1 — On the account page, not in Réglages

The connected accounts are a view of the account page (`#connected`), not a block of
Réglages. Setting a password needs the code step, which must survive the mailbox; linking
opens a provider's window, which tears the popup down (Chrome destroys it on focus loss)
and would cover the in-page drawer. Both already work on the account page. Réglages ›
Données › Compte, signed in, gains a « Comptes connectés » link (`openPage`), like
« Gérer mes données »; the account page's signed-in view gets the same link.

*Alternative*: a fourth Réglages block with the list. Rejected: half the actions would
leave it anyway, and the drawer/popup would need the code step and the torn-down-popup
error recovery a second time.

### D2 — Four background messages, one session method

New account messages, dispatched by `handleAccountMessage` like the others:
`account:identities`, `account:linkProvider{provider}`,
`account:unlink{provider, subject}`, `account:setPassword{email, password, locale}`.
The code reuses `account:verifyEmail{code}` — `VerifyEmail` binds a parked set-password
the same way it verifies a sign-up.

`ListIdentities` joins the `AccountPort` (`src/account/profile.ts`, already the
UserService seam for the handle). Linking adds `Session.linkProvider(provider)`: the same
id_token minting as `signInWithProvider` (factored, not copied), then `LinkIdentity` on
the bearer transport — never `SignInOidc`, so the session and its tokens stay as they
are. A closed provider window is a cancel (`cancelled: true`), as for sign-in.

*Alternative*: link from the page itself with its own transport. Rejected: the
background is the only session owner (`add-lingua-account-parity` D2); a second token
holder would race the refresh.

### D3 — Which providers can be linked

The rule is sign-in's: `availableProviders` (client id + `launchWebAuthFlow`). Safari
reports no `launchWebAuthFlow` from the extension, so it offers no link — not a special
case. A provider is offered only if the account does not have it yet. Already-linked
identities are listed everywhere.

### D4 — The set-password code step

After `SetLocalCredential` succeeds, the page moves to a code step bound to the
connected accounts (not sign-up's: a confirmed code must not sign in, and the password
was never held). The address is remembered in `storage.session` under its own key
(`PENDING_LINK_EMAIL_KEY`) so `#connected` reopens on the code step after a reload or
the mailbox; the password is dropped as soon as the RPC returns. On a confirmed code the
key is cleared and the list reloads. Since a parked submission cannot be resent (see
Context), the step offers « Recommencer » — back to the form, the address kept — instead
of « Renvoyer le code »; the old token simply expires.

### D5 — Removal and its guard

« Retirer » asks for a confirmation inline (« Retirer Google ? » / Retirer / Annuler).
With one identity left, there is no « Retirer », and the row says that the only method
cannot be removed. A `failedPrecondition` reply is still worded as the last method
(the list may be stale). Removing the method the reader signed in with is allowed — the
server keeps the session, which belongs to the account, not to the identity.

### D6 — Copy by context

`errorCopy` gains three contexts: `link` (`alreadyExists` → « Ce compte Google/Apple est
déjà lié à un autre compte Cymbra. »), `unlink` (`failedPrecondition` → « Tu ne peux pas
retirer ta seule méthode de connexion. »), `setPassword` (`invalidArgument` → password
policy, `alreadyExists` → « Cette adresse est déjà utilisée par un compte, ou ton compte a
déjà un mot de passe. »). `verify` keeps its wording for the code. `unauthenticated` in
any of them → « Ta session a expiré. Reconnecte-toi. » The French labels follow Music's
(`app_fr.arb`: « Comptes connectés », « Définir un mot de passe », « Lier Google »,
« Retirer »).

### D7 — The signed-out hint

Signed out, the Compte block adds one note under the email form: an account created with
Google or Apple signs in here once a password is set in « Comptes connectés », from a
browser where Google or Apple is offered (or from Cymbra Music). It names no account
state — the extension cannot know which account the reader means.

## Risks / Trade-offs

- [The reader sets a password with an address other than their Google one] → allowed by
  the server (the subject is the address typed); the list shows the address, so it is
  visible. No restriction is added.
- [A provider window outlives the account page] → the reply then has no page to land on;
  the next load of `#connected` shows the list as the server has it. No persisted error is
  needed, unlike the popup's sign-in.
- [Safari readers still cannot link Google/Apple from Lingua] → they can set a password
  (the Boox case) and use Music to link a provider; the hand-off is a follow-up.
- [Unlinking the only provider a browser offers, from that browser] → the guard keeps at
  least one method; the reader may still remove Google while on Firefox for Android if a
  password exists, which is the intended way out.

## Migration Plan

Extension-only, additive: no stored data changes, no backend change. Ships with the next
Lingua release; rollback is the previous package.
