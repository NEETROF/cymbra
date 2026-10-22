# Design — add-lingua-it-es-pivot-sources

## Context

Change 27a of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2, before change
27 (`add-lingua-pack-it-es`). it-es's planned sources — the Spanish Wiktionary's Italian section, then
the Italian Wiktionary's Spanish translations, then the Spanish Wiktionary's tables read backwards —
measured 57.8 / 46.8 / 35.0 % of the 5,000 / 10,000 / 20,000 commonest Italian lemmas, about 25 points
under fr-es's committed 83.2 / 70.9 / 57.1. The owner had new sources researched and settled I4 on
2026-10-23 (#1004): it-es ships reading, besides those three, the Italian Wiktionary's Spanish section
read backwards, Wikidata Lexemes, and seven other Wiktionaries' translation tables read as a pivot,
every pair kept — an exception to M5's « never a pivot through another language » that I11 records for
this pair. Apertium's GPL dictionary is not used.

This change brings those sources into the pipeline — derived, pinned, published — and nothing else:
change 27 reads them. They are a pipeline of their own (seven dumps, a new kind of derived file, a
mirrored Wikidata dump), measured apart, so the pack's change stays reviewable.

Where things stand: `pack_sources.py`'s `EDITIONS` lists, per Wiktionary dump, the files `derive`
writes from it — `("entries", lang)` or `("translations", lang, into)`, the latter keeping, for each
entry of the edition's language, its translations into one other language, a sense string beside each
word. `PINNED` names a pair's files fetched by URL at a commit. Change 16 creates `EDITIONS["it"]`.

## Goals / Non-Goals

**Goals:**
- The pivot pairs of seven Wiktionaries, the Italian Wiktionary's Spanish section and Wikidata's
  Italian–Spanish lexeme pairs derived and pinned for it-es, reproducibly.
- Nothing of another pair moves.

**Non-Goals:**
- Reading them into glosses — their order, filters, names and floor are change 27's.
- A pivot for any other pair (es-it is stage 4's to decide).
- Apertium (GPL-2.0-only) and the Open Multilingual Wordnet (a pivot through English synsets, its
  Spanish side partly machine-built, its Italian licence ambiguous) — researched, not taken.

## Measured

The owner's research (2026-10-23, read-only scratch, the repository's own `derive` and change 16's
study ranking), each source after the three planned ones:

| Source | Licence | Italian words | Coverage after the three, loose | Sample of rows added |
|---|---|---|---|---|
| Italian Wiktionary's Spanish section, read backwards | CC BY-SA 4.0 | 5,657 | 63.6 / 51.6 / 38.9 | 21 right, 7 partly, 2 wrong of 30 |
| Wikidata Lexemes (an Italian and a Spanish sense of one item, P5137) | CC0 | 4,103 pairs | 60.0 / 48.9 / 37.0 | 41 / 8 / 1 of 50 |
| The two together (A) | | | 65.3 / 53.2 / 40.5 | |
| The pivot of seven Wiktionaries | CC BY-SA 4.0 / GFDL | 80,073 | 84.6 / 76.1 / 65.7 | 40 / 8 / 2 of 50 |
| **All of them (C)** | | | **84.8 / 76.3 / 65.8** (strict 79.1 / 69.6 / 58.6) | |

- **The pivot by edition** (alone after the three): English 80.8 / 70.8 / 58.9, French 74.4 / 64.0 /
  51.7, German 75.4 / 64.4 / 51.2, Russian 70.6 / 59.9 / 47.6, Polish 70.0 / 58.8 / 46.2, Portuguese
  67.6 / 55.9 / 43.2, Dutch 61.5 / 49.9 / 37.8. Its first Spanish word agrees with the planned
  sources' words 80.4 % of the time where they gloss the same lemma (the two planned tables agree
  with each other 81.2 %). Its errors cluster on homographs and participles: `sparato` « Bragueta »,
  `sino` « Seno », `braccia` « Braza », `progettazione` « Designio ».
- **The dumps** (gzipped): German 310 MB, Russian 309 MB, Polish 138 MB, Dutch 135 MB, Portuguese 37
  MB; the English (2,981,058,381 B) and French dumps are already read for other pairs. The Wikidata
  Lexemes dump is 655,273,999 B; Wikidata keeps a dated dump about two weeks.
- **The derived pivot files**, one per edition: 1.0 to 7.5 MB (English 7.5, German 5.5, French 4.5,
  Russian 2.9, Polish 1.9, Portuguese 1.5, Dutch 1.0).

## Decisions

### D1 — A pivot file per edition, a table's sense kept whole

`derive` gains a kind, `("pivot", lang, "it", "es")`: for each entry of the edition's own language, each
translation table's sense that lists both an Italian and a Spanish word, written as one record — the
entry's word and part of speech, the sense, the Italian words, the Spanish words. A
`("translations", …)` file per language cannot be joined back on a homograph: the sense string is the
join, and it must be kept with both sides. `EDITIONS["en"]`, `["fr"]` gain
`kaikki-en-pivot-it-es.jsonl`, `kaikki-fr-pivot-it-es.jsonl`; `EDITIONS` gains `"de"`, `"pt"`, `"ru"`,
`"pl"`, `"nl"`, each with its dump's URL and its one pivot file. Every English- and French-dump
derivation then writes one more file; no other pair's `DUMPS` names it.

### D2 — The Italian Wiktionary's Spanish section

`EDITIONS["it"]` (change 16) gains `kaikki-it-Spagnolo.jsonl` (`("entries", "es")`): the Italian
Wiktionary's Spanish entries, read backwards by change 27 — a Spanish headword whose Italian gloss is a
single Italian word glosses that word. It is also es-it's future definitions file (stage 4).

### D3 — Wikidata Lexemes, mirrored

Wikidata serves its dated Lexemes dumps for about two weeks, so a pin naming one would fail after that.
The update downloads the dated dump once, the owner publishes it as a release asset of its own
(`lingua-pack-source-wikidata-lexemes-<date>`), as change 9 mirrored Morph-it!, and `PINNED["it-es"]`
names that asset by URL and sha256; a derivation writes `wikidata-lexemes-it-es.jsonl`, the pairs of an
Italian and a Spanish sense linked to the same item (P5137), with both lemmas, their parts of speech and
the item. Wikidata's licence for lexemes is CC0.

### D4 — `DUMPS["it-es"]` and the pin

`DUMPS["it-es"]` names, by edition, the three planned files, the seven pivot files, the Spanish section
and, through `PINNED`, the Lexemes file; it-es's pin records each by sha256, as every pair's does. Until
change 27 reduces it-es, no pack reads them: `DUMPS["it-es"]` exists so that change 27's first update
fetches and pins them.

### D5 — The cost of a pivot

An it-es update fetches five more dumps (≈ 0.9 GB gzipped) and reads the English one it-en already reads;
the reduce job's asset cache fetches each once. `SOURCES.md` records the sources, their licences, the
pivot's exception (I11) and this cost.

## Risks / Trade-offs

- **The pivot's errors** (homographs, participles) → change 27 orders the sources (the planned ones
  first, the pivot last), filters names, and publishes the coverage; change 28 refines what the sample
  shows.
- **Five more dumps per update** → it-es's alone; cached by sha256.
- **A dated dump that expires** → D3's mirror.

## Migration Plan

Nothing deploys and nothing is shipped: the files are derived and pinned when change 27 first reduces
it-es. Rollback is a revert.

## Effort

2.5–4 ideal days: the pivot kind in `derive` and its tests 1–1.5; the five editions and their URLs 0.25;
the Spanish section 0.25; the Lexemes mirror and its derivation 0.75–1.25; `DUMPS`, the pin, `SOURCES.md`
0.25–0.5.
