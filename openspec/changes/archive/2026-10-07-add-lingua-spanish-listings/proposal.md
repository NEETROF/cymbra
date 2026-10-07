# add-lingua-spanish-listings — the store listings name Spanish

## Why

`enable-lingua-spanish` (change 28) put es-fr in every package beside en-fr, and its manifest
summary already names both languages. Nothing else in the stores does yet. The listing copy kept in
the repository still describes an extension that teaches English to French speakers:
- the Chrome Web Store and addons.mozilla.org listing, `apps/lingua-extension/STORE-LISTING.md`;
- the App Store listing of the Safari app, `apps/lingua-apple/STORE-LISTING.md`.

The programme decided one listing per store, reworded, with the final wording the owner's
(decision D10). Its translation decision (D3) adds that the listings state each language's
translation like for like.

This is change 29, R4 in `docs/lingua/spanish-programme.md`.

## What Changes

- **One listing per store names English and Spanish**, English first. There is no second listing
  per language.
- **The extension listing** (Chrome Web Store, addons.mozilla.org):
  - the description, in French and English, says what Spanish adds: the card's tense and gender,
    the choice of languages in the settings, and the line « Plusieurs langues à la fois : gratuit
    pour l'instant. »;
  - the single purpose, the remote-code answer (one pack per language) and the reviewer's
    test instructions follow. The instructions stay within 1,000 characters, and their labels are
    quoted from the source.
- **The App Store listing**:
  - subtitle, promotional text, keywords and description name Spanish, within Apple's limits;
  - a « What's New » paragraph is ready for the release that ships it;
  - it never calls the app a beta and says nothing of price, as App Store review guideline 2.2
    keeps betas and trials off the store.
- **Translation, like for like:**
  - English: extended translation is offered, with its download size;
  - Spanish: the listings say it comes later. Changes 26 and 27 add its size when it ships.
- **Coverage, honestly:** the listings say that the French dictionary is a little less complete
  for Spanish, and point to the figures the site publishes (`add-site-lingua-spanish-pages`,
  change 30).
- **The wording is the owner's.** The design carries a full draft. The listing files take the
  wording the owner settles, and the owner pastes it into each dashboard with the release that
  ships Spanish.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: a requirement is added. The store listings name each studied
  language, say per language whether extended translation is offered, and point to the published
  coverage. The App Store listing never calls the app a beta.

## Impact

- **Products:**
  - Cymbra Lingua: its three store listings change;
  - the site's figures come from change 30;
  - nothing is consumed from ID or the platform.
- **Files:** `apps/lingua-extension/STORE-LISTING.md` and `apps/lingua-apple/STORE-LISTING.md`.
  No code, no package and no permission changes: the Spanish pack ships inside the package.
- **Release:**
  - nothing is submitted by this change;
  - the listings are pasted by hand, as today;
  - the « What's New » paragraph waits for the release that ships Spanish, which is the owner's.
