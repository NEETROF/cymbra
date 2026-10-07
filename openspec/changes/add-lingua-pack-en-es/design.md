# Design — add-lingua-pack-en-es

## Context

See proposal.md (Why), and change 21's design (D1 the reducer by import, D2 the sources, D3 the
pin and `pack_version`, D4 the credits, D5 the owner's settings, D6 not shipped). What en-es
needs on top:

| What | Where |
|---|---|
| The Spanish edition's rules | `reduce_edition_es.py` (`ES`: form-of pointers after « de/del », notes and sense-link subscripts, letters; `capitalised` true; `long_parenthesis` 0), tested on recorded senses; `AGlossIsWrittenInTheReadersLanguage` already derives `("translations", "en", "es")` from a made-up English entry and glosses "house" as « Casa, vivienda » |
| The Spanish Wiktionary's English section | es-fr's `kaikki-es` source: the whole-edition dump `eswiktionary/raw-wiktextract-data.jsonl.gz` (98 MB), from which `kaikki-es-traductions.jsonl` (es→fr) is derived; its English entries were censused and never kept; there is no per-language eswiktionary URL |
| The English Wiktionary's translation tables | the English entries' tables (68,579 entries with sense-level Spanish translations); no step fetches the English Wiktionary's English extract (`kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl`, ≈ 2 GB; the raw dump is larger); `translations_of` reads entry- and sense-level tables; `derive` makes one gzip pass |
| `DUMPS` | per pair; a derived file is a release asset (`lingua-pack-sources-<pair>-<snapshot>`), published by `update`; `fetch_pinned` downloads assets, never a dump |
| The reduce job | 45 minutes, ≈ 1.5 GB per pair; en-fr 10 s, es-fr 31 s |
| The English studied side | `tables/en/` (reference en-fr); `reduce-en-fr.py` holds it and binds `french.FR` |
| Coverage | `gloss_coverage.py` top 5k/10k/20k of `tables/en/freq.tsv` in the pair's `gloss.tsv`; published for `packs.json`'s pairs only |
| M6 | fr-es ships with its coverage published, a floor fixed before the committed measurement |

## Goals / Non-Goals

**Goals:**
- A committed, pinned, reproducible en-es, whose pinned reduction fetches small derived files.
- Glosses written by people, in Spanish: the Spanish Wiktionary's definitions first, the English
  Wiktionary's translation-table words second (M5).
- A floor the pair must reach to ship, fixed before the measurement.

**Non-Goals:**
- Shipping (35), the card wording (24), the model (25), the marks (26).
- Change 38's move to raw dumps for every pair: this change fetches one more extract the way the
  others are fetched, and leaves the migration to 38.

## Decisions

### D1 — `reduce-en-es.py`: en-fr's studied side by import, the Spanish edition bound

As change 21's D1: `reduce-en-fr.py` loaded by `importlib` for the English studied side (its
output left in `work`), `EDITION = spanish.ES`, `native_tables(entries, direct)`: entries = the
Spanish Wiktionary's English section, direct = the English Wiktionary's Spanish translations,
ordered as the table lists them; no inverted table. `max_lemmas` 40,000, as en-fr.

### D2 — Two derived files, one new dump source

`DUMPS["en-es"]`:
- `kaikki-es-English.jsonl`, `("entries", "en")` from es-fr's `kaikki-es` dump (the same URL, the
  same snapshot when both pairs are updated together, derived in the same pass as es-fr's and
  es-en's files);
- `kaikki-en-traductions-es.jsonl`, `("translations", "en", "es")` from a new dump source,
  `kaikki-en`: the English Wiktionary's English extract (`kaikki.org/dictionary/English/kaikki.
  org-dictionary-English.jsonl`), fetched by `fetch_live` only, derived in one pass, never kept.
Both are release assets of `lingua-pack-sources-en-es-<snapshot>`; a pinned reduction fetches
them and en-fr's assets, not the extract. `pin.json` records the extract's URL, size, sha256 and
`last_modified` as the other dumps' are recorded. If kaikki stops serving the per-language
extract, the raw dump is the source — change 38's work, which this pin makes a one-line change.

### D3 — A floor, fixed now

The spec fixes the floor at the study's figures less two points: 91.4 / 83.2 / 69.9 %. A
committed measurement under it fails `gloss_coverage.py --pair en-es --floor`, which the reduce
job runs; change 35 publishes the measured figures with the pair. The share of glosses that are
translation-table words (rather than the Spanish Wiktionary's definitions) is measured and shown
beside, as risk 5 asks.

### D4 — The owner's review

A sample of 100 glosses from the top 10,000 — the definitions and the translation-table words
marked as such — in the pull request (M9); the Spanish edition's settings stay as change 6 set
them unless the sample says otherwise.

## Risks / Trade-offs

- **The English extract is ≈ 2 GB** → fetched by `fetch_live` at an update only; the reduce job
  fetches the derived files (≈ 10 MB).
- **A thin pair that reads as a lesser product** (risk 5) → the floor (D3), the share of
  translation-table glosses shown, and change 35's published figures.
- **kaikki's per-language extracts removed** (risk 6) → the pin names the URL; change 38 moves
  every pair to the raw dumps.
- **A gloss in a third language** → the direct table is en→es only.

## Migration Plan

No release: tables and tooling only. The first `lingua-pack-update` dispatch for en-es publishes
its derived files' release.
