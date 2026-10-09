# Design — add-lingua-pack-fr-es

## Context

See proposal.md (Why). Where fr-es stands, and what this design builds on:

| What | Where, today |
|---|---|
| French's studied side | change 43 (`add-lingua-french-forms-tables`, proposed, implementation in progress on `claude/add-lingua-french-forms-tables-impl`): `reduce-fr-en.py`, French's reference reducer, writes `tables/fr/` — `forms.tsv` (124,050 rows), `freq.tsv` (60,000 lemmas), `studied.json` naming fr-en, an empty `tags.tsv` and an empty `lexical.tsv`; `build.sh` caps every `fr-*` pair at 60,000 lemmas; `lingua-pack-update` runs one French dispatch at a time (its `fr` group). Changes 45 and 46 add `grammar.tsv`, the pinned tag pool and `level.tsv` |
| French's dictionary words | change 48 (`add-lingua-pack-fr-en`, PR #824, under review): fr-en glosses from the English Wiktionary's French section alone, no translation table; `split` writes its glossed lemmas — 30,056 on change 43's implemented tables — to `tables/fr/lexical.tsv`; « 49 fr-es: after: French's dictionary words, the cross-native test for French (two natives) » |
| A reader pair | change 21 (`add-lingua-pack-es-en`): the native side alone from the committed studied tables, no other pair's reducer loaded, a pin recording the six studied tables it read and a `pack_version` naming their digest, its own release, reduced after its reference, measured against a floor before it ships |
| The closest precedent | change 22 (`add-lingua-pack-en-es`, merged): the Spanish Wiktionary's English section, then the English Wiktionary's Spanish translations (direct, the table's order), then the Spanish Wiktionary's English translations read backwards (inverted, by Spanish frequency); a letter glosses no word; `FLOORS["en-es"]` the one place its floor lives, set by the owner on its implementation's pull request (its task 5.1, open); the translation-table share in its README |
| The Spanish edition's rules | `reduce_edition_es.py`: `ES` (pointer wordings, notes — sense-link subscripts, and since 24b maintenance templates, disambiguation notes, numbered-sense references, the expansion notice, a usage note after the meaning —, letters; `capitalised`), `read_as_meanings` (senses marked obsolete or outdated after the others; `typography`: one « … », straight quotes paired « »). Loaded by en-es alone today; 24b (`refine-lingua-en-es-glosses` D1): « fr-es (change 49) will load them for the French section and measure them there » |
| en-es's pair rules (24b) | in `reduce-en-es.py`: `english_entries` (a name's note left off the common word, possessive and demonstrative adjectives as determiners, « forma en -ing »), `read_translated` (letters, translators' notes and disused words left out, a Spanish word read backwards listed once by the studied word's readings) |
| The catalogue | change 38's `EDITIONS`: `kaikki-es-Frances.jsonl` (« fr-es's glosses (change 49) »), `kaikki-fr-traductions.jsonl` (« es-fr's inverted fallback, fr-es's direct one »), `kaikki-es-traductions.jsonl` (« es-fr's direct fallback, fr-es's inverted one ») |
| es-fr, M6's precedent | the French Wiktionary's Spanish entries, then the Spanish Wiktionary's French translations, then the French Wiktionary's Spanish translations read backwards; 17,420 of its 22,826 glossed lemmas from a definition (76.3 %); its coverage published on the site, 87.6 / 77.2 / 63.7 % (`apps/site/src/data/lingua-coverage.json`), with no floor |
| Coverage | `gloss_coverage.py`: the share of the 5,000 / 10,000 / 20,000 commonest lemmas of `tables/<studied>/freq.tsv` glossed; `FLOORS` es-en and en-es (fr-en's added by change 48); the reduce job runs `--pair` for each with no `--floor`; `--write` lists the shipped pairs alone |

## Goals / Non-Goals

**Goals:**
- fr-es's Spanish glosses, sense runs and expressions, written by people in Spanish (M5), committed and
  pinned beside French's studied tables.
- M6: a floor and what happens below it, written before the measurement that commits the tables.
- en-es's lessons applied where the data has the same defects, measured on fr-es's own rows.
- en-fr, es-fr, es-en, en-es and fr-en byte for byte; `tables/fr/` read, never written.

**Non-Goals:**
- Shipping fr-es (52: `packs.json`, the published coverage, the listings); the Spanish card for French
  and fr-es's golden (51); the marks (50).
- A rule of `reduce_common.py` or of the Spanish edition (either would re-pin en-es); French's studied
  side (43, 45, 46).
- The defects left (*Known data defects*): measured and listed, for the owner or a refinement.

## Measured

A prototype in the scratchpad (never committed), run on:
- **French's tables**: change 43's implementation as reduced on its branch (commit `5c0a8b3e`,
  snapshot 2026.10.08: 124,050 forms, 60,000 lemmas); French's dictionary words as change 48's
  prototype makes them on those tables (30,056 lemmas, the figure its review measured); change 45's
  prototype readings (`grammar.tsv`, its tag pool) and change 46's prototype levels, for the pack's
  size and the readings D6 reads.
- **The sources**: the files change 38's measurement derived on 2026-10-08 from the Spanish
  Wiktionary's dump regenerated on 2026-10-02 12:12 (sha256 `46e1f04f…`, 1,233,016,167 B
  decompressed) — `kaikki-es-Frances.jsonl` (8,683 entries, 7,438,610 B, sha256 `2e724e39…`) and
  `kaikki-es-traductions.jsonl` (18,449 entries, 1,537,580 B, `bf44ebb4…`) — and from the French
  Wiktionary's dump regenerated on 2026-10-02 00:10 (`968f7df0…`, 6,865,136,428 B) —
  `kaikki-fr-traductions.jsonl` (56,466 entries, 6,191,621 B, `bd14a5c4…`). No dump was fetched
  again. The two translation files are byte for byte the ones es-fr pins: es-fr's committed tables
  were reduced from the same two regenerations.
- **The rules** of `origin/main` `35faf774` (`reduce_common.py`, `reduce_edition_es.py` with 24b's
  rules), wordfreq 3.1.1 under Python 3.12; packs built by `lingua-pack-build` from the same commit.

Coverage is `gloss_coverage.py`'s, on `tables/fr/freq.tsv`.

**The sources.** Each step glosses what the steps before it left out, words and expressions alike.

| Native side | Glossed lemmas | 5k / 10k / 20k | Expressions | Pack |
|---|---|---|---|---|
| The Spanish Wiktionary's French section alone | 4,569 | 40.0 / 28.5 / 18.3 % | 587 | 1,338,865 B |
| + the French Wiktionary's Spanish translations (direct) | 17,756 | 81.3 / 68.2 / 53.8 % | 11,273 | — |
| + the Spanish Wiktionary's French translations, backwards (inverted): en-es's and es-fr's shape, the programme's « 83.4 / 70.8 / 56.5 % » | 19,061 | 83.4 / 70.9 / 56.9 % | 12,177 | 1,693,713 B |
| The inverted table before the direct one | 19,061 | 83.4 / 70.9 / 56.9 % | 12,177 | — |
| The section and the inverted table, no direct | 11,209 | 67.5 / 53.4 / 38.9 % | 1,973 | — |
| The two tables, no definition | 18,211 | 79.1 / 67.0 / 53.9 % | 11,932 | 1,661,826 B |
| **This design (D2–D7)** | **19,050** (4,560 / 13,195 / 1,295) | **83.2 / 70.8 / 56.8 %** (4,161 / 7,081 / 11,359) | **12,177** (587 / 10,686 / 904) | **1,693,382 B** |
| This design, expressions from the section alone | 19,050 | 83.2 / 70.8 / 56.8 % | 587 | 1,497,212 B |

The programme's « 24 % of them definitions » is the share of all glossed lemmas a definition glosses:
24.0 % in the three sources' shape, **23.9 % in this design** (4,560 of 19,050). Among the glossed
lemmas of the 10,000 commonest — en-es's measure — **40.2 %** are definitions (2,849 of 7,081) and
59.8 % translation-table words (3,967 direct, 265 inverted); 48.1 % of the top 5,000's, 32.1 % of
the top 20,000's. en-es's share from a table is 26.0 % of its glossed top 10,000; es-fr's was 23.7 %
of all its glossed lemmas when its glosses were first committed.

The French Wiktionary's translators write a Spanish word per sense of a French entry, and the table
glosses most of fr-es: 13,195 lemmas. Its gloss is a word, not a meaning, but a word a person chose
(« répondre » « Contestar, responder », « bande » « Banda, faja, venda »). The Spanish Wiktionary's
French section is short and plain (11,143 senses, mostly translation-like: « maison » « Casa »,
« faire » « Hacer; (faire chaud, faire froid) hacer; … »), and wins wherever it has an entry.

**What none of them glosses.** 839 of the top 5,000: English words and names in French text (`the`,
`of`, `york`, `david`), initialisms and abbreviations (`km`, `tv`, `pq`), unaccented spellings
(`etat`, `meme`), and common French words neither edition translates into Spanish (`part`, `lors`,
`afin`, `taux`, `retrouver`, `tandis`, `classement`, `réellement`): no entry in the Spanish
Wiktionary's French section, no Spanish word in the French Wiktionary's table. fr-en glosses 529 of
them; fr-es glosses 12 / 40 / 126 lemmas of the top 5,000 / 10,000 / 20,000 that fr-en does not,
544 in all, none of them a dictionary word of French (`quant`, `onu`, `vih`, but also `el`, `for`,
`last`, `okay`).

**24b's rules on fr-es.** Each rule alone, then together, against the three sources with the
Spanish edition as committed. Rows / top 10,000.

| Rule | fr-es | First sense | Expressions | en-es (24b) | Examples |
|---|---|---|---|---|---|
| The edition's notes (`ES`, 24b D2: templates, disambiguation, numbered senses, expansion notice, a usage note after the meaning) — already the edition's, measured against the edition before 24b | 3 / 3 | 2 / 2 | 0 | 15 / 12 | « madame » « Señora (vocativo). Se utiliza en presencia de… » → « Señora (vocativo) », « mademoiselle », « de »; no template, disambiguation note, reference or notice occurs in the section |
| Senses marked obsolete or outdated after the others (`read_as_meanings`, 24b D5) | 19 / 7 | 10 / 2 | 1 | 41 / 27 | « chapelet » « Guirnalda; Rosario » → « Rosario; Guirnalda »; « rien », « mec », « foutre »; « baiser » opens on « Coger (sexualmente) », its « Besar » labelled outdated |
| One typography (`typography`, 24b D6), senses and table words | 0 | 0 | 3 | 39 / 26 | « prière de » « … Se ruega que … », « trêve de plaisanterie » « Bromas aparte… »; the section's 50 straight quotes sit in pointer senses |
| A name's note does not gloss the common word (24b D3, fr-es's pass) | 17 / 12 | 8 / 6 | 0 | 61 / 53 | « jean » no longer opens on « Nombre de pila de varón, equivalente del español Juan »; « pierre » « Piedra », « rose », « romain » « Romano », « gay », « royal », « gagner » without « Apellido » |
| Possessives and demonstratives are determiners, and their forms (24b D4, fr-es's pass) | 7 / 7, runs alone | 0 | 0 | 6 / 6 | `mon`, `ma`, `ton`, `ta`, `notre`, `ce` ADJ → DET; `mes` keeps « Mi », borrowed from `mon` as a determiner |
| Translators' notes and disused words (24b D7) | 0 | 0 | 0 | 136 / 28 | about twenty notes in 78,472 translations (« embaucar (1) », « afanar (se) »), none in a word a gloss keeps: not ported |
| A Spanish word read backwards listed once (24b D7) | 94 / 31 | 6 / 2 | 0 | 58 / 10 | « cet » « Este; Este » → « Este », « capital », « joli » « Lindo, bonito, guapo; Guapo » → « Lindo, bonito, guapo », « vigueur » |
| A letter glosses no word (en-es D1, the tables) | 11 / 11 lose a gloss | — | 0 | (en-es's first tables) | `h` « H », `x` « X », `r`, `b`, `o`, `g`, `k`, `w`, `z`, `q`, `i` « I latina, i »; `à`, `y`, `ô` keep their definitions |
| The studied word is no definition (D7, fr-es's) | 9 / 4 | 9 / 4 | 0 | — | « et » (rank 3) « Et » → « Y, e »; « troll » « Trol », « slip » « Calzoncillos, braguitas », « clochard » « Mendigo », « yucca » « Yuca »; « élite » « Élite » → « Elite » |
| **Together** | **149 / 64** | **33 / 14** | **4** | 295 / 149 | 7 / 7 runs alone; 11 letters lose their gloss; none gained; coverage 83.4 / 70.9 / 56.9 → 83.2 / 70.8 / 56.8 % |

A typographic apostrophe in a French headword read as `'` (D2) gains 4 lemmas — `main-d'œuvre`
« Mano de obra » (3,053), `chef-d'œuvre`, `inch'allah`, `qu'en-dira-t-on` — and 900 expressions
(« acte d'accusation », « aller de l'avant »): 6 headwords of the section, 1,317 of the direct table,
37 French words of the inverted one.

**What the expressions are met as.** Keyed by their words and counted in UD French's 425,111
syntactic words (GSD's three sections and PUD; UD writes `du` as `de le`, so it is an approximation,
change 48's measure): the section's 587 expressions occur 5,609 times (155 distinct), the direct
table's 10,686 occur 5,348 times (905 distinct), the inverted table's 904 occur 484 times (88
distinct). The commonest read right or as their words do: « lors de » « Durante » (260), « les
deux » « Ambos » (209), « plus tard » « Más tarde », « plus de » « Más » (195), « un autre » « Otro »
(72); « à la » « A la » (1,209) is compositional and harmless. The noisiest is the section's own
« de la » « Algo de, un poco de, una cierta cantidad » (3,340), the partitive the English
Wiktionary writes for fr-en too (« of the; some, the feminine partitive article »): how a French
expression is keyed is change 44's. Unlike fr-en's inverted table (« il est » "he's", 782 times,
change 48 D3), no table here turns French's commonest bigrams into another word's translation, and
fr-es is not French's reference pair: its glosses make no dictionary word. The tables' expressions
cost 196 KB of pack.

**The pack.** 1,693,382 B with change 43's tables and fr-en's dictionary words: gloss 178,768 B,
senses 57,049 B, expressions 211,561 B, the lexical table 7,500 B (fr-es glosses other lemmas than
fr-en, so it carries French's dictionary words); 1,911,078 B with changes 45's and 46's prototype
tables. Built beside fr-en's prototype pack from the same studied tables, the studied sections —
forms, lemmas, ranks, levels, paradigms and the tag pool — are byte for byte alike. en-es's
pack is 1,688,931 B, fr-en's 2,422,318 B with the same prototype tables. No committed row is empty
or ends on an opening mark under the row cut (`rowGloss`, measured on the 19,050 rows).

**The sample.** 100 rows of the top 10,000, systematic in rank order (every 70.8th glossed lemma):
40 definitions, 55 direct, 5 inverted. They read right but for « siège » « …; Local, stilla »,
« pieux » « Piadoso, devoto, pièsa », « caste » « Casta, caste », « maçon » opening on
« Ladrillador », and the page's own « de » « De (indica posesiva); De (indica asociación o
propiedad); … ». The pull request draws it again from its own tables for the owner (task 6.1).

**The floor's effect** (D8). Against 81.4 / 68.8 / 54.5 %:

| Native side | 5k / 10k / 20k | Against the floor |
|---|---|---|
| The section and the direct table | 81.3 / 68.2 / 53.8 % | under at all three |
| The section and the inverted table | 67.5 / 53.4 / 38.9 % | under at all three |
| The two tables, no definition | 79.1 / 67.0 / 53.9 % | under at all three |
| **This design** | **83.2 / 70.8 / 56.8 %** | **+1.8 / +2.0 / +2.3 points** |
| es-fr's published figures, for comparison | 87.6 / 77.2 / 63.7 % | 4.4 / 6.4 / 6.9 points above this design |

The floor holds fr-es to all three sources: losing any of them — a source kaikki stops deriving, a
table emptied by a schema drift — fails it at every top.

## Decisions

### D1 — fr-es, a reader pair of French

`reduce-fr-es.py` writes the native side alone, as `reduce-en-es.py` writes en-es's (change 21 D1):

- **French's tables read as committed** (`LINGUA_STUDIED`, `tables/fr/`): the lemmas `forms.tsv` maps
  to themselves, ranked by `freq.tsv`, within `--max-lemmas` — 60,000 by `build.sh`'s `fr-*` cap,
  every lemma French commits —, and `grammar.tsv`'s readings for D6. A folder whose ranks and forms
  disagree is refused, naming both files, as en-es refuses one.
- **`FR`**, French's `common.Studied`, repeated as en-es repeats English's — fr-en's reducer is not
  loaded: change 43's token pattern (letters with French's accents, ligatures and diaeresis, inner
  hyphens and apostrophes, an elided piece's final one) and French's seven coordinators (*et*, *ou*,
  *mais*, *ni*, *or*, *car*, *donc*), the `FR` change 48 writes.
- **`EDITION = spanish.ES`**, the Spanish edition's rules; `LOCUTIONS` empty, as every pair's.
- **The manifest** studies `fr`, is glossed in `es`, names French's analyser version as `analysis/mod.rs`
  writes it (`FRENCH_ANALYZER_VERSION`); **the notice and the manifest** credit both sides — the
  English Wiktionary's French section, wordfreq and UD French-GSD for French's tables and its
  dictionary words, as fr-en's notice credits them; the Spanish Wiktionary for the definitions and the
  French translations read backwards; the French Wiktionary for the Spanish translations; wordfreq for
  the Spanish words' order.
- **Its rule digest** is `reduce-fr-es.py`, `reduce_common.py` and `reduce_edition_es.py`: no other
  pair's reducer, so en-es's, es-fr's and fr-en's rules never reach it but through the committed
  tables. From this change on, an edit of the Spanish edition re-pins en-es and fr-es (D10).

### D2 — Three sources, definitions first

`native_side`, en-es's step by step form of `common.native_tables` (so each source's lemmas are kept
for D9): the Spanish Wiktionary's French entries gloss first — eight whole senses, grouped by part of
speech —, then the French Wiktionary's Spanish translations, in the table's order, then the Spanish
Wiktionary's French translations read backwards, the commonest Spanish word first by wordfreq `es`;
at most three Spanish words per part of speech, as one sense, opening on a capital as the edition's
senses do. Expressions take the same three steps. A proper noun's translation glosses nothing
(`read_translations`). This is es-fr's and en-es's shape, and the programme's measure.

Before any of them read it, a French headword's typographic apostrophe is read as `'` — in the
section's headwords, the direct table's French entries and the inverted table's French words —, as
French's forms are (change 43 D3) and fr-en's headwords (change 48 D1): `main-d’œuvre` is the
committed lemma `main-d'œuvre`.

*Rejected — the definitions alone.* 40.0 / 28.5 / 18.3 %: a Spanish speaker would meet an
unglossed word in every other line.
*Rejected — the tables before the definitions.* The same coverage — the union of the three sources,
in any order (the inverted table before the direct one gives 83.4 / 70.9 / 56.9 % too) —, and a
definition a person wrote for the French word would give way to the first words of a list.
*Rejected — a pivot through English* (the English Wiktionary lists French and Spanish under one
English sense): M5, and *A gloss is in the reader's language, written by a person*.

### D3 — The catalogue's three files, one release of fr-es's own

`DUMPS["fr-es"] = {"es": ("kaikki-es-Frances.jsonl", "kaikki-es-traductions.jsonl"), "fr":
("kaikki-fr-traductions.jsonl",)}` — the catalogue's files, no new derivation (change 38, *A pair of
stage 3 registers what it reads*). An update of fr-es fetches the Spanish and the French editions'
dumps (103 MB and 737 MB gzipped), derives the three files in one pass of each, records each dump in
the pin and publishes the files under `lingua-pack-sources-fr-es-<snapshot>` (15.2 MB raw); a
re-reduction fetches those three and no dump. fr-es's pin records its own release (change 21 D2)
even when es-fr's update read the same regeneration: their two translation files then carry one
sha256, and the reduce job's asset cache fetches each once.

The pin's `studied` record names fr-en and the sha256 of the six studied tables the build read
(`forms`, `freq`, `grammar`, `level`, `lexical`, `tags`), and `pack_version` their digest (change 21
D3). The first tables come from `lingua-pack-update`, dispatched on the implementation's branch in
update mode for fr-es, as en-es's did; the pinned reduction follows on the same branch and must
reproduce them. That first update's tables are **the committed measurement** M6 speaks of (D8).

### D4 — The Spanish edition's notes and pre-pass, as en-es reads them

fr-es reads the section through the Spanish edition as en-es does (24b D1): `common.without_letter_senses`
(a `character` entry, a sense naming a letter), then `spanish.read_as_meanings` (the senses the
edition marks obsolete or outdated after the others of their entry, one typography), then fr-es's
own pass (D5), then the shared rules, whose cleaning runs the edition's notes (`ES`). The table
above measures each: 3 rows for the notes, 19 for the order, 3 expressions for the typography. They
are the edition's rules, applied unchanged: the French section is written as the English section is,
in fewer senses.

The order takes the edition's labels as written, as 24b's D5 does: « baiser » opens on « Coger
(sexualmente) » because the Spanish Wiktionary labels « Besar » outdated — modern French bears it
out, and the sample names it for the owner (Open Question 2). Nothing is left out by the order.

### D5 — fr-es's own pass over the French entries

`french_entries(src, dst)`, run after `read_as_meanings`, holds 24b's two pair rules, transposed:

- **A name's note does not gloss the common word spelled like it** (24b D3): a sense of a `name`
  entry whose headword opens on a capital, glossed « Apellido… », « Nombre de pila… », « Nombre
  personal… » or « Hipocorístico… », is left out when the word has an entry in lower case, not a
  proper noun's, holding a meaning; an entry left with no sense goes. 17 / 12 rows (« jean »,
  « pierre », « rose », « romain »). A name's own row keeps its notes: 313 / 167 rows are glossed by
  them alone (« françois » « Nombre de pila de varón, equivalente del español Francisco », « marie »),
  and tell the reader what the capitalised token is.
- **Possessives and demonstratives are determiners** (24b D4): an `adj` entry the edition tags
  `possessive` or `demonstrative` (`mon`, `ton`, `ta`, `notre`, `ce`) reads as `det`, and so does an
  `adj` entry whose every sense is a form of such a word (`mes`, « Forma del masculino plural de
  mon »): without that clause `mes` (rank 98) lost its gloss, the form borrowing in its own part of
  speech only. 7 / 7 rows, their runs alone. The edition heads `son` and `leur` « pronombre
  posesivo », kept as written (*Known data defects*).

*Rejected — moving en-es's pair rules into the Spanish edition.* They are not English-specific, and
one module would hold them for both pairs; but editing `reduce-en-es.py` and `reduce_edition_es.py`
re-pins en-es and moves its golden's pack line in a change about fr-es. They are written again in
`reduce-fr-es.py`, as en-es wrote es-en's letter rule again (« es-en's rule with the noun added »),
and their move is named as a follow-up (Open Question 6).

### D6 — The translation tables: letters, a word listed once, one typography

`read_translated` reads the two tables as en-es's reads its own (24b D7), where the data has the
defect:

- **A letter glosses no word.** A `character` entry, a one-letter word's noun entry and a one-letter
  word every translation of which is itself are left out of both tables, and a one-letter French key
  of the inverted table is dropped: `h` « H », `x` « X » and nine more letters ranked among French's
  2,300 commonest lemmas lose a gloss that was the letter. `à`, `y` and `ô`, words of one letter, keep
  the definitions the section gives them before any table is read.
- **A Spanish word listed once.** Read backwards, the Spanish Wiktionary lists a French word under
  each part of speech of the Spanish word (`este`, adjective and pronoun, both list `cet`): it is listed
  once, under the first of those parts of speech French's readings name (`tables/fr/grammar.tsv`,
  change 45), else under the first listed. 94 / 31 rows (« cet » « Este », « capital »); without
  French's readings, 30 rows head another part of speech (« entraîneur » an adjective, « condom »
  « Preservativo, profiláctico; Condón »).
- **One typography**: `spanish.typography` on each Spanish word of both tables (« Bromas aparte… »).

Measured and not ported: en-es's translators' notes and disused words. The French Wiktionary's
translators write about twenty notes in 78,472 Spanish translations (« embaucar (1) », « afanar (se) »,
« bola de fibras (de posidonia) »), none labelled disused, and none in a word a fr-es gloss keeps.

The direct table's parts of speech are the French word's own: a Spanish word it lists under two of
them stays under both, as en-es keeps « israeli » « Israelí; Israelí » (24b D7). It is commoner here
— 421 / 129 rows, « parti » « Partido; Partido », « européen » « Europeo; Europeo » —, and the owner
may want it otherwise (Open Question 3).

### D7 — The studied word is no definition

The Spanish Wiktionary defines the conjunction `et`, French's third commonest word, « Et. » — the
French word, not a Spanish one; the French Wiktionary translates it « y », « e ». M5 forbids a gloss
in the studied language. A definition whose every sense is the French headword itself, as written up
to case, yields to the French Wiktionary's Spanish translations of the word when they hold no word
spelled as the headword: `et` « Y, e ». Measured, 9 / 4 rows: `et`, `élite` « Élite » → « Elite »,
`troll` « Trol », `slip` « Calzoncillos, braguitas », `clochard` « Mendigo », `gourmet`
« Gastrónomo », `azimut` « Acimut », `yucca` « Yuca », `octogonal` « Octagonal ». A cognate the
French Wiktionary also translates as itself keeps its definition: 179 / 82 rows define the word by
itself as written, and the table lists that same word for all but these nine (« entre » « Entre »,
« venir » « Venir », « normal » « Normal »).

*Rejected — waiting for the Spanish Wiktionary to be corrected.* The page arrives with fr-es's next
update at best, and `et` is in every sentence; the rule fires on nothing once it is corrected.
*Rejected — a written gloss for `et`.* A gloss written here is the optional
`add-lingua-curated-word-glosses`, outside the counts; the French Wiktionary's translators wrote one.

### D8 — M6: the floor, and what happens below it, fixed before the measurement

**The value.** `FLOORS["fr-es"] = (81.4, 68.8, 54.5)`: the study's 83.4 / 70.8 / 56.5 % less two
points — the rule en-es's floor (change 22 D3) and fr-en's (change 48 D4) follow. The prototype
measures 83.2 / 70.8 / 56.8 %, 1.8 / 2.0 / 2.3 points above, and *The floor's effect* shows it
cannot be met without all three sources. The two points are the room a regeneration of the dumps
needs: the committed measurement reads the dumps of its dispatch day, not the prototype's.

**Where.** In `gloss_coverage.py`'s `FLOORS` and nowhere else, as en-es's (change 22 D3): the reduce
job runs `gloss_coverage.py --pair fr-es` with no `--floor`, and the Python tests hold the committed
tables to that entry, assert the job passes none, and pin the entry's value to the requirement's.

**When.** The owner settles the value and the consequence on this proposal's pull request (task 0.1);
the implementation writes them into `FLOORS` and the requirement **before** it dispatches the update
whose tables are committed (task 3.1). en-es's floor was set on its implementation's pull request,
after its tables were measured (its task 5.1); M6 rules that order out for fr-es.

**Below it.**
1. **At the committed measurement.** If the first update's tables measure under the floor at any of
   the three tops, they are not committed: this change does not merge, its pull request records the
   figures, and no package lists fr-es — change 52 ships French for English speakers alone (fr-en),
   and the site's coverage, the listings and the store pages (52, 53) name fr-es nowhere. fr-es is
   dispatched again at a later regeneration of kaikki's Spanish or French dump, against the same
   floor.
2. **After it is committed.** Every pull request that re-reduces fr-es — an update of its sources, a
   change of its rules or of the Spanish edition's, a change of French's studied tables (fr-en's
   update brings fr-es along: *A change to a studied language's tables reaches every pair of that
   language*) — fails the reduce job under the floor, naming the figure. Such a pull request does not
   merge until it measures at or above it, and readers keep the last committed fr-es.
3. **Never lowered after a measurement.** A lower floor is a decision of the owner recorded in the
   programme before the measurement it applies to, never in the pull request that measured under the
   old one. A source or a rule that raises coverage is a change of its own, proposed and measured as
   this one, under M5.

**Above it.** fr-es ships with change 52, and its coverage is published with the shipped pairs' —
`gloss_coverage.py --write`, the site's like-for-like table, beside es-fr's (M6's precedent) — the
moment `packs.json` lists it; before that it is shown nowhere but in its README and pull request.

*Rejected — es-fr's published figures as the floor* (es-en's rule): 4.4 / 6.4 / 6.9 points above
anything the three sources give, so fr-es would never ship, against M6.
*Rejected — the prototype's figures as the floor*: no room for a regeneration; a dispatch a tenth of
a point under would stop a pair that reads as the prototype does.
*Rejected — below the floor, ship with a warning*: the floor would decide nothing.

### D9 — The share of definitions, and the sample, shown

`reduce-fr-es.py` writes `measures.json` as en-es's does — among the glossed lemmas of the top
10,000, those from a definition, from the direct and from the inverted table, and the share from a
table —, which `split` files nowhere and `pack_report.py` prints beside the coverage. The share of
all glossed lemmas from a definition (the programme's « 24 % ») is printed with it. Both are in the
tables' README and the pull request, stored in no pack. A sample of 100 glosses of the top 10,000,
systematic in rank order, each marked by its source, goes in the pull request for the owner.

### D10 — The checks

- **The pipeline.** `lingua-pack-update.yml` offers `fr-es` (its `fr` concurrency group is change
  43's); the reduce job reduces it after fr-en and runs `gloss_coverage.py --pair fr-es`; the
  monthly dry run takes it through `pairs`. `packs.json`, the site and the listings are untouched.
- **The committed tables** (`crates/lingua-pack/tests/committed_tables.rs`): `tables/fr-es/` holds
  exactly `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md`;
  its pin's studied record names fr-en and `tables/fr/`'s six tables as committed; its release is
  `lingua-pack-sources-fr-es-…`; its notice credits both sides; its pack has its pin's sha256, is
  under 5 MiB, carries French's dictionary words (fr-en's), and is listed nowhere.
- **French's cross-native test** (`crates/lingua-wasm/tests/cross_native.rs`,
  `spec_scenario_french_through_another_native_language`): fr-en and fr-es built from the committed
  tables; their studied sections byte-equal (the tag pool up to its pinned prefix); every probe of
  the French invariance baseline answered through each, alike once glosses and senses are removed;
  a lemma fr-es glosses and fr-en does not (`quant`) is no dictionary word; `maison` reads « Casa ».
  The scenario's engine holds the one pack, so its native language is the pack's (`render_with`).
- **The row cut** (`apps/lingua-extension/test/row-gloss-tables.spec.ts`): `PAIRS` gains fr-es — no
  row empty, none ending on an opening mark (measured: none of 19,050).
- **The rule digests** (`test_reduce_editions.py`): an edit of `reduce_edition_es.py` moves en-es's
  and fr-es's digests and no other's; an edit of `reduce-en-es.py` moves en-es's alone, of
  `reduce-fr-es.py` fr-es's alone.

### D11 — What moves, and what cannot

- **`fr-en.golden` does not move.** fr-es is not in the French invariance baseline, whose engine is an
  English-native reader's (fr-en beside es-en); this change edits no French rule, no table of
  `tables/fr/` and no file of `crates/`'s sources. fr-es's own golden — the French scenario glossed in
  Spanish, rendered by the Spanish card — is change 51's, as en-es's was change 24's.
- **en-fr, es-fr, es-en, en-es and fr-en cannot move.** No file of their rule digests changes:
  `reduce_common.py` and the three `reduce_edition_*.py` are read, not edited, their reducers
  untouched; `pack_sources.py`, `gloss_coverage.py` and `build.sh` are in no digest (change 38). No
  table of `tables/en/`, `tables/es/`, `tables/fr/` or of their pairs moves; their packs rebuild to
  their pins' sha256, and the five goldens, the extension's snapshots (`selection-rows-fr.txt`,
  `word-card-es-en.txt`, `word-card-en-es.txt`) and the French baseline's `beside es-en` line pass
  without re-blessing. The gate is `git diff --stat origin/main --` over those paths, empty, and the
  reduce job reproducing every committed byte (task 5.1).

OpenSpec: two ADDED requirements in `lingua-data-packs` and one MODIFIED in `lingua-analysis` (*An
analysis does not depend on the native language*, held by no open change). `archiveAfter`: 38 (the
catalogue), 21 (a reader pair and its pin), 22 (en-es's floor, kept in one place), 24b (the Spanish
edition's rules), 39 (French as a studied language, its baseline), 43 (French's tables) and 48
(French's dictionary words, not yet on `main`: `openspec_archive_order.py` waits for it once it is).

### D12 — Order, and what later changes take from here

| Change | Relation |
|---|---|
| 43 forms tables | before (required): `tables/fr/`, French's 60,000 cap, the `fr` concurrency group |
| 48 fr-en | before (required): French's dictionary words (`lexical.tsv`), fr-en reduced before fr-es in every loop, the French baseline on the real pack the cross-native test compares |
| 45 grammar, 46 levels | before (planned): D6's readings and the pinned tag pool; the levels the pin records. Landed after, each re-records fr-es's pack in its own pull request (*A change to a studied language's tables reaches every pair of that language*) |
| 40, 41, 42, 44 | either side. 41 and 42 bump French's analyser version, which fr-es's manifest names: landed after, each re-reduces fr-es too, as Spanish's bumps re-reduce es-en; 44 keys whatever `mwe.tsv` holds |
| 50 marks | after or beside: fr-es's marks are the translation engine's (fr-en → en-es), not the glosses' |
| 51 word card | after: the Spanish card renders fr-es's glosses and runs, and pins fr-es's golden; it lists what reads wrong for a refinement before 52, as change 24 did for en-es |
| 52 enable | after: lists fr-es only if its committed tables stand at or above the floor (D8), publishes its coverage |
| 53 listings and site | after: fr-es's figures and claims, or none |

## Known data defects

What still reads wrong in fr-es, measured on the prototype, for the owner's review and a refinement
before change 52 if it is wanted (rows / top 10,000).

| Class | Rows | Examples | Where its fix belongs |
|---|---|---|---|
| A note before the meaning | 46 / 23 | « être » « (être + participio) Haber », « chien » « (Canis lupus familiaris) Perro », « faire » « (faire chaud, faire froid) hacer » | change 24's Q5, the Spanish edition |
| Labels not shown | register 136 / 79, obsolete or outdated 59 / 26, regional 31 / 18, rare 4 / 2 | « mec » « …; Proxeneta », « baiser » | 24b's Q1, the Spanish edition |
| A pointer the edition's wording misses | 13 / 11 | `mme` « Abreviatura de madame », `al` « Abreviatura de année-lumière », `du` « Contracción de la preposición de y el articulo le, del » (informative, M21's `du` whole) | the Spanish edition's `_FORM_OF` (re-pins en-es and fr-es) |
| A function word's description | 2 / 2 | `il` « …; Pronombre sujeto expletivo impersonal. (No tiene traducción al español. No existe en español.) », `de` « … (no existe en español) » | change 24's Q5 |
| A possessive headed as a pronoun | 2 / 2 | `son` « Su » PRON, `leur` « Suyo, suya » PRON | upstream, or a rule naming them |
| The direct table's word under two parts of speech | 421 / 129 | « parti » « Partido; Partido », « européen » | Open Question 3 |
| The inverted table's other-sense words | about a dozen of 265 / 265 | « us » « EEUU », « fr » « Imperial, calabaza », « rap » « Secuestro », « pa » « Autopiloto », « luc » « San Lucas » | 24b's Q4, Open Question 4 |
| A translation written in French | 8 / 2 | « arnaque » « Arnaque », « gâche » « Gâche » | upstream (the French Wiktionary's tables) |
| Upstream wording | single rows | « siège » « …; Local, stilla », « pieux » « …, pièsa », « modo » « Jur mödo, jur, mödo », « ol » « Elle », « chaussée » « Carreta, … » | upstream |
| « etc » without its period | 5 / 4 | « avec », « adresse », « cochon » | shared, every pair (24b D8) |
| A proper noun's run first | 596 / 353 rows hold one | « france » « Francia; Nombre de pila de mujer », « terre » | the case-aware card, shared |
| Words of no French dictionary glossed | 544 / 40 | `el` « Ella, ello o él », `for` « Fuero », `last` « Lastre », `okay` « Oquey, oqué » | fr-es's tables; no dictionary word, no vocabulary count |

## Risks / Trade-offs

- **[The committed measurement falls under the floor]** → D8 says what follows, before it is known: no
  commit, no listing, French for English speakers alone, a later dispatch against the same floor.
- **[A thin pair reads as a lesser product]** (risk 5) → the floor, the published coverage beside the
  other pairs', the share of definitions in the README; the sample read by the owner.
- **[A translation is a word, not a meaning]** → 59.8 % of the glossed top 10,000; still a Spanish word a
  person chose for that French word, grouped by part of speech like any gloss (es-fr's D3 risk).
- **[An edit of the Spanish edition now moves two pairs]** → said in D1 and tested (D10); a rule for
  one pair goes in its reducer.
- **[en-es's rules written twice]** → the follow-up of Open Question 6; until then each pair's tests pin
  its copy.
- **[The readings D6 reads arrive after this change]** → without `grammar.tsv` the inverted table's
  words take the first part of speech listed (30 rows differ); change 45 re-reduces fr-es in its
  pull request.
- **[kaikki's regeneration differs from the prototype's]** → the floor's two points, and the figures
  measured again on the committed tables (task 3.2).

## Migration Plan

Nothing to migrate: no reader holds a French pack. Rollback is a revert of `tables/fr-es/`, the
reducer and the registrations; the sources release stays, as every pair's does.

## Effort

2.5–4.5 ideal days, against the programme's 2.5–5: the reducer — D1–D7 — and its tests 1–1.5; the
registration (`DUMPS`, the workflows, `FLOORS` and its tests) 0.5–0.75; the dispatch, the pinned
reduction, the pin, the README and `SOURCES.md` 0.5–0.75; the committed-tables, cross-native and row
checks 0.5–1; the sample, the spec and the programme 0.25–0.5.

## Open Questions

For the owner:
1. **The floor and its consequence** (D8): 81.4 / 68.8 / 54.5 %, and *Below it* as written. Settled
   on this proposal's pull request, before the implementation dispatches the update it measures — the
   one question that blocks task 3.1.
2. **Labels and the order of a row** (24b's Q1 and Q2, now fr-es's too through the Spanish edition):
   « baiser » opening on « Coger (sexualmente) », its « Besar » labelled outdated; 136 / 79 rows show
   a register's sense without its label, 59 / 26 an obsolete one, 31 / 18 a regional one.
3. **The direct table's word under two parts of speech** (D6): 421 / 129 rows (« parti » « Partido;
   Partido »). Kept, as en-es keeps « israeli », or listed once in fr-es under the first part of
   speech French's readings name, a rule of fr-es's reducer?
4. **The inverted table's other-sense words** (24b's Q4): about a dozen of the 265 rows of the top
   10,000 it glosses read wrong (« us » « EEUU », « rap » « Secuestro »). Accepted, or a rule?
5. **A sentence true of its time**: 24b's requirement *en-es's glosses read an English word's meanings,
   not its page's notes* says its edition rules are read by en-es alone and re-pin en-es alone; from
   this change on, en-es and fr-es read them. Best amended when 24b is archived, as change 48 asks for
   23b's.
6. **en-es's pair rules in the Spanish edition**: D5's and D6's transposed rules (names, determiners,
   letters, a word listed once) moved into `reduce_edition_es.py` by a follow-up that re-pins en-es and
   fr-es together, outside the 57 — or kept in each reducer.
7. **The share of definitions on the site** (change 53): beside the coverage for every pair, or in the
   README alone as en-es's share is.
