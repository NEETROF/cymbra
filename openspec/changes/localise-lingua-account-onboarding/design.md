# Design — localise-lingua-account-onboarding

## Context

See proposal.md (Why) and change 13's design. Today:

| Module | Copy / behaviour |
|---|---|
| `account/copy.ts` | `errorCopy(context, kind)` and the private `linkCopy(provider, kind)`: the errors in plain words per flow and provider (25 + 3 templates) |
| `account/flow.ts` | the steps' messages (« Un code de vérification a été envoyé à ${email} », « C'est noté : ton pseudo est @${handle} »…); `deps.locale` sent on four requests — `account:signUp`, `account:resendVerification`, `account:requestPasswordReset`, `account:setPassword`; `deleteAccountUrl(locale)` chooses the French or English deletion page by `locale.toLowerCase().startsWith("fr")` |
| `account/view.ts` | 82 literals + « Lié le ${date} » (`toLocaleDateString("fr-FR")`), « Retirer ${provider} de ton compte ? », « Saisis le code envoyé à » + `<b>email</b>` |
| `account/account.ts` | `locale: navigator.language || "fr"` — the whole tag, on all four requests; its only `chrome.storage.local` read is `followSurfaceLook`'s (the colours, at module load); the pending e-mail is in `chrome.storage.session` |
| `account.html` | three text nodes (two French per the lint) |
| `onboarding.html` | 13 text nodes (title, « Quelles langues apprends-tu ? », « Pour commencer », the account offer) |
| `onboarding.ts`, `level-row.ts` | « Débutant — je pars de zéro », « Niveau enregistré : ${…}. Tu peux fermer… » |
| Cymbra ID | `UserAccount.locale` (`user.proto` field 7): a non-empty locale is recorded as sent — at sign-up, on a resend and on a reset request, last writer wins — and an empty one keeps the stored one, the e-mail then written in it (`user-locale-preference`; `backend/auth/src/module.rs` `effective_locale`, `backend/user/src/module.rs` `set_locale`); setting a password (`SetLocalCredential`) records nothing and writes in the request's locale, English when it carries none; the templates read a locale by its primary subtag (`SupportedLocale::parse`) and speak Music's four locales (fr, en, es, it); Music applies the account's locale to its own interface at each sign-in and cold start only on an exact match (`AppLanguage.fromCode`: `fr` yes, `fr-FR` no), and writes its own with `SetLocale`; the site's deletion pages `/suppression-compte` (fr) and `/en/delete-account` |

## Goals / Non-Goals

**Goals:**
- Every text of the account and the onboarding comes from the catalogue.
- The account's e-mails and the deletion link come in the interface language once the reader has
  chosen it, without moving an Italian reader's e-mails (M12); until then nothing a reader sees
  moves, and a device where they have not chosen writes over no account's language (the owner's
  decision of 2026-10-10, D2).
- The baseline is empty after this change, changes 18 and 19 having taken their files off.

**Non-Goals:**
- A Spanish deletion page on the site (change 29): until it exists, the deletion link for a
  Spanish interface goes to the English page, as it does for every non-French locale today.
- The native language's question (20).

## Decisions

### D1 — The account page reads the key first

`account.ts` reads the interface language from `chrome.storage.local` with its first read of
its own, before any copy renders (`followSurfaceLook`'s colour read and the `chrome.storage.session`
pending-e-mail reads are other things) and hands it to the flow and the view (it has no engine); `onboarding.ts` reads it the
same way before `hydrateEngine` (a read, not a write: the store's ownership rule is about
writes). Both pages are filled at mount with change 14's `fillPage` and carry `lang`.

### D2 — The locale sent to the server

**Decided by the owner on 2026-10-10 (in session): the second way — the interface language only
once the reader has chosen it — minding several devices.** Two facts shape it:

- **The account's locale is shared, the choice is not.** Cymbra ID keeps one locale per account:
  sign-up, a resend and a reset request record a non-empty one over it, last writer wins, and keep it
  when they carry none, writing the e-mail in it (`user-locale-preference`: *Empty locale does not
  overwrite*, *Stored locale used when request carries none*); setting a password records nothing
  and writes in the request's locale, English when it carries none; Cymbra Music adopts the
  account's locale at each sign-in and cold start when it is one of its codes, whole. The reader's
  choice of native language is a device's: change 20's marker (`cymbra-lingua-native-chosen`,
  chrome.storage.local) and its record (`cymbra-lingua-last-native`, the background's store) are
  never synced, nor is the profile. So a request from one device moves the e-mails, and Music, on
  every device.
- **A choice is both of change 20's records** (`chosenLanguage`): the device marked the choice as
  made, and the store's owner recorded the language the reader last chose — the one the page shows.
  Neither alone is one: an update marks every installed extension as chosen without asking it
  (M22), and a new install's preset records its language before the reader answers. An answer that
  keeps the language the extension already had records nothing; where nothing was recorded before
  (a French browser confirming French), the page keeps to the rule below for a reader who has not
  chosen — French e-mails either way. No new marker.

Hence `accountLanguage(interfaceLanguage, browserLanguage, chosen)`:

- **Until the reader has chosen on this device** — every reader while one native language ships —
  the page writes over no account's language. Sign-up carries the browser's whole tag as it always
  did (`navigator.language || "fr"`): it records it on an account that has none yet. Setting a
  password carries it too: it records nothing. A resend and a reset request carry **no locale**
  (`keepAccountLocale`), so Cymbra ID keeps the account's own — a language the reader chose on
  another device, or Music's — and writes the e-mail in it. The deletion page by the browser's tag,
  as before.
- **Once they have**, `accountLocale(interfaceLanguage, browserLanguage)` on all four requests: the
  browser's primary subtag when it is one Cymbra speaks and the extension does not (`it`); otherwise
  the interface language — a bare primary subtag, never the browser's whole tag. The deletion page
  by the interface language alone (`deleteAccountUrl`, the rule it always had: French for a tag that
  starts with `fr`, English otherwise until the site's Spanish page exists — change 29 adds `es`) —
  an Italian-browser reader with a French interface gets Italian e-mails from the server and the
  French deletion page.

A reader who chose English on laptop A, then asks for a password reset on laptop B — a French
browser, nothing chosen there — keeps English e-mails, and Cymbra Music keeps English: B sends no
locale. Music's language moves only with a request from a device where the reader chose, to the
language chosen there; a reader who chose differently on two devices moves it with each one's
sign-up, resend or reset, as they move it with Music's own setting. A test holds the list of
languages Cymbra speaks (`fr`, `en`, `es`, `it`) beside Music's `app_<code>.arb` files in
`apps/music/lib/l10n/` (those files alone), so a fifth language added to Music is noticed; another
holds the four requests against a Cymbra ID that answers as the server does, on two devices.

For a reader who never chose, on one device, nothing moves: the sign-up records the browser's tag as
before, the resend and the reset that follow are written in it, Music ignored that whole tag and
still does, the deletion page is the same. What differs from before is what the warning asks: a
resend or a reset no longer writes this browser's tag over a locale another device or Music gave the
account — its e-mails follow the account.

**Alternatives not taken.** M12 as first designed, the interface language at once for every
reader: while French is every reader's interface language, a reader whose browser is not in French
would have got French e-mails from their next sign-up, resend, reset or set-password, their Cymbra
Music switched to French at its next start (it ignored the whole tags Lingua sent and applies a
bare `fr`), and the French deletion page. The second way as first written, the browser's whole tag
on all four requests until the reader chose: a device where nothing is chosen would write its
browser's tag over a language chosen elsewhere at each resend or reset. Sending no locale from
Lingua at all: a Lingua-only reader's sign-up would bring English e-mails.

### D3 — Slot messages and dates

« Lié le ${date} », « Saisis le code envoyé à <b>email</b> » and « 1 à ${max} lettres ou
chiffres » are change 13's slot messages `linkedOn(date)`, `codeSentTo(email)` and
`handleEmpty(max)`/`handleInvalid(max)`; the date is formatted for the interface language
(`fr-FR` kept for French, change 13's `formatDate`), the e-mail rendered in bold by the view.
`errorCopy(context, kind, language = "fr")` takes the language as an optional last parameter and
reads the catalogue's `account-errors` module; the private `linkCopy` likewise; the flow holds the
language and passes it — so `account-copy.spec.ts` passes unchanged.

The errors are a module of their own, `src/i18n/{fr,en,es}/account-errors.ts` — the keys
`errorCopy` and `linkCopy` read, with `providerGoogle` and `providerApple`, moved out of `account`
with their values verbatim, typed after the French — because `reading/account-setting.ts` imports
`errorCopy`: with one `account` module, the content script, the popup, the side panel and the reader
carried the account page's whole copy in three languages. `accountCopy(language)` returns
`{ ...errors, ...account }`, one object for the page's `fillPage`, the flow and the view.

### D4 — The baseline, after this change

The lint's baseline, down to the account's and the onboarding's files after 14–16, 18
(`reading/grammar-labels.ts`) and 19 (`analyzer/language-labels.ts`), loses them and is empty.

## Risks / Trade-offs

- **An Italian reader's e-mails moved** → once chosen, D2 keeps the browser's `it`; before, the page
  writes over nothing; tested.
- **A locale the server does not speak** → once chosen, the interface language is one of the four;
  a German browser sends the interface language, not `de`.
- **Several devices** → the choice is a device's and is not read elsewhere; a device where the reader
  did not choose sends no locale on a resend or a reset, so it never writes over a language chosen on
  another device or given by Music (D2); tested with two devices.
- **An account with no recorded locale** → on a device where nothing is chosen, its reset e-mail is
  in English where it came in the browser's language. Narrow: Lingua and Music record one at sign-up,
  and Music at sign-in when the account has none (`account-language-sync`); an account created with
  Google or Apple that set its password later and never opened Music has none.
- **Setting a password where nothing is chosen** → the e-mail in this browser's language, as before:
  Cymbra ID records nothing for it, so no choice is written over, but it writes that one e-mail in the
  request's locale, not the account's. Changing that is Cymbra ID's, not this change's.
- **A new install that never answered, then updated** → its preset's record and the update's mark are
  both there: its preset's language — the browser's own, or English — counts as chosen.
- **A storage read that hangs** → `interfaceLanguage` waits 500 ms at most, then shows French with a
  warning; the page's 1.5 s reveal is only for a script that dies before the fill.
- **A French byte in the account flows** → the five account spec files, unchanged.

## Migration Plan

One release. A device where the reader has not chosen sends what it sent, except on a resend and a
reset request, which carry no locale: the account keeps the one it has. A device where the reader
chose — none until change 34 ships the choice — sends the account locale of D2, a bare `fr`, `en`,
`es` or `it`, on its next request that carries one. Nothing is migrated on the server: an account
keeps its locale until a request records another.
