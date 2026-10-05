# add-site-lingua-spanish-pages — the site's Lingua page names Spanish

## Why

Cymbra Lingua ships Spanish beside English since `enable-lingua-spanish` (change 28). The site's
Lingua page, `cymbra.app/lingua` in French and `cymbra.app/en/lingua` in English, still presents an
extension for English alone. Its last card reads « 🧭 Anglais → français — La première paire de
langues. D'autres suivront. »

The programme also promised readers the Spanish dictionary's gaps in figures, side by side with
English (decision D4: « thinner glosses ship with their numbers published »). The store listings
point to the site for them (`add-lingua-spanish-listings`, change 29).

This is change 30, R4 in `docs/lingua/spanish-programme.md`.

## What Changes

- **Both pages name English and Spanish.** The pages are `apps/site/src/pages/lingua.astro` and
  `en/lingua.astro`. These elements change:
  - the meta description and the tagline;
  - the word card's description: in Spanish, it names the tense and the gender;
  - the level card: Spanish levels are estimated from word frequency, for want of a free CEFR list;
  - the language card: the choice of languages, and « Plusieurs langues à la fois : gratuit pour
    l'instant. ».
- **A coverage section, figures side by side.** For each language, the share of the 5,000, 10,000
  and 20,000 commonest words that have a French gloss in the shipped dictionary:
  - the same measure for both languages;
  - computed from the committed tables;
  - stating that a gloss is never a machine translation;
  - stating which languages extended translation serves: English today, Spanish later.
- **The figures follow the tables.**
  - `scripts/lingua-data/coverage.py` measures every pair and writes
    `apps/site/src/data/lingua-coverage.json`, which the pages read;
  - a check fails when the committed figures no longer match the tables, so the monthly table
    update cannot leave them stale.
- **Publication stays the owner's.** The site deploys by hand (`site-deploy`, a dispatch), with the
  release that ships Spanish.

## Capabilities

### New Capabilities

- `site-lingua-page`: what the site's Lingua page says about the languages Lingua teaches and the
  coverage of their dictionaries.

### Modified Capabilities

None.

## Impact

- **Products:**
  - the site `cymbra.app` (apps/site): the Lingua pages change, in French and English;
  - Cymbra Lingua: its tables are read, nothing in it changes;
  - nothing is consumed from ID or the platform.
- **Code:**
  - `apps/site/src/pages/lingua.astro`, `en/lingua.astro`;
  - `apps/site/src/data/lingua-coverage.json`, new;
  - `scripts/lingua-data/coverage.py` and its test.
- **CI:** the coverage check runs where the tables are already checked (the lingua-data unit tests
  in `lingua-extension-check`). `site-check` builds the pages.
- **Release:** nothing deploys on merge. The owner dispatches `site-deploy` with the release.
