# add-lingua-spanish-audience-listings — the store listings in Spanish, for Spanish speakers learning English

## Why

Change 37 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, with M13 and M16. The listings are copy kept in the repository and pasted by the owner
into each dashboard: `apps/lingua-extension/STORE-LISTING.md` (Chrome Web Store, addons.mozilla.org)
and `apps/lingua-apple/STORE-LISTING.md` (the App Store's Safari app). After change 36, each
listing carries French and English texts; the Spanish ones do not exist yet. Change 35 ships en-es
for Spanish speakers learning English. M13 binds the English listing to the first `_locales`; this
change applies the same rule to `_locales/es` (change 35). The extension's summary in Spanish comes
from `_locales/es` (change 27).

## What Changes

- **The extension's listing in Spanish**: the detailed description for Spanish speakers learning
  English, beside the French and English ones; the summary quoted from
  `_locales/es/messages.json`.
- **The fields that are one per store** (change 36): the test instructions, within 1,000
  characters, gain the Spanish path — how a reviewer reaches the Spanish interface studying
  English, its labels quoted from `src/i18n/es/`; the single purpose, the permission justifications
  and the remote-code answer, single English fields, are updated in place, the remote-code answer
  naming en-es as the direct model for a Spanish-native reader.
- **The App Store listing in es-ES and es-MX**, M16 as settled on change 36 (fr-FR stays primary):
  name, subtitle, promotional text, keywords, description, « What's New », each within Apple's
  limits; never a beta (guideline 2.2), nothing about price (guideline 2.3.7); the es-MX text is
  the es-ES text (M10: neutral Spanish). Each locale's Privacy policy URL
  `https://cymbra.app/es/privacidad/` (change 29) and Marketing URL `https://cymbra.app/es/lingua/`
  (change 30; built once en-es is listed and the site deployed); the Support URL
  `https://cymbra.app/es/soporte/` — change 29 says it is Music's page, so the owner chooses
  between it and `/en/support/`.
- **The App Store review notes**, one English field within 4,000 characters (change 36), gain the
  Spanish path: which interface a Spanish-speaking reviewer gets and how to choose it, the en-es
  sources in "Third-party material", and en-es's model and its size.
- **The dashboards' languages**: change 36's step applied to `es` — the first package carrying
  `_locales/es` is uploaded to the Chrome Web Store without publishing, the Spanish listing filled
  under `es`, then submitted; the owner checks addons.mozilla.org's Spanish summary after
  submission; both results recorded in `STORE-LISTING.md`.
- **Per language, like for like**: each text says whether extended translation is offered for the
  languages it names (M15's settlement) and what it downloads (the catalogue's sizes), and points
  to the coverage the site publishes in the text's language (change 30).
- **Screenshots**: the list of captures per locale and platform, taken by the owner from a build of
  change 35's branch with the interface in Spanish.
- **The wording is the owner's** (M9): the files carry full drafts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by
  `add-lingua-english-listings`, so this change is archived after it): its texts gain Spanish;
  every scenario kept, *A listing in English* included, one added.

## Impact

- **Products.** The two listing files; no code.
- **Order.** After changes 29, 30 and 36 and M15's settlement for en-es; pasted after the site
  deploy that follows change 35.
- **The owner**: reviews the drafts, chooses the Support URL, captures the screenshots and pastes
  everything into the dashboards with the release (M18).
