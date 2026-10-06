# add-lingua-spanish-forms-tables — the Spanish pack's forms and frequencies

## Why

The Spanish analyser (`add-lingua-spanish-analysis`, `add-lingua-spanish-detection-guard`) has rules
but no lexicon: there is no es-fr pack. Its forms table decides how every Spanish token is
lemmatised. It also decides how words are counted, known and reviewed, and which reading of a
homograph (`fue`: *ser* or *ir*) a reader meets.

The programme fixed the sources and the gates (`docs/lingua/spanish-programme.md`):
- **Forms and grammar**: kaikki's extract of the English Wiktionary, Spanish section, which tags
  every form.
- **Frequencies**: wordfreq `es`, already pinned.
- **Homographs**: UD Spanish-GSD as the tie-breaker.
- **Measurement**: UD Spanish-PUD, kept for tests.
- **Gates on PUD**: 98.5 % of tokens resolved, 93.5 % of content lemmas, 97 % of auxiliaries.

A prototype run on 2026-10-03 (S1) settles the size question the programme left open. With the
combined clitic forms left out, which the enclitic rule resolves, the 60,000 commonest lemmas and
their forms attested in wordfreq make a forms table of 144,805 rows, 2.6 MB. That is far under the
≈10 MB that would have called for a snapshot outside git, so the tables are committed like
en-fr's. With the real analyser, the pack passes the gates on PUD: 99.38 % of tokens resolved,
95.92 % of content lemmas and 97.95 % of auxiliaries.

This is change 20, in G1, the internal Spanish build. No package ships the es-fr pack yet; that is
`enable-lingua-spanish`.

## What Changes

- **A reducer for es-fr** (`scripts/lingua-data/reduce-es-fr.py`), on the shared rules of
  `reduce_common.py`:
  - **forms** from kaikki's tagged inflection tables and form-of senses, without the combined clitic
    forms and without multi-word or non-Spanish strings. A string that is both a combined form and
    a plain form keeps the plain one (`principales`: *principal*);
  - **one lemma per form**: a reviewed override first, then GSD's counts of that form under each
    lemma, then the form's own entry, then the lemma's frequency;
  - **the 60,000 commonest lemmas** by wordfreq (inflected and combined forms skipped), and their
    forms attested in wordfreq.
- **Committed tables** under `scripts/lingua-data/tables/es-fr/`: `forms.tsv`, `freq.tsv`, an empty
  `gloss.tsv` (the French glosses are `add-lingua-spanish-gloss-tables`), `NOTICE`, `manifest.json`
  and `pin.json`. The override list and its reasons are part of the reducer, so editing them is a
  rule change.
- **A measurement harness** (`scripts/lingua-data/measure/es-pud.sh` and a `lingua-pack` binary). It
  runs the real analyser with the built es-fr pack over UD Spanish-PUD, fetched at a pinned commit
  and never committed, and fails under the gates.
- **The pipeline knows es-fr**:
  - `build.sh` reduces it;
  - `lingua-pack-update` offers it;
  - the extension check builds its pack from the committed tables, as it builds en-fr's.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The Spanish pack's forms and frequencies* and *Spanish forms are
  measured on a held-out treebank*.

## Impact

- **Products.** Cymbra Lingua's data pipeline:
  - `scripts/lingua-data/` (the reducer, the tables, the harness, `SOURCES.md`);
  - `crates/lingua-pack` (the measurement binary);
  - `.github/workflows/lingua-pack-update.yml` (the pair option).

  The es-fr pack is built and checked in CI. No extension package carries it: `packs.json` still
  lists en-fr alone. The extension, the server, the protos, ID, Music, Live, the back office and
  the site are not affected.
- **Sources and licences**:
  - kaikki / English Wiktionary: CC BY-SA 4.0 + GFDL;
  - wordfreq: CC BY-SA 4.0;
  - UD Spanish-GSD: CC BY-SA 4.0, counts only;
  - UD Spanish-PUD: CC BY-SA 3.0, measured against and never shipped.

  All are credited in `NOTICE`.
- **Release.** G1, internal: nothing ships.
