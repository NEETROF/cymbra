# refine-lingua-it-es-glosses — it-es's glosses open on the word their sources agree on, in Spanish, before Spanish speakers read Italian

## Why

Change 28 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 27
(`add-lingua-pack-it-es`, #1068) committed it-es's glosses on 2026-10-25 — 24,705 rows, coverage
**85.2 / 76.1 / 65.1 %** against the floor 83.2 / 74.0 / 63.1 — and listed what still reads wrong (its
design's *Known data defects* and *Left for change 28*, its README's *Known defects, for change 28*).
The owner approved merging it-es as the base after a specialist review of the committed tables
(2026-10-25), which this change must answer before change 30 offers Italian to Spanish speakers: **25
of the 300 commonest rows wrong (8.6 %)** and 33 partly right, 4.7 % wrong in ranks 301–5,000, 2 % in
5,001–20,000, and eleven systematic issues.

Measured on the committed tables and on it-es's pinned sources (design, *Measured*; criteria committed
before the figures):
- **The first source wins against agreement.** it-es reads seven sources, the first that glosses a word
  glossing it whole: `rumore` « Rumor » where four sources say « Ruido », `caso` « Azar », `accordo`
  « Acorde », `andare` « Andar », `corso` « Cambio », `periodo` « Menstruación », `unico` « Simple ».
- **The wrong part of speech first, or the common one missing**: `ancora` « Ancla; Todavía »,
  `spesso` « Gordo » without « a menudo », `verso` « Sentido » without « hacia ».
- **Words that are not the word**: a feminine's lent masculine (`ragazza` « Chico… », `nonna`
  « Abuelo »), words that are not Spanish (`evidente` « Evident », `obbedire` « Obéir », `ladro`
  « …, Spagna »), rare or heraldic words (`rapidamente` « Aquejadamente », `cellulare` « Movíl »,
  `controllo` « Aprobanza »), verb forms in a noun's row (`faccia` « Cara, haga, hagas »), Wikidata's
  lone items (`don` « D. », `domestico` « Gato »).
- **Notes in place of a gloss**: thesaurus dumps (`inoltre`, 166 characters), given names described
  (`paolo` « Nombre de pila de varón, equivalente del español Pablo »), a surname's note on a common word
  (`senatore` « Apellido »), repeats (`persona` « Persona (ser humano); Persona », `d'accordo`).
- **The `di` family**: `del` « Algo, un tanto…; Para; Del », `della` a description, `dei`, `degli`,
  `delle` unglossed — three of the 50 commonest words.
- **What no rule reaches**: `a` « En (cuando…) », `ci`, `te` « La, lo, te », `cui` « Quien », `posto`
  « En », `nulla`, `subito` « Súbito », `solo` « Único », `bene` « Aristocrático », `the` « Té ».

## What Changes

Every rule is it-es's own, in `reduce_italian_es.py` and `reduce-it-es.py`, which no other pair loads;
fr-en's treebank rule and MarkIT's counts are imported unchanged, as it-fr imports them (design D1).

- **Agreement** (D2): a row whose first word its own source alone gives, or that fewer sources open on
  than another word, opens on the word its sources agree on — each of the pivot's tables counting as a
  source, the winner of the row's part of speech, a reviewed row never moved — and a row Wikidata's lone
  items gloss takes the next agreed source: **1,485 rows**, and 99 (`caso` « Caso », `andare` « Ir,
  andar », `rumore` « Ruido; Rumor », `corso` « Curso… », `europeo` « Europeo », `don` « Don »).
- **Parts of speech** (D3): MarkIT's order for a function word's row, it-fr's rule (9 rows: `ancora`
  « Todavía; Ancla », `perché` « Porque; … »); a missing adverb or preposition filled from the next
  source (`spesso` « A menudo…; Gordo », `verso` « Hacia; Sentido »). MarkIT joins it-es's pin.
- **Words read as their sources mean them** (D4–D6, D10): a feminine's own word for its lent masculine
  (5 rows: `ragazza` « Chica, muchacha, niña »); words that are not Spanish, by wordfreq across five
  languages (303 words, 40 rows), and place names as labels (`ladro`); a rare first word yields to a
  commoner one (837 rows: `rapidamente` « Rápidamente… »), heraldic senses out; verb forms off other rows
  (`faccia` « Cara », `dati` « Datos, facto »).
- **Notes and repeats** (D7–D9): lists of eight items or more cut at three (`inoltre`); given names read
  as their Spanish name (`paolo` « Pablo (nombre de pila) », 102 rows); a surname's note off a common word
  another source glosses (`senatore` « Senador »); taxa and repeated senses out (`cavallo` « Caballo »,
  `persona`, `d'accordo` « De acuerdo, vale »).
- **The `di` family** (D11): a reviewed table, `del` « Del », `della` « De la », `dei` and `degli` « De
  los », `delle` « De las » — four written by a person, marked so.
- **`FIRST_RUN` and `CORRECTIONS`** (D12): 11 rows of the 300 opened on a sense a source gives (`posto`
  « Lugar, sitio, puesto », `subito` « Inmediatamente… », `bene`, `solo`, `che` « Que; Qué »), 6
  corrections (`a` « A; En (…) » and `te` « Ti, te », each with one word a person wrote; `cui` « Que;
  Cuyo »; `the` unglossed, as it-en leaves it).
- **Measured and left** (D13): ten partly right rows of the 300 (`vi`, `oltre`, `conto`), the wrong rows
  past the 300 no rule reaches (task 0.2), the pivot's homographs, the Spanish edition's gaps.
- **One re-reduction** (D14): a dispatch (`lingua-pack-update`, mode `reduce`, it-es) at it-es's pin, to
  which the implementation's first commit adds MarkIT's two sections by hand; together the rules change
  **2,066 rows** (145 / 384 / 776 / 761), 2,018 in their first sense, `dei`, `degli`, `delle` gained,
  `the` lost; coverage **85.3 / 76.1 / 65.1 %**; on the review's judged rows the 300 commonest read
  wrong **0 %** (from 8.6 %) and partly right 3.4 % (from 11 %), as this design's reader reads them, for
  the specialist to confirm; Italian's studied tables, the eight other pairs and the PUD gates byte for
  byte; `it-es.golden` and the Spanish card re-blessed once by the update workflow (D15).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED *it-es's glosses open on the Spanish word its sources agree on, in the order Italian uses the
    word*, *it-es's glosses hold Spanish words, read as their sources mean them*, *it-es's glosses hold
    no name's note, list or repeat a common word does not need* and *it-es glosses the `di` family and
    corrects, by name, what its sources write*.
  - MODIFIED, copied whole from the open change that holds it, the clauses this change edits named in
    the pull request: *Italian is glossed in Spanish from the Spanish Wiktionary's Italian section, the
    two Wiktionaries' tables, Wikidata and a pivot* (`add-lingua-pack-it-es`: the first source glosses a
    lemma but where agreement moves its row; a gloss a reviewed table writes marked as a person's).
  - This change archives after `add-lingua-pack-it-es` (`archiveAfter`). Change 30 renames change 27's
    other requirement, which this change does not touch.
- `lingua-analysis`: none. The analysis reads it-es's glosses as written.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce_italian_es.py` (the rules, `FIRST_RUN`, `CORRECTIONS`,
    `DI_FAMILY`), `reduce-it-es.py` (each source's gloss kept, the passes' order, MarkIT read from the
    work folder), `test_reduce_it_es.py`, `test_reduce_editions.py`; `pack_sources.py`
    (`PINNED["it-es"]`, MarkIT's two sections) and `test_pack_sources.py`; `tables/it-es/` (`pin.json`,
    MarkIT's two sources by hand, then `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json`,
    `README.md` by the dispatch); `SOURCES.md`; *consumed*: `reduce_common.py`, `reduce_edition_es.py`,
    `reduce_spanish_tables.py`, `reduce_italian_acronyms.py`, `reduce_french_treebank.py`,
    `reduce_italian_treebank.py`, `gloss_coverage.py`, unchanged.
  - `crates/lingua-wasm/tests/baseline/it-es.golden` re-blessed; `crates/lingua-pack/tests/committed_tables.rs`
    against the new pin, its list of it-es's source keys naming MarkIT's two.
  - `apps/lingua-extension/test/baseline/word-card-it-es.txt` re-blessed once.
  - ID, Music, Live, the back office, the site, the backend, lingua-core and the engine, the
    extension's source, the Apple host app and the agent plugin are untouched.
- **What does not move.** `tables/it/` and the eight other pairs — tables, pins, packs, goldens and
  snapshots — byte for byte; the shared rules and the Spanish edition's; `FLOORS["it-es"]`; Italian's
  analyser version; UD Italian-PUD's figures.
- **Release.** Silent: no package lists it-es before change 30.
- **Order.** After change 27 (on `main`, its requirement still open), whose requirement this change
  modifies; before change 30; either side of change 23, which edits the same lines of
  `pack_sources.py`'s `PINNED` for it-en.
- **The owner's**: two questions (tasks 0.1, 0.2); the gloss diff after the specialist's review, the
  golden's and the card's moved lines (task 4.1).
- **Effort**: 3.5–5 ideal days (design, *Effort*), against the programme's 2–3.5.
