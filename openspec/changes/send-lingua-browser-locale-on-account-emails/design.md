# Design — send-lingua-browser-locale-on-account-emails

## Context

See proposal.md (Why). What the account page sends today (change 17 on main,
`apps/lingua-extension/src/account/`):

| Request (message → RPC) | Not chosen on this device | Chosen |
|---|---|---|
| Sign-up (`account:signUp` → `SignUpLocal`) | the browser's whole tag (`en-GB`; `fr` when it gives none) | the account locale: the interface language, bare (`fr`, `en`, `es`), or the browser's `it` |
| Resend the code (`account:resendVerification` → `ResendVerification`) | **no locale** (`""`) | the account locale |
| Request a reset (`account:requestPasswordReset` → `RequestPasswordReset`) | **no locale** (`""`) | the account locale |
| Set a password (`account:setPassword` → `SetLocalCredential`) | the browser's whole tag | the account locale |
| The deletion link | by the browser's tag | by the interface language |

`chosenLanguage(preferences, store)` reads change 20's two records (the device's mark and the
store's last native language); `accountLanguage(interfaceLanguage, browserLanguage, chosen)` returns
`{ locale, keepAccountLocale, deletion }`; `page.ts` hands them to `AccountFlow`, whose private
`overwritingLocale()` sends `""` on the resend and the reset while `keepAccountLocale` is true — that
is, until the reader has chosen. The extension sends a locale on these four messages only
(`state/session.ts`: `signUpLocal`, `resendVerification`, `requestPasswordReset`,
`setLocalCredential`) and calls `SetLocale` nowhere.

What Cymbra ID does with them, before and after 17b:

| | Before 17b — what change 17 was designed against | 17b (#859, on main; its deploy, task 3.6, not yet ticked at 917877cd) |
|---|---|---|
| The e-mail's language | sign-up, set-password: the request's, else English; resend, reset: the request's, else the stored one, else English (`effective_locale`) | all four: the stored one, else the request's, else English (`email_locale`) |
| What a request records | sign-up: its locale, on the new account; resend, reset: a non-empty locale **over** the stored one, last writer wins; set-password: nothing | all four, through `AuthModule::account_email_locale`: a non-empty request locale only when nothing non-empty is stored |
| What replaces a stored locale | a resend, a reset, `SetLocale` | `SetLocale` alone |

Cymbra Music adopts the account's locale after sign-in only when it is one of its codes, whole
(`fr` yes, `fr-FR` no); pushes its own when the account has none; and leaves one it cannot display
alone, without overwriting it (`account-language-sync`).

## Goals / Non-Goals

**Goals:**
- An account with no stored language gets its resend and reset e-mails in the browser's language,
  not English, from a device where the reader has not chosen.
- No device, chosen or not, writes over a language the account has — held by the server since 17b,
  for every client.
- One locale per state, the same on the four requests; less code.

**Non-Goals:**
- The chosen state (change 17's D2, decision M12): unchanged.
- The deletion link: unchanged.
- Lingua writing the account's language when the reader chooses (`SetLocale`) — 17b's Q3, settled:
  a Lingua choice does not move the account's language.
- Normalising stored tags (`fr-FR` → `fr`), or anything on the server.

## Decisions

### D1 — One locale per state, on all four requests

After this change:

| Request | Not chosen on this device | Chosen |
|---|---|---|
| Sign-up | the browser's whole tag (unchanged) | the account locale (unchanged) |
| Resend the code | the browser's whole tag (was no locale) | the account locale (unchanged) |
| Request a reset | the browser's whole tag (was no locale) | the account locale (unchanged) |
| Set a password | the browser's whole tag (unchanged) | the account locale (unchanged) |
| The deletion link | by the browser's tag (unchanged) | by the interface language (unchanged) |

In code: `AccountLanguage` loses `keepAccountLocale` and is `{ locale, deletion }`;
`accountLanguage` returns `{ locale: tag, deletion: tag }` until the reader has chosen
(`tag = browserLanguage || "fr"`), and what it returns today once they have.
`AccountFlowDeps.keepAccountLocale` and `AccountFlow.overwritingLocale()` go; `resend` and
`requestReset` send `this.deps.locale`, as `signUp` and `setPassword` do; `page.ts` passes no flag.
The comments of `locale.ts`, `flow.ts` and `page.ts` say the rule with 17b's server. The flag is
removed rather than left always false: its only purpose was the old server's last-writer-wins, and a
seam kept for it would invite a use the server no longer needs.

What the reader gets, by the account's state, on 17b's server:

| The account | Before this change (17 + 17b) | After |
|---|---|---|
| Has a language L, from any device or app | every e-mail in L; nothing recorded | the same |
| Has none; nothing chosen on this device | resend, reset: English, nothing recorded; sign-up, set-password: the browser's language, recorded | all four: the browser's language (English when Cymbra does not write it), the first one recorded |
| Has none; chosen on this device | all four: the account locale, recorded | the same |

So the change shows on one kind of account only. A resend serves an unverified local credential,
whose sign-up recorded a locale (Lingua's and Music's sign-ups always carry one), so it practically
always finds one. A reset serves any account with a password; those with no language are 17b's case
1 — created with Google or Apple, a password set before 17b's deploy (17b's set-password records one
from then on), never opened in Cymbra Music (which pushes its own at sign-in when there is none).

### D2 — With 17b's server, no device can overwrite a stored language

1. **What the extension sends.** A locale on four requests only — `SignUpLocal`,
   `ResendVerification`, `RequestPasswordReset`, `SetLocalCredential` — and never `SetLocale`.
2. **What those four do on 17b's server.** Each handler in `backend/auth/src/module.rs` —
   `sign_up_local`, `resend_verification`, `request_password_reset`, `set_local_credential` — takes
   its e-mail's language from one helper, `AuthModule::account_email_locale(user_id, request)`, and
   that helper is the auth module's only call to `UserPort::set_locale` outside its tests: it reads
   `UserPort::locale`, and writes the request's locale only when nothing non-empty is stored and the
   request's is non-empty. Resend and reset reach it only inside their account-exists branch (17b's
   D5).
3. **The only other writer.** `SetLocale` (`backend/user/src/grpc.rs`), authenticated, for the
   caller's own account — Cymbra Music's and the back office's language settings. The extension
   never calls it.
4. **Hence.** An account with a non-empty stored language L keeps L under any sequence of these four
   requests — from any number of devices, chosen or not, carrying any locale — and every one of its
   e-mails is written in L (`email_locale`: the stored one first). A request's locale is read only
   for an account that has none: the first one fills it, and every later request finds it stored.
   The one write left over a value is 17b's D2 race — two first requests on an empty account at the
   same instant may both write, and the last stays — and both are locales that account's requests
   carried.

The empty locale of change 17 therefore protects nothing on this server, and the locale a resend or
a reset carries is only the fallback for an account without one. The protection holds for every
client: Lingua versions before change 17, which sent the browser's tag on all four, and Cymbra Music.

*Example.* Laptop A, a French browser where the reader chose English: the sign-up records `en`.
Laptop B, a French browser where nothing is chosen, asks for a reset carrying `fr-FR`: the server
finds `en`, records nothing and writes the e-mail in English; Cymbra Music stays in English.

### D3 — The browser's whole tag, not its primary subtag

Until the reader chooses, the resend and the reset carry what the sign-up and the set-password
already carry, the browser's whole tag, not a bare `fr`:

- **One rule** for the four requests in each state.
- **Cymbra Music.** A bare `fr` recorded on an account with none is one of Music's codes: Music would
  adopt it at the next sign-in, its interface moved by a request from a device where the reader chose
  nothing — what M12's first design was set aside for. `fr-FR` it cannot display, so it keeps its own
  interface and leaves the account's alone (`account-language-sync`).
- **The e-mail** reads the primary subtag anyway (`SupportedLocale::parse`): `fr-FR` is French.

The cost: an account a reset fills with a whole tag no longer gets Music's own code pushed at
sign-in — as a Lingua sign-up or set-password already leaves it (17b's D3). And a browser tag Cymbra
does not write (`de-DE`) fills the account with a language its e-mails are not written in: Q1.

### D4 — Released after 17b's backend deploy, never before

On the server before 17b — what production runs until the owner deploys #859 — a resend or a reset
carrying a non-empty locale records it over the stored one (`set_locale`, last writer wins) and is
written in it (`effective_locale`, the request's first). Released before that deploy, this change
undoes what change 17's D2 was for:

*Example.* The reader chose English on laptop A; the account's language is `en`. On laptop B — a
French browser, nothing chosen — they ask for a password reset, which now carries `fr-FR`. The old
server records `fr-FR` over `en`: that e-mail and every later one come in French, and so do Cymbra
Music's practice reminders, which read the stored locale. Music's screen stays English (`fr-FR` is
not one of its codes) but no longer pushes English back (it leaves a locale it cannot display alone),
so the account stays `fr-FR` until the reader changes Music's language. Any reader who chose on one
device could be moved by another.

Hence **task 1.1 [manual]**: the implementation's pull request merges only once the owner has
deployed 17b's backend (`backend-deploy`) and run its check from outside (17b's task 3.6). The
merge is gated, not only this release: a Lingua release cut for an unrelated fix after the merge —
the extension's or the Apple host app's, which carries it — would carry this change too. Once 17b is
deployed, the order holds by itself.

*Not taken.* An automated gate in `lingua-extension-release`: the server exposes nothing that says
17b is there (`/healthz` and `/readyz` answer `ok`), a probe would need a test account and its
mailbox, and a check that the last successful `backend-deploy` ran a commit holding #859 would serve
this one release.

*Rollback.* If 17b's backend were rolled back while this version is in the stores, the hazard
returns until 17b is redeployed; the remedy is the redeploy, not a Lingua release (a store review
takes days).

### D5 — Tests

In `test/account-locale.spec.ts`:

- **The fake Cymbra ID answers as 17b's server does**: each e-mail in the stored language, else the
  request's, else English, by the primary subtag; a request's non-empty locale recorded, on all four,
  only when nothing is stored; a `setLocale(tag)` for Music's language setting, the one write over a
  stored locale; Music adopting a stored locale only when it is one of its codes, whole. Today's fake,
  which answers as the server before 17b, is kept under another name for one test (e).
- **What a page sends** (`sentLocales`), not chosen: `en-GB` on all four, `fr-FR` on all four,
  `it-IT` on all four; a browser that gives no language: `fr` on all four; the deletion link as
  today. Chosen: unchanged.
- **Two devices** against the 17b fake:
  - (a) English chosen on A (its sign-up records `en`); B, a French browser where nothing is chosen,
    asks for a reset and sets a password, both carrying `fr-FR`: the account keeps `en`, every e-mail
    is English (the set-password's too, since 17b), Music shows English.
  - (b) An account Music gave `es`; B, an American browser where nothing is chosen, on the code step,
    resends and resets, both carrying `en-US`: the account keeps `es`, both e-mails are Spanish, Music
    shows Spanish.
  - (c) An account with no language (17b's case 1); B, a French browser where nothing is chosen, asks
    for a reset carrying `fr-FR`: the e-mail is French (English with change 17 as it is), the account
    records `fr-FR`, Music's interface does not move; then A, where English is chosen, asks for a
    reset carrying `en`: the e-mail is French and the account keeps `fr-FR`.
  - (d) A choice does not move an account's language — change 17's test *the device where the reader
    chose writes their choice — and only there*, inverted as 17b's server answers: the account is
    `en`, B chose French, its reset carries `fr`: the account keeps `en`, the e-mail is English.
  - (e) The release order's reason: (a) against the server before 17b — B's reset moves the account
    to `fr-FR` and its e-mail to French. It pins why task 1.1 holds the merge.
- **The `lingua-account` test**: the resend and the reset request carry the sign-up's locale in both
  states.

In `test/account-page.spec.ts`, *an English interface in an Italian browser, nothing chosen on this
device*: the resend carries `it-IT` where it carried `""`.

Unchanged: `account-flow.spec.ts` (it never set the flag, so its resend and reset already carry
`fr-FR`), `account-host.spec.ts`, `session.net.spec.ts`, `account-copy`, `account-view`,
`account-handle`, `onboarding`, `onboarding-level-row` and `level-choice`; the lint's baseline stays
empty; every golden and snapshot.

### D6 — The spec deltas and the archive order

*The account's e-mails follow the interface language* (`lingua-interface-language`) is ADDED by
change 17 and MODIFIED by `add-site-spanish-locale` (change 29), which archives after 17 and whose
delta is still to start from 17's text (17's proposal). When two open changes modify one requirement,
the one archived last replaces it whole. This change lists both in `archiveAfter`, so it archives
last, and its block carries both: change 17's text with this change's edits, and change 29's Spanish
deletion page by reference — *A Spanish-native reader* says the deletion link is the one
`lingua-privacy` gives a Spanish interface, the English page before change 29 and the Spanish page
after. Nothing in this change moves the link. Change 17's six scenarios keep their names; two are
added.

*Email verification by code* and *Password reset by code* (`lingua-account`) are ADDED by
`add-lingua-account-parity` and MODIFIED by change 17: only the parenthesis that names the locale
changes, and every scenario is verbatim. Both changes are in `archiveAfter`.

Not modified: *Account creation by email from the extension* (it already sends the browser's
language until the reader chooses), `lingua-privacy`, and 17b's `user-locale-preference` and
`transactional-email`. `prefer-account-locale-for-emails` is in `archiveAfter` because this change's
text states its rule — a request's locale recorded only on an account that has none — which the
specs hold only once 17b is folded.

## Risks / Trade-offs

- **Released before 17b's deploy** → a device where nothing is chosen writes over the account's
  language again (D4); task 1.1 gates the merge.
- **17b rolled back after the release** → the same until it is redeployed (D4).
- **A browser tag Cymbra does not write fills an empty account** (`de-DE`) → its e-mails English, as
  before, and a device where the reader later chooses cannot fill it → Q1.
- **Music no longer pushes its own code onto an account a reset filled with a whole tag** → as a
  Lingua sign-up or set-password already leaves it (D3).
- **A browser that gives no language** → `fr`, a bare code Music adopts, as the sign-up and the
  set-password already send; every browser the extension runs in gives one.
- **Two first requests on an empty account at once** → 17b's D2 race, harmless.
- **A French byte moved** → no copy, catalogue or page is touched; the account and onboarding spec
  files pass unchanged.

## Migration Plan

Nothing is migrated. One Lingua release, after 17b's backend deploy (D4). An account with a language
sees nothing change. An account without one is filled by its next resend or reset from a device
where nothing is chosen, with that browser's tag. Rolling this change back brings change 17's empty
locale back, harmless on either server.

## Open Questions

The design follows each recommendation; the owner settles both before the implementation merges
(task 1.2).

- **Q1 — A browser in a language Cymbra's e-mails are not written in.** *Example:* Hans's account
  has no language (created with Google, his password set before 17b's deploy, never opened in
  Music). He asks for a password reset from Lingua on his German browser, where he chose nothing:
  the request carries `de-DE`, the e-mail comes in English — as with change 17 — and the account now
  keeps `de-DE`. Later he chooses French in Lingua on his laptop and asks for another reset: the
  e-mail is still English, because a stored language wins even when the e-mails are not written in
  it (17b's D1). Had Lingua sent nothing for a browser language Cymbra does not write, the account
  would have stayed empty and the laptop's `fr` would have filled it. *Recommendation:* send the tag
  anyway — a German-browser sign-up and set-password already record `de-DE` the same way, one rule
  serves the four requests, the case needs both an account with no language and a browser Cymbra
  does not write, and Music's language setting replaces the stored tag.
- **Q2 — The whole tag or a bare code before the choice.** *Example:* Léa's account has no language;
  she asks for a reset from Lingua on her French browser, where she chose nothing. With the whole tag
  `fr-FR` (D3), the e-mail is French and the account keeps `fr-FR`; Cymbra Music, which she uses in
  English, stays in English at her next sign-in. With a bare `fr`, the e-mail is French too, and
  Music switches to French at her next sign-in. *Recommendation:* the whole tag — moving Music's
  screen from a device where she chose nothing is what M12 set aside.
