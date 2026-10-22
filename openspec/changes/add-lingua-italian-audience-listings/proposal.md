# add-lingua-italian-audience-listings — the store listings for Italian speakers

## Why

Change 46 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. Change 45 ships en-it with the Italian interface. Its packages carry `_locales/it` (change
39), so the Chrome Web Store and addons.mozilla.org show an Italian summary to Italian browsers. Its
Safari app declares Italian (change 40), so the App Store names Italian among the app's languages.
Without Italian listings, an Italian visitor reads an Italian summary over a French, English or Spanish
description. On the App Store's Italian storefront, they read the French listing.

I9, settled on 2026-10-22, adds it-IT to the App Store's locales and ships the Italian listing with the
first build that declares Italian, as the English and Spanish listings shipped with the first localised
builds (M13; matrix changes 36 and 37).

## What Changes

- **The Italian texts, in the listing files** (design D1), written for Italian speakers studying
  English:
  - the Chrome Web Store and AMO description, about 480 words;
  - the App Store it-IT name, subtitle, promotional text, keywords, description and What's New, about
    500 words more;
  - each claim tied to the file that makes it true at change 45, in the claims table.
- **The shared English fields** (D2): the single purpose, the permission justifications, the remote-code
  answer, the test instructions and the review notes name the Italian path. The test instructions (12
  characters left, 5 if a line break counts two) and the review notes (6 left) keep every existing
  label, with measured cuts. These fields are English, so the owner reviews them.
- **The summary is quoted, not drafted** (D3): it is `_locales/it`'s, which the version check already
  holds to what Italian readers study.
- **Screenshots** (D4): the Italian sets listed per store. Change 45's capture harness makes the
  extension's; the App Store's are taken by hand from change 45's build, as the precedent's were.
- **The dashboards' procedure for Italian** (D5), with its results table, as for Spanish.
- **The Italian reviewed by the Italian reviewer agent** (D6, I2) in this change's pull request; the
  English fields by the owner.
- **Before change 45's submission** (D7): merged before change 45's packages are submitted, since the
  owner pastes from main. Nothing is submitted here.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by
  `add-lingua-italian-english-listings`):
  - the listings carry Italian texts for Italian speakers once a pair glossed in Italian ships;
  - an Italian text is in wording the Italian reviewer agent has reviewed, as I2 settles;
  - a scenario *A listing in Italian* is added.

## Impact

- **Products.** Cymbra Lingua's listing files only: `apps/lingua-extension/STORE-LISTING.md` and
  `apps/lingua-apple/STORE-LISTING.md`. No code, site, app or backend change.
- **Commit type**: `docs(lingua)`. release-please assigns a component by path, so a `feat` or `fix`
  touching `apps/lingua-*` would cut a release.
- **The owner**: the support link, the screenshots of the App Store set, and pasting the listings with
  change 45's submission (I12).
- **Prerequisites**:
  - changes 2 (en-it's download size), 33 (the Italian interface the texts quote), 34 (the card's tense
    names), 36 (en-it's glosses), 39 (`_locales/it`), 40 (the Safari app's Italian), 41 (`/it/lingua/`)
    and 42 (en-it's coverage on it) implemented;
  - change 38's decision on en-it's translation, both variants drafted until then.
- **Order**: on main before change 45's submission, pasted after the site deploy that follows change 45;
  written against change 45's branch.
- **Effort, against 2–5.5 ideal days**: 2.25–4 (design, *Effort*).
