# localise-cymbra-id-email-legal-links — a Spanish e-mail links the Spanish legal pages

## Why

Change 32 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, with M12. Cymbra ID sends two e-mails — the verification code and the password-reset
code — in English, Spanish, French or Italian (`SupportedLocale`). Their footer links the terms and
the privacy policy: French recipients get `/cgu/` and `/confidentialite/`, every other recipient the
English pages (`legal_links`, `backend/platform/src/email_template/mod.rs`; the requirement
*Localized footer legal links*). Change 29 publishes the Spanish pages; a Spanish e-mail should link
them. The layout also declares `<html lang="en">` whatever the e-mail's language, so a screen reader
reads a French e-mail with an English voice.

## What Changes

- **`legal_links`**: French → `/cgu/`, `/confidentialite/`; Spanish → `/es/terminos/`,
  `/es/privacidad/`; English and Italian → `/en/terms/`, `/en/privacy/` (no Italian site).
- **`<html lang>`** is the e-mail's locale.
- **The tests**: `non_french_uses_english_legal_links` becomes one test per locale; the sample
  renderer (`emit_samples`) covers Spanish.
- **The rule stated for the e-mails alone**: the requirement's "consistent with the `legal-links`
  resolution used by the apps" is dropped; Music's in-app links keep their own rule (a Music change
  may follow).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `transactional-email` (Cymbra ID; a legacy name, not renamed here): MODIFIED *Localized footer
  legal links* — Spanish gets the Spanish pages; both scenarios kept, one added; ADDED *An e-mail
  declares its language*. Neither is held by an open change.

## Impact

- **Products.** Cymbra ID: `backend/platform/src/email_template/` (`legal_links`, the layout) and
  its tests, the auth module's test that asserts `/cgu/`. Cymbra Music's Spanish users get Spanish
  links in these e-mails while the app keeps linking `/en/privacy/`.
- **Order.** After change 29, merged and **deployed**: the owner deploys the site before the
  backend that links its Spanish pages (M18). A sign-up e-mail queued before the backend deploy keeps
  the links it was rendered with.
- **Not here.** A deletion or support link in the e-mails; Italian pages.
