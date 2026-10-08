# add-lingua-english-listings — the store listings in English, for English speakers learning Spanish

## Why

Change 36 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 2, with M13 and M16. The listings are copy kept in the repository and pasted by the owner
into each dashboard: `apps/lingua-extension/STORE-LISTING.md` (Chrome Web Store, addons.mozilla.org)
and `apps/lingua-apple/STORE-LISTING.md` (the App Store's Safari app). Both are French, for French
speakers — « the listing language: French — the interface is French » — with an English description
of the extension as a second text. Change 34 ships es-en for English speakers learning Spanish; M13 requires
the English listing to ship in the same submission as the first `_locales` that carries English
(change 27). The extension's summary in English comes from `_locales/en`; the rest of each listing
does not exist yet.

## What Changes

- **The extension's listing in English**: the detailed description for English speakers learning
  Spanish, beside the French one; the summary quoted from `_locales/en/messages.json`; the one
  test-instructions field per store rewritten for what a reviewer sees — a non-French browser
  presets the English interface studying Spanish — its labels quoted from `src/i18n/en/`, saying
  how to choose French in Réglages, within 1,000 characters.
- **The single purpose, the permission justifications and the remote-code answer** are single
  English fields, not one per language: updated in place, they name Spanish for English speakers
  beside English and Spanish for French speakers, and the remote-code answer names es-en as the
  direct model for an English-native reader.
- **The App Store listing in en-US and en-GB**, as M16 recommends (open; fr-FR stays primary):
  name, subtitle, promotional text, keywords, description, « What's New », each within Apple's
  limits; never a beta (guideline 2.2), nothing about price (guideline 2.3.7); the en-GB text is
  the en-US text (M10: US English). Each locale's Privacy policy URL
  `https://cymbra.app/en/privacy/`, Support URL `https://cymbra.app/en/support/` and Marketing URL
  `https://cymbra.app/en/lingua/`.
- **The App Store review notes**: one field, in English, limited to 4,000 characters — today's
  block measures 4,042, over it. Rewritten to fit: "Purpose & audience" and "Regional
  differences" say who each interface language serves and how a reviewer chooses it; "Third-party
  material" adds the es-en sources (the English Wiktionary's Spanish section, the Spanish
  Wiktionary's translations); the translation model named and its size updated from the
  catalogue.
- **The dashboards' languages** (change 27's risk): the first package carrying `_locales` sets the
  Chrome Web Store listing's default language to English; that package is uploaded without
  publishing (a dashboard upload of the build artifact, or a release input that skips `:publish`),
  the French listing re-entered under `fr`, the English filled under `en`, then submitted.
  addons.mozilla.org reads `__MSG_` at upload: the owner checks its default locale and summary
  after submission. Both results recorded in `STORE-LISTING.md`.
- **Per language, like for like**: each text says whether extended translation is offered for the
  languages it names (M15's settlement) and what it downloads (the catalogue's sizes), and points
  to the coverage the site publishes in the text's language (change 30).
- **The French texts** change only where they say the product is French-only (D2).
- **Screenshots**: the list of captures per locale and platform, taken by the owner from a build of
  change 34's branch with the interface in English.
- **The wording is the owner's** (M9): the files carry full drafts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by no
  open change): one listing per store, a text in each native language a shipped pair is glossed
  in, naming what it teaches a reader of that language; every scenario kept, one added.

## Impact

- **Products.** The two listing files; no code.
- **Order.** After changes 27 (the summary it quotes) and 30 (the page it points to), and M15's
  settlement for es-en; before change 34's release; pasted after the site deploy that publishes
  34's figures.
- **The owner**: settles M16's locales, reviews the drafts, captures the screenshots and pastes
  everything into the dashboards with the release (M18). With fr-FR primary (M16), a German App
  Store user sees the French listing while the app opens in English (M13).
