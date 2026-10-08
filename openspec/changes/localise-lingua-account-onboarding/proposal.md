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
`navigator.language`, the browser's whole tag (`en-GB`, `fr-FR`), on four requests — sign-up,
resend the code, request a password reset, set a password — and it chooses the deletion page by
it. M12 settles it: the account's e-mails and the
deletion link follow the interface language for `fr`, `en` and `es`; another language Cymbra
speaks — Italian, which Music speaks and the extension does not — keeps the browser's, so a
reader with an Italian browser is not moved from Italian e-mails to English ones. The shared
account locale Music adopts does move for others: see Impact.

## What Changes

- **The account and the onboarding read their copy from `src/i18n/<language>/{account,
  account-errors, onboarding}.ts`**, picked by the interface language read from `chrome.storage.local` before
  any copy renders (the account page has no engine; the onboarding reads it before its engine);
  their pages are filled at mount with change 14's `fillPage` and carry `lang`; « Lié le
  ${date} », « 1 à ${max} lettres ou chiffres » and « Saisis le code envoyé à … » are change 13's
  slot messages `linkedOn`, `handleEmpty`/`handleInvalid` and `codeSentTo`, with the locale's date
  in English and Spanish; `errorCopy` takes the language as an optional last parameter (the
  private `linkCopy` likewise), so its spec passes unchanged.
- **The locale sent to the server** on the four requests that carry one: the interface language
  when it is `fr`, `en` or `es`; the browser's language instead when the browser is in a language
  Cymbra speaks and the extension does not (`it` today), so that Music's Italian stays Italian.
  The account's e-mails therefore come in the interface language. **The deletion link** follows
  the interface language alone — it is a page the reader reads, not an e-mail (`lingua-privacy`).
- **The account's errors are a catalogue module of their own**, `account-errors`, which the
  reading surfaces' account setting reads without carrying the account page's whole copy into the
  content script, the popup, the side panel and the reader.
- **The French byte for byte**: the `account-copy`, `account-flow`, `account-view`,
  `account-handle`, `account-host`, `onboarding-level-row` and `level-choice` spec files pass
  with their French assertions unchanged; the onboarding page has no spec and gains one; each
  surface gains one test in English.
- **The baseline** loses these files and is empty: changes 18 and 19 took theirs
  (`reading/grammar-labels.ts`, `analyzer/language-labels.ts`) off first.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *The account and the onboarding speak the interface
  language*, *The account's e-mails follow the interface language*.

- `lingua-account` (held by `add-lingua-account-parity`, 28/29, awaiting archive — this change
  archives after it): MODIFIED *Account creation by email from the extension*, *Email verification
  by code* and *Password reset by code*, whose text sends "the browser's UI language" / "the
  browser locale": they now send the account locale of M12; every scenario kept, the one that
  names the locale reworded.

- `lingua-privacy`: MODIFIED *Cymbra account deletion is reachable from Lingua* — the link is in
  the reader's interface language, no longer the browser's (French for French, English otherwise
  until the site has a Spanish page); its three scenarios kept, the first two reworded to the
  interface language. `add-site-spanish-locale` (change 29) modifies the same requirement again to
  add the Spanish page, and archives after this change.

The other `lingua-account` requirements (held there and by `add-lingua-connected-accounts`,
16/16) quote French copy read under the umbrella rule, and *Auth errors shown in plain words* is
met in every language.

## Impact

- **Products.** Cymbra Lingua: `apps/lingua-extension` (the files above, the catalogue modules,
  the baseline). Cymbra ID is consumed, not changed: the account's `locale` field exists
  (`user.proto` `locale = 7`), the sign-up and the deletion link already take a locale; the
  server's e-mail templates speak `fr`, `en`, `es` and `it` already (Music's locales). Music, Live,
  the back office and the site are untouched.
- **No byte moves in the copy.** The spec files are the check.
- **Not silent for every reader — the owner's acknowledgement.** Until change 20 ships the
  choice, every reader's interface language is French. Three effects follow, each visible:
  - **E-mails.** Cymbra ID stores the locale as sent and writes in `fr`, `en`, `es` or `it`. A
    reader whose browser is not in French (and not in Italian), who gets English or Spanish
    e-mails today, gets French ones from their next sign-up, resend, reset or set-password.
    The templates read a locale by its primary subtag (`SupportedLocale::parse`), so for them a
    French browser's `fr-FR` and the new `fr` are the same.
  - **Cymbra Music's interface.** Music copies the account locale onto its own interface at each
    sign-in and cold start, by exact match on the code (`app_locale.dart`,
    `AppLanguage.fromCode`): a whole tag such as `en-GB`, `fr-FR` or `it-IT` — what Lingua sent
    until now — matched nothing and was ignored, while a bare `fr` or `it` is applied. So a reader
    whose Music is in English or Spanish gets Music switched to French on its next start after a
    sign-up, resend, reset or set-password from Lingua; a reader with an Italian browser gets
    Music in Italian.
  - **The deletion link.** It follows the interface language, so every reader whose browser is
    not in French — who gets the English page today — gets the French page.

  This is what M12 asks for, and it is a visible change for those readers, not a silent one: the
  owner acknowledges it before this change merges (task 3.3), or picks D2's second alternative —
  send the interface language only once the reader has chosen it (change 20's marker), and the
  browser's whole tag and today's deletion rule until then — which this change can carry instead.
- **Order.** After change 13 and change 14 (`fillPage`); independent of 15, 16. Archived after
  `add-lingua-account-parity`, whose requirements it modifies.
- **Not here.** The native language's question at onboarding (20); the languages' names (19);
  the site's Spanish pages (29).
