# localise-lingua-account-onboarding — the account and the onboarding read their copy from the catalogue, and the account's e-mails follow the interface language

## Why

Change 17 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, the last of the four that move the interface's copy into the
catalogue of change 13, and the one that settles decision M12. The account surface
(`account/{copy,flow,view}.ts`, `account.html`) holds ≈ 120 unique texts — the sign-in flows,
the errors in plain words, the connected accounts, « Lié le ${date} » — and the onboarding
(`onboarding.html`, `onboarding.ts`, `level-row.ts`) ≈ 20: « Bienvenue dans Cymbra Lingua »,
« Quelles langues apprends-tu ? », the level rows, « Pour commencer », the account offer.

The account page also tells the server which language to write to the reader in: today
`navigator.language`, the browser's, on sign-up. M12 settles it: the account's e-mails and the
deletion link follow the interface language for `fr`, `en` and `es`; another language Cymbra
speaks — Italian, which Music speaks and the extension does not — keeps the browser's, so a
reader with an Italian browser is not moved from Italian e-mails to English ones, and the
shared account locale Music adopts does not move either.

## What Changes

- **The account and the onboarding read their copy from `src/i18n/<language>/{account,
  onboarding}.ts`**, picked by the interface language read with their first storage read (the
  account page has no engine; the onboarding reads it before its engine); their pages are filled
  at mount and carry `lang`; « Lié le ${date} » and « 1 à ${max} lettres ou chiffres » become
  slot messages with the locale's date in English and Spanish.
- **The locale sent to the server** on sign-up and on the deletion link: the interface language
  when it is `fr`, `en` or `es` (always, today); the browser's language instead when the browser
  is in a language Cymbra speaks and the extension does not (`it` today), so that Music's
  Italian stays Italian. The account's e-mails and the deletion page therefore come in the
  interface language.
- **The French byte for byte**: the `account-copy`, `account-flow`, `account-view`,
  `account-handle` and the onboarding spec files pass unchanged; each surface gains one test in
  English.
- **The baseline** loses these files — the last ones: the lint's baseline is then empty, and
  the lint forbids a French literal anywhere outside the catalogue.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *The account and the onboarding speak the interface
  language*, *The account's e-mails follow the interface language*.

No requirement is modified; `lingua-account`'s requirements (held by `add-lingua-account-parity`
and `add-lingua-connected-accounts`, both 28/29 and 16/16, awaiting archive) quote French copy
read under the umbrella rule, and *Auth errors shown in plain words* is met in every language.

## Impact

- **Products.** Cymbra Lingua: `apps/lingua-extension` (the files above, the catalogue modules,
  the baseline). Cymbra ID is consumed, not changed: the account's `locale` field exists
  (`user.proto` `locale = 7`), the sign-up and the deletion link already take a locale; the
  server's e-mail templates speak `fr`, `en`, `es` and `it` already (Music's locales). Music, Live,
  the back office and the site are untouched.
- **No byte moves.** The spec files are the check; the locale sent today (`navigator.language`)
  is `fr-*` for every French-native reader with a French browser, and becomes `fr` — the server
  reads the primary subtag.
- **Order.** After change 13; independent of 14–16. The last of the four: the baseline empties.
- **Not here.** The native language's question at onboarding (20); the languages' names (19);
  the site's Spanish pages (29).
