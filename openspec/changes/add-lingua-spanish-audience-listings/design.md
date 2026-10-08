# Design — add-lingua-spanish-audience-listings

## Context

See proposal.md (Why). The listing files after change 36: `apps/lingua-extension/STORE-LISTING.md` —
constants (name, homepage, privacy policy, support, category, the listing-language line, AMO id),
the summary (from `_locales`, 112 characters, Apple's bound), the French and English descriptions,
the single English fields (single purpose, permissions, remote code), data usage, the
test instructions (one field per store, ≤ 1,000 characters), the dashboards' languages;
`apps/lingua-apple/STORE-LISTING.md` — name (30), subtitle (30), promotional text (170), keywords
(100), description (4,000), « What's New » (4,000), review notes (English, one field, 4,000
characters), URLs, screenshots per platform, in fr-FR, en-US and en-GB. Nothing is uploaded by
CI. The requirement *The store listings name each studied language* says "one listing per store",
"a text in each native language" and "glosses in the text's language" (change 36).

## Goals / Non-Goals

**Goals:**
- A complete Spanish listing in each store, for Spanish speakers learning English, drafted for the owner.

**Non-Goals:**
- Uploading listings (`add-lingua-asc-localisations-upload` is optional, outside the programme's counts).
- Pushing AMO's listings from the repository: AMO's Spanish fields are pasted by hand, as change
  36's English ones are.
- Another audience's listing.

## Decisions

### D1 — One listing per store, a text per language

Each store keeps its one listing; the dashboards' per-language fields carry the Spanish texts beside
the French and the English. The files gain a « Spanish » section per field, with its count; the
listing-language line (change 36) names Spanish once en-es ships.

### D2 — What the Spanish texts say

What a reader of Spanish gets: the highlighted words of the language they study, the honest
percentage, the card in Spanish with its grammar and its gloss written by people (M5), review and
statistics, local analysis and privacy, books, the account; extended translation as M15 settles it
for en-es, with its download size; the coverage figures on the site. The French and English texts
are unchanged; the fields that are one per store gain the Spanish path (D3).

### D3 — The fields that are one per store

The test instructions (≤ 1,000 characters) gain the Spanish path: how a reviewer reaches the
Spanish interface studying English, its labels quoted from `src/i18n/es/`. The single purpose, the
permission justifications and the remote-code answer, single English fields, are updated in place,
the remote-code answer naming en-es as the direct model for a Spanish-native reader. The App
Store review notes (one English field, ≤ 4,000 characters) gain the Spanish path: which interface
a Spanish-speaking reviewer gets and how to choose it, the en-es sources in "Third-party
material", en-es's model and its size.

### D4 — The App Store locales

es-ES and es-MX, M16 as settled on change 36. The es-MX text is the es-ES text (M10: neutral
Spanish). Each locale's URLs: Privacy `https://cymbra.app/es/privacidad/` (change 29), Marketing
`https://cymbra.app/es/lingua/` (change 30; built once en-es is listed and the site deployed),
Support `https://cymbra.app/es/soporte/` — change 29 says it is Music's page, so the owner chooses
between it and `/en/support/`. No text calls the app a beta (guideline 2.2) or says anything of
price (guideline 2.3.7).

### D5 — The dashboards' languages

Change 36's step, applied to `es`: the first package carrying `_locales/es` is uploaded to the
Chrome Web Store without publishing (a dashboard upload of the build artifact, or a release input
that skips `:publish`), the Spanish listing filled under `es`, then submitted. addons.mozilla.org
reads `__MSG_` at upload: the owner checks its Spanish summary after submission. Both results are
recorded in `STORE-LISTING.md`.

### D6 — Screenshots

A list per platform and locale, each naming the page and the state to capture, taken by the owner
from change 35's build: the slots App Store Connect names (iPhone 6.5" 1284 × 2778 — a 6.9" capture
is refused there; iPad 13"; Mac), es-MX reusing es-ES's set; the Chrome Web Store's 1280 × 800
without alpha.

## Risks / Trade-offs

- **Counts over a store's limit** → each field carries its count, checked in review; the single
  fields grown by the Spanish path stay within 1,000 and 4,000 characters.
- **A listing that promises what is not shipped** → the texts are written for change 35's
  packages and pasted with them.
- **A Support URL that serves no Lingua reader** → the owner's choice (D4).

## Migration Plan

Pasted by the owner with change 35's release, after the site deploy that follows it.
