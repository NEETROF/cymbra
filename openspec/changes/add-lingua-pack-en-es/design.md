# Design — add-lingua-pack-en-es

## Context

See proposal.md (Why), and change 21's design (D1 the native side alone, D2 shared sources and
one release per pair, D3 the pin's studied record, D4 the credits, D5 the owner's settings, D6
not shipped and the floor). What en-es needs on top:

| What | Where |
|---|---|
| The Spanish edition's rules | `reduce_edition_es.py` (`ES`: form-of pointers after « de/del », notes and sense-link subscripts, letters; `capitalised` true; `long_parenthesis` 0), tested on recorded senses; `AGlossIsWrittenInTheReadersLanguage` derives `("translations", "en", "es")` from a made-up English entry and glosses "house" as « Casa, vivienda » |
| The Spanish Wiktionary's English section | es-fr's `kaikki-es` source: the whole-edition dump `eswiktionary/raw-wiktextract-data.jsonl.gz`, from which `kaikki-es-traductions.jsonl` (es→fr) is derived, and change 21 derives `kaikki-es-traductions-en.jsonl` (es→en); its English entries were censused and never kept; no per-language eswiktionary address exists |
| The English Wiktionary's translation tables | the English entries' tables (68,579 entries with sense-level Spanish translations); no step fetches the English Wiktionary's English extract (`kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl`, served uncompressed; the repository records no size for it, nor for the raw dump); `translations_of` reads entry- and sense-level tables; `derive` opens a dump with `gzip.open` |
| `DUMPS` | per pair; a derived file is a release asset; `fetch_live` downloads each dump as `<name>.dump.jsonl.gz`; the dump record holds release, url, fetched, last_modified and files; `fetch_pinned`/`fetch_live` require `KAIKKI[pair]` today |
| `native_tables` | entries, then direct and inverted tables, locutions winning; es-fr reads both directions |
| The English studied side | `tables/en/` (reference en-fr); its sources are en-fr's: the French Wiktionary's form links, ESDB (SCOWL, with its WordNet notice), CEFR-J and Octanove, wordfreq |
| Coverage | `gloss_coverage.py` top 5k/10k/20k of `tables/en/freq.tsv` in the pair's `gloss.tsv`; `--pair` and `--floor` from change 21; the reducer prints `native_tables`' primary count to stderr |
| Risks | 5: a thin pair reads as a lesser product (en-es, fr-es), mitigated by published like-for-like coverage and a floor; 6: kaikki removes its per-language files, mitigated by "en-es deriving from the raw English dump", a snapshot, and change 38 |

## Goals / Non-Goals

**Goals:**
- A committed, pinned, reproducible en-es, whose pinned reduction fetches small derived files.
- Glosses written by people, in Spanish: the Spanish Wiktionary's definitions first, the
  translation tables' words second, in both directions (M5).
- A floor the owner sets, before the measurement is committed.

**Non-Goals:**
- Shipping (35), the card wording (24), the model (25), the marks (26).
- Change 38's move to raw dumps for every pair: this change reads one extract and leaves the
  move to 38 with a one-line change of address.

## Decisions

### D1 — `reduce-en-es.py`: the native side alone, three sources, the Spanish edition

As change 21's D1: `tables/en/forms.tsv` and `freq.tsv` read as committed, `EDITION =
spanish.ES`, `common.native_tables(entries, ranks, studied=EN, edition=EDITION, fallbacks=[(direct,
list), (inverted, by_spanish_frequency(…))])`: entries = the Spanish Wiktionary's
English section; direct = the English Wiktionary's Spanish translations, in the table's order;
inverted = the Spanish Wiktionary's English translations read backwards, ordered by Spanish
frequency, as es-fr orders its inverted table by French frequency. `max_lemmas` 40,000, as en-fr.
The digest is `reduce-en-es.py`, `reduce_common.py` and `reduce_edition_es.py`.

The rule for the lemmas en-es glosses is `read_studied`'s: the lemmas `forms.tsv` maps to
themselves, ranked by `freq.tsv`, kept when their rank is within `max_lemmas` (40,000) or they are
words of `level.tsv` — the level lists' words en-fr keeps whatever their rank, 612 of English's
40,685 lemmas — so that at the default cap en-es keeps every lemma `tables/en/` commits. Why: the
pack carries en-fr's lemmas whatever en-es reads, so a cap by rank alone would leave those 612
CEFR words, which a learner is shown with a level, unglossable in Spanish though en-fr glosses
them (en-es glosses 338 of them today).

A letter glosses no word in either direction: a single letter is glossed only by a sense that is
neither the letter nor a name borrowed through it — `a` « un, una », `I` « yo » and the vocative
`O` « oh, oy » are words; « i latina » names the letter, and the note `do` names C through it.
Read forwards, `without_letter_translations` drops a `character` entry, a one-letter word whose
every translation is itself, and a one-letter word's noun entry (the letter under its name, or a
name borrowed through it: the words written as one letter are an article, a pronoun, a
preposition, a particle, never a noun). Read backwards the entry is the Spanish word's, so that
test sees nothing of the English side — the Spanish Wiktionary's `do` lists « C », and the first
tables glossed `c` (rank 376) « Do » — and `read_translated` drops every one-letter studied key of
the inverted table: the one word of a letter it reaches, `yo` « I », the entries and the direct
table gloss before it is read.

### D2 — Two derived files, one new dump source, the extract read as served

`DUMPS["en-es"]`:
- from es-fr's `kaikki-es` dump (the same address; en-es's update downloads the dump — 98 MB by
  the study's measure, recorded at the first update — and derives two files in one pass):
  `kaikki-es-English.jsonl`, `("entries", "en")`, and `kaikki-es-traductions-en.jsonl`,
  `("translations", "es", "en")` — change 21's derivation, run on en-es's own snapshot of the
  dump, since a dump record carries one release for all its files and en-es downloads the dump
  anyway; en-es's pin is self-contained, and an es-en re-snapshot moves no en-es byte;
- `kaikki-en-traductions-es.jsonl`, `("translations", "en", "es")` from a new dump source,
  `kaikki-en`: the English Wiktionary's English extract, served uncompressed — `fetch_live` saves
  a dump as it is served and `derive` reads a plain or a gzipped file, told apart by the gzip
  magic. Fetched by `fetch_live` only, derived in one pass, never kept.
All three are release assets of `lingua-pack-sources-en-es-<snapshot>`, published under
`release_tag(pair, snapshot)` with notes that name no extract (`sources.kaikki` is absent for a
dumps-only pair, and `pack_sources.py assets` lists the dump records' files alone). A pinned
reduction fetches the three derived files and nothing larger. The dump records are written as every dump's is
(release, address, fetched, last_modified, files).

Why the extract and not the raw dump risk 6 names: the raw English dump is several times the
extract and holds every language's entries; `derive` reads both, so the address is the only
difference, and the extract keeps the update within the job's reach today. If kaikki stops
serving the extract, change 38 switches the address to the raw dump; this change says so in
`SOURCES.md`.

### D3 — A floor the owner sets, kept in one place

Risk 5 names en-es; M6's rule for fr-es — a floor, and what happens below it, fixed before the
committed measurement — is applied here, and the owner sets the value on this change's pull
request, where the study's figures less two points (91.4 / 83.2 / 69.9 %) are proposed. The floor
lives in `gloss_coverage.py`'s `FLOORS["en-es"]` and nowhere else: the reduce job runs
`gloss_coverage.py --pair en-es` with no `--floor`, as es-en's line does, and the `check` job's
Python tests hold the committed tables to that entry and assert the job passes none — two
copies of one value drift (the coverage exclusions did, between the `rust` and `sonar`
workflows, until `.github/coverage-ignore-regex.txt` became their one source), and a floor the
job and the tests read from different places would fail one and pass the other. Task 5.1 writes
the owner's value into that one entry and into the requirement's text. A committed measurement
under it fails, naming the figure. Change 35 publishes the measured figures with the pair.

### D4 — The translation-table share

Among the glossed lemmas of the top 10,000, the share whose gloss came from a translation table
(direct or inverted) rather than from an entry. The reducer calls `common.reduce_gloss` and
`common.fallback_glosses` as `native_tables` does and keeps the lemma set of each step, so the
share is computed in `reduce-en-es.py` alone and `reduce_common.py` is not edited (the main
spec's *A shared rule changes*); the share is printed to stderr, and `pack_report` prints it
beside the coverage; it is shown in the pull request and in the
tables' README, not stored in the pack.

### D5 — The owner's review

A sample of 100 glosses from the top 10,000 — the definitions and the translation-table words
marked as such — in the pull request; the Spanish edition's settings stay as change 6 set them
unless the sample says otherwise.

## Risks / Trade-offs

- **The English extract's size is unknown to the repository** → measured and recorded at the
  first update; `fetch_live` runs at an update, not in the reduce job.
- **A thin pair that reads as a lesser product** (risk 5) → the floor (D3), the share (D4), and
  change 35's published figures.
- **kaikki's per-language extracts removed** (risk 6) → the address in the pin; change 38.
- **A gloss in a third language** → the tables are en→es and es→en only.

## Migration Plan

No release: tables and tooling only. en-es's first `lingua-pack-update` dispatch runs on the
pull request branch and publishes its derived files' release; the pinned reduction follows.
