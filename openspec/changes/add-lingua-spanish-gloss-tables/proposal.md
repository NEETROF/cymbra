# add-lingua-spanish-gloss-tables — the Spanish pack's French glosses

## Why

The es-fr pack has forms, frequencies and grammar (`add-lingua-spanish-forms-tables`,
`add-lingua-spanish-grammar-tables`), but no gloss: a Spanish card would show no translation.
Decision D4 of the programme (`docs/lingua/spanish-programme.md`) sets the rules:
- the gaps in the data are accepted, and their numbers are published;
- a curated list adds the missing verbal locutions;
- a gloss is never English, and never machine-pivoted.

The study named the sources: the French Wiktionary's Spanish entries, with fallbacks from the
Spanish Wiktionary's translations and from the French tables read backwards. It counted
≈23,700 glossed lemmas, and ≈85/73/58 % of the top 5k/10k/20k without fallback.

Measured on the 2026-10-03 snapshot:

| Lemmas | French Wiktionary | with the translations |
|---|---|---|
| top 5,000 | 82.9 % | 87.7 % |
| top 10,000 | 69.6 % | 77.3 % |
| top 20,000 | 54.4 % | 63.8 % |

That is 22,826 glossed lemmas, 17,420 of them from the French Wiktionary. There are 15,133
expressions: 2,952 from the French Wiktionary, the rest from the translations. The pack grows to
2,126,574 B, 40.6 % of the budget.

kaikki has announced it will retire its per-language files. The French Wiktionary's French
section, which holds the translation tables, is a 3.2 GB per-language file. Its dump of the whole
edition is 703 MB, and also holds the Spanish entries.

This is change 22, in G1, the internal Spanish build. No package ships the es-fr pack yet.

## What Changes

- **Sources from whole Wiktionary dumps.** `pack_sources.py` reads kaikki's dumps of the French
  and Spanish Wiktionaries. It derives three files from them:
  - the French Wiktionary's Spanish entries, as written;
  - its French entries' translation tables, cut down;
  - the Spanish Wiktionary's French translations.

  Each file is kept as an asset of the snapshot's release and pinned by sha256; a dump is never
  kept whole. The update workflow publishes every asset.
- **`reduce-es-fr.py` writes the glosses.**
  - `gloss.tsv`, `senses.tsv` and `mwe.tsv` come from the French Wiktionary's Spanish entries,
    through the rules every pair shares, at en-fr's limits.
  - Then, for what those leave out: the French words the Spanish Wiktionary lists, then the French
    entries whose tables list the word, the commonest first.
  - At most three French words per part of speech, as one sense. A proper noun's translation
    glosses nothing.
- **Curated locutions.** A `LOCUTIONS` list wins over every source. Its glosses are written by a
  person, so it ships empty (design D5).
- **`NOTICE`** credits the French and Spanish Wiktionaries. The README and `SOURCES.md` publish the
  coverage.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The Spanish pack's French glosses* and *Sources derived from whole
  Wiktionary dumps are pinned*.

## Impact

- **Products.** Cymbra Lingua's data pipeline:
  - `scripts/lingua-data/` (the sources tool, the reducer, the tables, the README,
    `SOURCES.md`);
  - `.github/workflows/lingua-pack-update.yml` (every asset published);
  - `crates/lingua-pack` (a test).

  No core change. No package carries the es-fr pack until `enable-lingua-spanish`. The
  extension, the server, the protos, ID, Music, Live, the back office and the site are not
  affected.
- **Sources and licences**: the French and Spanish Wiktionaries through kaikki, CC BY-SA 4.0 +
  GFDL. That is the licence family already accepted, and no new licence enters the pack.
- **The owner**: the snapshot's release now holds four assets instead of one. The release is
  published by the owner, as for change 20.
- **Release.** G1, internal: nothing ships.
