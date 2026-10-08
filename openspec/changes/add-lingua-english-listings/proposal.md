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

- **The extension's listing in English**: the detailed description for English speakers learning Spanish, the single
  purpose, the permissions and remote-code answers, and the reviewer's test instructions
  (≤ 1,000 characters, labels quoted from the English interface), beside the French ones; the summary
  quoted from `_locales/en/messages.json`.
- **The App Store listing in en-US and en-GB** (M16 (open) recommends en-US and en-GB beside fr-FR; fr-FR stays primary): name, subtitle,
  promotional text, keywords, description, « What's New », each within Apple's limits; never a beta,
  nothing about price (guideline 2.2); the review notes, in English, saying what the reviewer sees in
  each language.
- **Per language, like for like**: each language a listing names says whether extended translation
  is offered for it (M15's settlement) and what it downloads (the catalogue's sizes), and points to the
  coverage the site publishes (change 30).
- **Screenshots**: the list of captures per locale and platform, taken by the owner from a build of
  change 34's branch with the interface in English.
- **The wording is the owner's** (M9): the files carry full drafts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by no
  open change): one listing per store, its texts in each native language a shipped pair is glossed
  in; every scenario kept, one added.

## Impact

- **Products.** The two listing files; no code.
- **Order.** Before change 34's release, which submits them with the packages.
- **The owner**: settles M16's locales, reviews the drafts, captures the screenshots and pastes
  everything into the dashboards with the release (M18).
