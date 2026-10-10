# localise-lingua-account-onboarding — the account and the onboarding read their copy from the catalogue, and the account's e-mails follow the interface language once chosen

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
it. M12 settles it: the account's e-mails and the deletion link follow the interface language for
`fr`, `en` and `es`; another language Cymbra speaks — Italian, which Music speaks and the extension
does not — keeps the browser's, so a reader with an Italian browser is not moved from Italian
e-mails to English ones. The owner settled when (2026-10-10, in session): once the reader has
chosen their language on the device (change 20), and not before — and, minding several devices, a
device where the reader has not chosen never writes over the account's language, which every
device and Cymbra Music share.

## What Changes

- **The account and the onboarding read their copy from `src/i18n/<language>/{account,
  account-errors, onboarding}.ts`**, picked by the interface language read from `chrome.storage.local` before
  any copy renders (the account page has no engine; the onboarding reads it before its engine);
  their pages are filled at mount with change 14's `fillPage` and carry `lang`; « Lié le
  ${date} », « 1 à ${max} lettres ou chiffres » and « Saisis le code envoyé à … » are change 13's
  slot messages `linkedOn`, `handleEmpty`/`handleInvalid` and `codeSentTo`, with the locale's date
  in English and Spanish; `errorCopy` takes the language as an optional last parameter (the
  private `linkCopy` likewise), so its spec passes unchanged.
- **The locale sent to the server** on the four requests that carry one, **once the reader has
  chosen their language on this device** (both of change 20's records): the interface language
  when it is `fr`, `en` or `es`; the browser's language instead when the browser is in a language
  Cymbra speaks and the extension does not (`it` today), so that Music's Italian stays Italian. The
  account's e-mails therefore come in the language chosen. **The deletion link** then follows the
  interface language alone — it is a page the reader reads, not an e-mail (`lingua-privacy`).
- **Until they have chosen**, nothing that was sent moves, and nothing is written over: the
  browser's whole tag at sign-up (a new account) and when setting a password (which records
  nothing), as before; **no locale** on resending the code and requesting a reset, which would write
  it over the account's — so Cymbra ID keeps the language the account has, chosen on another device
  or given by Music, and writes the e-mail in it; the deletion link by the browser's tag, as before.
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
  browser locale": they now send the account locale of M12 once the reader has chosen their
  language, and before that the browser's language at sign-up and no locale on a resend or a reset
  request (D2); every scenario kept, the one that names the locale reworded.

- `lingua-privacy`: MODIFIED *Cymbra account deletion is reachable from Lingua* — the link is in
  the reader's interface language once they have chosen it, and in the browser's until then, as
  before (French for French, English otherwise until the site has a Spanish page); its three
  scenarios kept, the first two reworded to both states. `add-site-spanish-locale` (change 29)
  modifies the same requirement, and *The account's e-mails follow the interface language*, again
  to add the Spanish page, and archives after this change: its deltas are to start from these
  texts.

The other `lingua-account` requirements (held there and by `add-lingua-connected-accounts`,
16/16) quote French copy read under the umbrella rule, and *Auth errors shown in plain words* is
met in every language.

## Impact

- **Products.** Cymbra Lingua: `apps/lingua-extension` (the files above, the catalogue modules,
  the baseline). Cymbra ID is consumed, not changed, and no `.proto` moves: the account's `locale`
  field exists (`user.proto` `locale = 7`), the sign-up and the deletion link already take a
  locale, an empty one already keeps the account's (`user-locale-preference`); the server's e-mail
  templates speak `fr`, `en`, `es` and `it` already (Music's locales). Music, Live, the back office
  and the site are untouched.
- **No byte moves in the copy.** The spec files are the check.
- **Silent for a reader who has not chosen — the owner's decision (2026-10-10, task 3.3).** M12
  as first designed would have moved, while French is every reader's interface language, the
  e-mails of every reader whose browser is not in French to French, their Cymbra Music to French
  (Music applies a bare code by exact match, `AppLanguage.fromCode`, and ignored the whole tags
  Lingua sent), and their deletion link to the French page. The owner chose D2's second way
  instead, minding several devices:
  - **Until the reader has chosen on this device** — every reader while one native language ships
    — what a reader sees does not move on one device: the sign-up records the browser's tag as
    before, the resend and the reset are written in it, Music ignores it as before, the deletion
    page is the same. A resend or a reset no longer writes this browser's tag over a locale another
    device or Music gave the account: its e-mails follow the account. An account with no recorded
    locale at all (created with Google or Apple, its password set later, never opened in Music) gets
    its reset e-mails in English where they came in the browser's language.
  - **Once they have chosen** (change 34 ships the choice), the next sign-up, resend, reset or
    set-password from that device records the interface language — or `it` — and Music adopts it.
    The choice is the device's (change 20 never syncs it): another device where the reader has
    not chosen writes over nothing.
- **Order.** After change 13 and change 14 (`fillPage`); independent of 15, 16. Archived after
  `add-lingua-account-parity`, whose requirements it modifies.
- **Not here.** The native language's question at onboarding (20); the languages' names (19);
  the site's Spanish pages (29).
