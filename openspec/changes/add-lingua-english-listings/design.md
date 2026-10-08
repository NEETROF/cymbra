# Design — add-lingua-english-listings

## Context

See proposal.md (Why). The listing files: `apps/lingua-extension/STORE-LISTING.md` — constants (name,
homepage, privacy policy, support, category, listing language French, AMO id), the summary (from
`manifest.json`, 112 characters, Apple's bound), the French and English descriptions, single purpose,
permissions, remote code, data usage, test instructions (one field per store, ≤ 1,000 characters);
`apps/lingua-apple/STORE-LISTING.md` — name (30), subtitle (30), promotional text (170), keywords
(100), description (4,000), « What's New » (4,000), review notes (English, one field, 4,000
characters; today's block measures 4,042), URLs, screenshots per platform. Nothing is uploaded by
CI. The requirement *The store listings name each studied language* says "one listing per store"
and "French glosses".

## Goals / Non-Goals

**Goals:**
- A complete English listing in each store, for English speakers learning Spanish, drafted for the owner.

**Non-Goals:**
- Uploading listings (`add-lingua-asc-localisations-upload` is optional, outside the programme's counts).
- Pushing AMO's listings from the repository: AMO's English fields are pasted by hand — one more
  dashboard × locale than risk 7's « AMO listings pushed from it » assumed.
- Another audience's listing.

## Decisions

### D1 — One listing per store, a text per language

Each store keeps its one listing; the dashboards' per-language fields carry the English texts beside
the French. The files gain a « English » section per field, with its count; the listing-language line
says the listing is in each native language a shipped pair is glossed in.

### D2 — What the English texts say

What a reader of English gets: the highlighted words of the language they study, the honest
percentage, the card in English with its grammar and its gloss written by people (M5), review and
statistics, local analysis and privacy, books, the account; extended translation as M15 settles it
for es-en, with its download size; the coverage figures on the site. The French texts change only
where they say the product is French-only (the App Store description's last line, the
listing-language line, the summary's rationale with change 27's task 3.2); otherwise unchanged.

### D3 — The fields that are not per language

The Chrome Web Store's single purpose, its permission justifications and the remote-code answer
are single English fields, updated in place: they name Spanish for English speakers beside English
and Spanish for French speakers, and the remote-code answer names es-en as the direct model for an
English-native reader. The test instructions are one field per store: rewritten for what a
reviewer sees — a non-French browser presets the English interface studying Spanish — every label
quoted from `src/i18n/en/`, saying how to choose French in Réglages, within 1,000 characters.

### D4 — The App Store locales and review notes

en-US and en-GB, as M16 recommends; the owner settles M16 on this pull request. The en-GB text is
the en-US text (M10: US English). Each locale's URLs: Privacy `https://cymbra.app/en/privacy/`,
Support `https://cymbra.app/en/support/`, Marketing `https://cymbra.app/en/lingua/`. With fr-FR
primary, a German App Store user sees the French listing while the app opens in English (M13,
change 28): a consequence the owner weighs when settling M16. The review notes are one field,
in English, within 4,000 characters — today's block is 4,042, over it: "Purpose & audience" and
"Regional differences" are rewritten to say which interface language each reader gets and how a
reviewer chooses it; "Third-party material" adds the es-en sources; the translation model and its
size are updated from the catalogue. No text calls the app a beta (guideline 2.2) or says anything
of price (guideline 2.3.7).

### D5 — The dashboards' languages

Change 27's risk: the first package carrying `_locales` sets the Chrome Web Store listing's
default language to English. That package is uploaded without publishing (a dashboard upload of
the build artifact, or a release input that skips `:publish`), the French listing re-entered under
`fr`, the English filled under `en`, then submitted. addons.mozilla.org reads `__MSG_` at upload:
the owner checks its default locale and summary after submission. Both results are recorded in
`STORE-LISTING.md`.

### D6 — Screenshots

A list per platform and locale, each naming the page and the state to capture, taken by the owner
from change 34's build: the slots App Store Connect names (iPhone 6.5" 1284 × 2778 — a 6.9" capture
is refused there; iPad 13"; Mac), en-GB reusing en-US's set; the Chrome Web Store's 1280 × 800
without alpha.

## Risks / Trade-offs

- **Counts over a store's limit** → each field carries its count, checked in review; the review
  notes, over today, are cut to fit.
- **A listing that promises what is not shipped** → the texts are written for change 34's
  packages and pasted with them.
- **A dashboard re-keyed to English with the French listing lost** → D5's upload without
  publishing, and the owner's check recorded.

## Migration Plan

Pasted by the owner with change 34's release, after the site deploy that publishes its figures.
