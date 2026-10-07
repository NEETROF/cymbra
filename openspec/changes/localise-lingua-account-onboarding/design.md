# Design — localise-lingua-account-onboarding

## Context

See proposal.md (Why) and change 13's design. Today:

| Module | Copy / behaviour |
|---|---|
| `account/copy.ts` | `errorCopy(context, kind)`, `linkCopy(provider, kind)`: the errors in plain words per flow and provider (25 + 3 templates) |
| `account/flow.ts` | the steps' messages (« Un code de vérification a été envoyé à ${email} », « C'est noté : ton pseudo est @${handle} »…); `locale` sent on sign-up (`signUp(…, locale)`); `deleteAccountUrl(locale)` chooses the French or English deletion page by `locale.toLowerCase().startsWith("fr")` |
| `account/view.ts` | 82 literals + « Lié le ${date} » (`toLocaleDateString("fr-FR")`), « Retirer ${provider} de ton compte ? », « Saisis le code envoyé à » + `<b>email</b>` |
| `account/account.ts` | `locale: navigator.language || "fr"` |
| `account.html` | two text nodes |
| `onboarding.html` | 13 text nodes (title, « Quelles langues apprends-tu ? », « Pour commencer », the account offer) |
| `onboarding.ts`, `level-row.ts` | « Débutant — je pars de zéro », « Niveau enregistré : ${…}. Tu peux fermer… » |
| Cymbra ID | `UserAccount.locale` (`user.proto` field 7), the e-mail templates in Music's four locales (fr, en, es, it); the site's deletion pages `/suppression-compte` (fr) and `/en/delete-account` |

## Goals / Non-Goals

**Goals:**
- Every text of the account and the onboarding comes from the catalogue.
- The account's e-mails and the deletion link come in the interface language, without moving an
  Italian reader's e-mails (M12).
- The baseline is empty after this change.

**Non-Goals:**
- A Spanish deletion page on the site (change 29): until it exists, the deletion link for a
  Spanish interface goes to the English page, as it does for every non-French locale today.
- The native language's question (20).

## Decisions

### D1 — The account page reads the key first

`account.ts` reads the interface language with its first storage read (it has no engine) and
hands it to the flow and the view; `onboarding.ts` reads it before `hydrateEngine`. Both pages
are filled at mount and carry `lang`.

### D2 — The locale sent to the server

`accountLocale(interfaceLanguage, browserLanguage)`: the browser's primary subtag when it is one
Cymbra speaks and the extension does not (`it`); otherwise the interface language. Sent on
sign-up as today's `locale`, and used by `deleteAccountUrl`, which keeps its rule (French page
for `fr`, English otherwise) until the site's Spanish page exists (change 29 adds `es`). A test
holds the list of languages Cymbra speaks (`fr`, `en`, `es`, `it`) beside Music's locales, so a
fifth language added to Music is noticed.

Alternative: stop sending a locale from Lingua. The account's e-mails would stay whatever the
last client wrote; a Lingua-only reader would get English. M12 chose the interface language.

### D3 — Slot messages and dates

« Lié le ${date} » takes a date formatted for the interface language (`fr-FR` kept for French);
« Saisis le code envoyé à <b>email</b> » takes the e-mail as a slot the view renders in bold;
« 1 à ${max} lettres ou chiffres » takes the bound. `errorCopy` and `linkCopy` keep their
signatures and read the catalogue's tables.

### D4 — The baseline empties

The lint's baseline, down to these files after 14–16, loses them; the lint then names no file and
forbids a French literal anywhere outside `src/i18n/`.

## Risks / Trade-offs

- **An Italian reader's e-mails moved** → D2 keeps the browser's `it`; tested.
- **A locale the server does not speak** → the interface language is one of the four.
- **A French byte in the account flows** → the four account spec files, unchanged.

## Migration Plan

One release, silent. The locale written on the next sign-up is the interface language's primary
subtag, which is what the server kept from `navigator.language` for a French browser.
