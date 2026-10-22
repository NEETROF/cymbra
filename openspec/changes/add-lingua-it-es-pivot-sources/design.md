# Design — add-lingua-it-es-pivot-sources

## Context

Change 27a of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2, before change
27 (`add-lingua-pack-it-es`). it-es's planned sources — the Spanish Wiktionary's Italian section, then
the Italian Wiktionary's Spanish translations, then the Spanish Wiktionary's tables read backwards —
measured 57.8 / 46.8 / 35.0 % of the 5,000 / 10,000 / 20,000 commonest Italian lemmas, about 25 points
under fr-es's committed 83.2 / 70.9 / 57.1. The owner had new sources researched and settled I4 on
2026-10-23 (#1004): it-es also reads the Italian Wiktionary's Spanish section read backwards, Wikidata
Lexemes, and seven other Wiktionaries' translation tables read as a pivot, every pair kept — an
exception to M5 that I11 records for this pair. Apertium's GPL dictionary is not used.

This change registers and derives every file it-es reads — the planned three and the new ones — so that
change 27's first update fetches, derives, publishes and pins them. Change 27 reads them into glosses.

Where things stand: `pack_sources.py`'s `EDITIONS` lists, per Wiktionary dump, the files `derive` writes
from it — `("entries", lang)`, or `("translations", lang, into)`, which keeps, for each entry of the
edition's language, its translations into one other language with the table's `sense` string when it has
one. `read_edition` fetches a dump once per run and derives its catalogue; `fetch_live` compresses the
derived files and the workflow publishes them in the pair's sources release, which the reduce job's asset cache reads back by sha256; a dump itself is
never cached. `PINNED` names a pair's files fetched as served, at a commit. Change 16 creates
`EDITIONS["it"]`. A pair's pin records the files of `DUMPS[pair]` alone, and `check_registered` holds
every one of them to an edition's catalogue.

## Goals / Non-Goals

**Goals:**
- Every file it-es reads registered, derived and pinned, reproducibly: the planned three, the pivot of
  seven Wiktionaries, the Italian Wiktionary's Spanish section, Wikidata's Italian–Spanish lexeme pairs.
- Nothing of another pair moves.

**Non-Goals:**
- Reading them into glosses — their order, filters, names and floor are change 27's (*What change 27
  takes from here*).
- A pivot for another pair (es-it is stage 4's to decide).
- Apertium (GPL-2.0-only) and the Open Multilingual Wordnet (a pivot through English synsets, its Spanish
  side partly machine-built, its Italian licence ambiguous) — researched, not taken.

## Measured

The owner's research (2026-10-23, read-only scratch, the repository's own `derive` and change 16's study
ranking), each source after the three planned ones; re-checked by the review from the research's
outputs:

| Source | Licence | Italian words | Coverage after the three, loose |
|---|---|---|---|
| Italian Wiktionary's Spanish section, read backwards | CC BY-SA 4.0 + GFDL | 5,657 | 63.6 / 51.6 / 38.9 |
| Wikidata Lexemes (an Italian and a Spanish sense of one item, P5137) | CC0 | 2,904 (4,103 pairs) | 60.0 / 48.9 / 37.0 |
| The two together (A)¹ | | | 65.3 / 53.2 / 40.5 |
| The pivot of seven Wiktionaries | CC BY-SA 4.0 + GFDL | 80,073 | 84.6 / 76.1 / 65.7 |
| **All of them (C)**¹ | | | **84.8 / 76.3 / 65.8** (strict 79.1 / 69.6 / 58.6) |

¹ A and C also count the lexemes' translation links (P5972, 193) and Spanish glosses on Italian senses
(8): one row of C in 20,000, two of A; derived in the same pass (D3).

- **The pivot by edition** (alone after the three): English 80.8 / 70.8 / 58.9, French 74.4 / 64.0 /
  51.7, German 75.4 / 64.4 / 51.2, Russian 70.6 / 59.9 / 47.6, Polish 70.0 / 58.8 / 46.2, Portuguese
  67.6 / 55.9 / 43.2, Dutch 61.5 / 49.9 / 37.8. Ordered by support across editions, its first Spanish
  word agrees with the two planned translation tables' words on 80.4 % of the 6,041 lemmas both gloss
  (those two tables agree with each other 81.2 %); one edition alone agrees 68.0 % (English) to 81.7 %
  (Portuguese). Its errors cluster on homographs and participles: `sparato` « Bragueta », `sino`
  « Seno », `braccia` « Braza ».
- **How tables name their senses**: of 175,556 pivot rows, the Polish ones are keyed by a sense number
  alone (16,498 of 16,503); 50,392 (28.7 %) name no sense at all — French 25,568, Russian 13,475,
  Portuguese 10,147, Dutch 1,197 —, 14,044 of them on entries with more than one sense; 166 French
  tables are « Traductions à trier suivant le sens », mixing senses.
- **The dumps** (gzipped, read on 2026-10-22): English 2,981,058,381 B and French 0.74 GB (already read
  for other pairs, not by it-es's planned sources), German 310,139,452, Russian 308,932,290, Polish
  137,850,882, Dutch 134,749,717, Portuguese 37,273,122; Wikidata's Lexemes dump
  `wikidata-20261021-lexemes.json.gz`, 655,273,999 B, published weekly and kept about six weeks.
- **The derived pivot files**, one per edition: 7.8, 5.8, 4.8, 3.0, 2.0, 1.5 and 1.0 MB (English,
  German, French, Russian, Polish, Portuguese, Dutch).

## Decisions

### D1 — The planned three, registered

`EDITIONS["es"]` gains `kaikki-es-Italiano.jsonl` (`("entries", "it")`) and
`kaikki-es-traductions-it.jsonl` (`("translations", "es", "it")`); `EDITIONS["it"]` (change 16) gains
`kaikki-it-traductions-es.jsonl` (`("translations", "it", "es")`) — so `DUMPS["it-es"]` names files an
edition's catalogue holds.

### D2 — A pivot file per edition, kept by the sense the table names

`derive` gains a kind, `("pivot", lang, "it", "es")`: for each entry of the edition's own language, its
translations into Italian and into Spanish grouped by the table's `sense`, else its `sense_index`, else
the sense it sits under, else the whole entry, as measured; one record per group — the entry's word and
part of speech, `sense` and `sense_index` where the table gives them, whether the entry has more than one
sense, the Italian words, the Spanish words. A `("translations", …)` file per language would drop
`sense_index` and lose the entry's identity: Polish would collapse to one group per entry. A group with
words of one language alone gives nothing. `EDITIONS["en"]` and `["fr"]` gain
`kaikki-en-pivot-it-es.jsonl` and `kaikki-fr-pivot-it-es.jsonl`; `EDITIONS` gains `"de"`, `"pt"`, `"ru"`,
`"pl"`, `"nl"`, each with its dump's URL and its one pivot file. Every English- and French-dump
derivation then writes one more file; no other pair's `DUMPS` names it, so no other pin moves (a pin
records `DUMPS[pair]` alone).

### D3 — The Italian Wiktionary's Spanish section, and Wikidata Lexemes as an edition of its own

- `EDITIONS["it"]` gains `kaikki-it-Spagnolo.jsonl` (`("entries", "es")`): the Italian Wiktionary's
  Spanish entries, read backwards by change 27 — the first gloss of each sense that is not a form, on a
  lowercase Spanish entry that is not a name or a character, split on « , » and « ; » into at most three
  parts of at most two words each, as measured. It is also es-it's future definitions file (stage 4).
- **Wikidata Lexemes** are read like a Wiktionary dump: an edition `"wikidata"` whose URL is
  `https://dumps.wikimedia.org/wikidatawiki/entities/latest-lexemes.json.gz`, the dated dump it serves
  recorded by its Last-Modified, decompressed sha256 and size, as kaikki's dumps are, and one derived
  file, `wikidata-lexemes-it-es.jsonl`: the pairs of an Italian and a Spanish sense
  linked to the same item (P5137), the translation links between an Italian and a Spanish sense (P5972),
  and the Spanish glosses of Italian senses, each with both lemmas, their parts of speech and the item.
  Published zstd-compressed in it-es's own sources release like every derived file, it is read back from
  there by a re-reduction; no mirror of the dump is kept. The kind has its own reader: the dump is one
  JSON array whose entity lines end with « , », which `derive`'s line reader would skip silently, and a
  pair needs every Italian (`Q652`) and Spanish (`Q1321`) lexeme read before it is written — so the
  reader strips the comma, keeps those two languages, collects, then writes. Wikidata's licence for
  lexemes is CC0. The pin's record of it is named for Wikidata, not kaikki.

### D4 — `DUMPS["it-es"]` and the pin

`DUMPS["it-es"]` names, by edition, the planned three (D1), the seven pivot files (D2), the Spanish
section and the lexeme pairs (D3); it-es's pin records each by sha256. Until change 27 reduces it-es, no
pack reads them.

### D5 — The cost of a pivot

An it-es update fetches the Spanish and Italian dumps of its planned sources, and, for the pivot, seven
more — the English (2.98 GB), French (0.74 GB) and five others (0.93 GB), ≈ 4.65 GB gzipped — and the
Lexemes dump (0.66 GB). Dumps are never cached: `read_edition` fetches each once per run; a re-reduction
from the pin reads the derived files from the release. The monthly dry run of every pair reads these
dumps too once it-es has tables. `SOURCES.md` records the sources, their licences — CC0 joining the
licences allowed —, the pivot's exception (I11) and this cost.

### D6 — What change 27 takes from here

The pack's change must, from these files:
1. fix the floor at its task 0.1 on the committed ranks less two points, and name this change in
   `archiveAfter`;
2. filter as measured — no pivot entry tagged name, abbreviation, symbol, character, letter, affix or
   punctuation, nor one-letter headwords; no capitalised headword (German's nouns excepted); no
   capitalised Italian or Spanish word but acronyms of at most six letters; no Spanish word with a grave
   accent, « ° », or longer than sixty characters;
3. order a pivot's Spanish words by support across tables and editions, then Spanish frequency, then the
   alphabet (one edition alone agrees 68–82 %, the support order 80.4 %), at most three per part of
   speech, the part of speech the pivot entry's;
4. fix and measure the sources' order (the research read lexeme links, lexeme items, the Spanish section,
   then the pivot);
5. decide whole-entry groups on entries of several senses and the « à trier » tables (sense-only:
   83.4 against 83.8 at the 5,000, four editions);
6. decide whether the pivot supplies expressions;
7. carry provenance in the README's sample (edition, entry, sense) and publish coverage per source;
8. credit the seven Wiktionaries and Wikidata in the notice and manifest, Wikidata's CC0 needing a
   licence variant in `crates/lingua-pack/src/licence.rs` or its `Permissive` one;
9. MODIFY the main spec's *A gloss is in the reader's language, written by a person*, whose « never
   pivoted through a third language » and scenario *No pivot through English* the pivot makes false for
   it-es — the exception I11 records.

## Risks / Trade-offs

- **The pivot's errors** (homographs, participles) → change 27's order, filters and published coverage;
  change 28 refines what the sample shows.
- **Groups that name no sense** (28.7 %) → kept under the entry, marked so; change 27 decides (D6.5).
- **≈ 5.3 GB of dumps per it-es update** → it-es's alone; a re-reduction reads the release.

## Migration Plan

Nothing deploys and nothing is shipped: the files are derived and pinned when change 27 first reduces
it-es. Rollback is a revert.

## Effort

2.5–4 ideal days: the pivot kind in `derive` and its tests 1–1.5; the five editions and their URLs, the
planned three registered 0.25; the Spanish section 0.25; the Lexemes edition and its derivation
0.75–1.25; `DUMPS`, the pin, `SOURCES.md`, the edited tests 0.25–0.5.
