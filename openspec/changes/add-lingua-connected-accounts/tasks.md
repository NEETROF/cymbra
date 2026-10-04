# Tasks — add-lingua-connected-accounts

Implementation starts once PR #660 (Réglages sub-tabs + the Compte block) is merged.

## 1. Session and background

- [x] 1.1 `src/account/profile.ts`: `AccountPort.identities()` over `UserService.ListIdentities` → `{ provider, subject, linkedAt }[]`, failures as categories like `profile()`; vitest with a fake client (local subject = email, empty list, unavailable)
- [x] 1.2 `Session.linkProvider(provider)`: factor the id_token minting out of `signInWithProvider` (one path for both), then `AuthService.LinkIdentity{id_token}` on the bearer transport — never `SignInOidc`; a closed window → `"cancelled"`; vitest: the tokens are unchanged after a link, a cancel makes no RPC, `alreadyExists` surfaces as a category
- [x] 1.3 `Session.unlink(provider, subject)` and `Session.setPassword(email, password, locale)` over `UnlinkIdentity` / `SetLocalCredential`; vitest: categories for `failedPrecondition`, `alreadyExists`, `invalidArgument`, `unauthenticated`, and the password never reaches a storage area
- [x] 1.4 Messages `account:identities`, `account:linkProvider`, `account:unlink`, `account:setPassword` in `src/account/messages.ts`, dispatched by `handleAccountMessage` (`src/account/host.ts`); replies carry categories only; `host.spec.ts` covers each, including signed out (`unauthenticated`, no RPC)

## 2. Account page — Comptes connectés

- [x] 2.1 `src/account/flow.ts`: a `connected` view (deep link `#connected`, signed in only — signed out falls back to sign-in) holding the identities, the providers that can be linked (D3: available × not yet linked), a removal being confirmed, and the set-password form / code step (D4); vitest for each transition
- [x] 2.2 Set a password (D4): submit → `account:setPassword` → code step, address under `PENDING_LINK_EMAIL_KEY` in `storage.session`, password dropped once the reply is in; code → `account:verifyEmail` → key cleared, list reloaded; « Recommencer » back to the form with the address kept; a reload of `#connected` resumes on the code step; vitest asserts no storage write ever holds the password
- [x] 2.3 Link (D2, D3): « Lier Google » / « Lier Apple » only for an available provider the account lacks; cancel → nothing shown; success → list reloaded; vitest with providers `{google:true, apple:false}` and `{false,false}` (Firefox for Android, Safari)
- [x] 2.4 Remove (D5): inline confirmation; no « Retirer » with a single identity and the note why; `failedPrecondition` worded as the last method; vitest
- [x] 2.5 `src/account/view.ts`: render the list (provider name, the address for email and password, « Lié le <date> » in French), the actions, the form, the code step and the errors; pure-renderer tests like the existing views; the signed-in view links to « Comptes connectés »
- [x] 2.6 Copy (D6) in `src/account/copy.ts`: contexts `link`, `unlink`, `setPassword`; `unauthenticated` → reconnect in every one; vitest table, and the existing lint (no error `message` read) still passes

## 3. Réglages › Données › Compte

- [x] 3.1 `src/reading/account-setting.ts`, signed in: a « Comptes connectés » link opening `account.html#connected` (works from the popup, the side panel and the in-page drawer); vitest
- [x] 3.2 Signed out: the D7 note under the email form; vitest

## 4. Checks

- [x] 4.1 Gates: `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (coverage ≥ 80 % overall, new modules covered), `yarn build` + `yarn check:variants`
- [ ] 4.2 [manual] Chrome on macOS, production backend: on a Google-only account, set a password (code from the mailbox), see the method listed; link and remove Google on an account that has a password; try to link a Google account that owns another Cymbra account and read the message
- [ ] 4.3 [manual] Boox Go 10.3 (Firefox for Android), production: sign in with the address and password set in 4.2 and land on the same account (same handle, same deck); the connected accounts list both methods and offer no « Lier Google »
- [ ] 4.4 [manual] Safari on macOS: the list shows, « Définir un mot de passe » and « Retirer » work, no « Lier » is offered
