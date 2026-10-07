# Design — localise-lingua-account-onboarding

## Context

See proposal.md (Why) and change 13's design. Today:

| Module | Copy / behaviour |
|---|---|
| `account/copy.ts` | `errorCopy(context, kind)`, `linkCopy(provider, kind)`: the errors in plain words per flow and provider (25 + 3 templates) |
| `account/flow.ts` | the steps' messages (« Un code de vérification a été envoyé à ${email} », « C'est noté : ton pseudo est @${handle} »…); `deps.locale` sent on four requests — `account:signUp`, `account:resendVerification`, `account:requestPasswordReset`, `account:setPassword`; `deleteAccountUrl(locale)` chooses the French or English deletion page by `locale.toLowerCase().startsWith("fr")` |
| `account/view.ts` | 82 literals + « Lié le ${date} » (`toLocaleDateString("fr-FR")`), « Retirer ${provider} de ton compte ? », « Saisis le code envoyé à » + `<b>email</b>` |
| `account/account.ts` | `locale: navigator.language || "fr"` — the whole tag; its storage reads are `chrome.storage.session` (the pending e-mail), none of `chrome.storage.local` |
| `account.html` | two text nodes |
| `onboarding.html` | 13 text nodes (title, « Quelles langues apprends-tu ? », « Pour commencer », the account offer) |
| `onboarding.ts`, `level-row.ts` | « Débutant — je pars de zéro », « Niveau enregistré : ${…}. Tu peux fermer… » |
| Cymbra ID | `UserAccount.locale` (`user.proto` field 7), stored as sent and adopted as such by Music; the e-mail templates in Music's four locales (fr, en, es, it); the site's deletion pages `/suppression-compte` (fr) and `/en/delete-account` |

## Goals / Non-Goals

**Goals:**
- Every text of the account and the onboarding comes from the catalogue.
- The account's e-mails and the deletion link come in the interface language, without moving an
  Italian reader's e-mails (M12).
- The baseline keeps only changes 18's and 19's files after this change.

**Non-Goals:**
- A Spanish deletion page on the site (change 29): until it exists, the deletion link for a
  Spanish interface goes to the English page, as it does for every non-French locale today.
- The native language's question (20).

## Decisions

### D1 — The account page reads the key first

`account.ts` reads the interface language from `chrome.storage.local` — a read of its own,
before any copy renders; its `chrome.storage.session` reads (the pending e-mail) are another
thing — and hands it to the flow and the view (it has no engine); `onboarding.ts` reads it the
same way before `hydrateEngine` (a read, not a write: the store's ownership rule is about
writes). Both pages are filled at mount with change 14's `fillPage` and carry `lang`.

### D2 — The locale sent to the server

`accountLocale(interfaceLanguage, browserLanguage)`: the browser's primary subtag when it is one
Cymbra speaks and the extension does not (`it`); otherwise the interface language — a bare
primary subtag, never the browser's whole tag. It is `deps.locale`, sent on the four requests
that carry one today. `deleteAccountUrl(interfaceLanguage)` takes the interface language, not the
account locale: French page for `fr`, English otherwise until the site's Spanish page exists
(change 29 adds `es`) — an Italian-browser reader with a French interface gets French e-mails'
neighbour, Italian, from the server, and the French deletion page from the link. A test holds
the list of languages Cymbra speaks (`fr`, `en`, `es`, `it`) beside Music's `app_<code>.arb`
files in `apps/music/lib/l10n/` (those files alone), so a fifth language added to Music is
noticed.

What changes for a reader today is in the proposal (Impact): the server keeps what is sent, so
an English-browser reader's e-mails move to French while French is every reader's interface
language. Alternatives: stop sending a locale from Lingua — the account's e-mails would stay
whatever the last client wrote, and a Lingua-only reader would get English; or send the
interface language only once the reader has chosen it (change 20's `cymbra-lingua-native-chosen`)
and the browser's whole tag until then, which keeps today's e-mails for everyone until the
choice exists. M12 chose the interface language; the owner confirms or picks the second
alternative in task 3.3.

### D3 — Slot messages and dates

« Lié le ${date} » takes a date formatted for the interface language (`fr-FR` kept for French,
change 13's `formatDate`); « Saisis le code envoyé à <b>email</b> » takes the e-mail as a slot
the view renders in bold; « 1 à ${max} lettres ou chiffres » takes the bound. `errorCopy(language,
context, kind)` and `linkCopy(language, provider, kind)` take the language first and read the
catalogue's tables; the flow holds the language and passes it.

### D4 — The baseline, after this change

The lint's baseline, down to the account's and the onboarding's files and two others after
14–16, loses the account's and the onboarding's; `reading/grammar-labels.ts` (change 18) and
`analyzer/language-labels.ts` (change 19) stay until those changes, which empty it.

## Risks / Trade-offs

- **An Italian reader's e-mails moved** → D2 keeps the browser's `it`; tested.
- **A locale the server does not speak** → the interface language is one of the four; a German
  browser sends the interface language, not `de`.
- **An English-browser reader's e-mails move to French** → the owner's acknowledgement, or the
  second alternative of D2 (task 3.3).
- **A French byte in the account flows** → the four account spec files, unchanged.

## Migration Plan

One release. The locale written on a reader's next request that carries one is the account
locale of D2 — a bare `fr`, `en`, `es` or `it` where the server held the browser's whole tag —
and their e-mails follow from then on; the change is visible to a reader whose browser is not
in French (proposal, Impact). Nothing is migrated on the server: an account that sends no new
request keeps its locale.
