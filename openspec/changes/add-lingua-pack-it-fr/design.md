# Design

## Context

See proposal.md — Why. Where things stand:

- `reduce-it-fr.py` (change 9, amended) writes `tables/it/` — forms, ranks, and, with changes 10,
  11, 13 and 14, the spelling and acronym rows, the readings and the levels — from the English
  Wiktionary's Italian section; `tables/it-fr/gloss.tsv` is empty and `tables/it/lexical.tsv` holds
  `NATO` and `USA` (change 11). `studied.json` names it-fr Italian's reference pair.
- es-fr is the precedent of a studied language glossed in French: `reduce-es-fr.py` loads
  `reduce_edition_fr` (the French Wiktionary's rules, `FR`) and reads `kaikki-fr-Espagnol.jsonl`
  (the section's definitions), `kaikki-fr-traductions.jsonl` (the French entries' translation tables,
  read backwards) and `kaikki-es-traductions.jsonl` (the Spanish Wiktionary's French translations,
  the direct fallback) through `reduce_common.native_tables`; `dictionary_words(glosses, runs)` in
  `reduce_edition_fr` gives the language's dictionary words (change 24c). fr-en is the precedent of a
  reference pair committing its glosses beside its studied tables, the French baseline handed to the
  committed tables in the same pull request; fr-es of a coverage floor fixed before the first
  committed measurement (M6, `FLOORS`).
- `pack_sources.py`: `EDITIONS` lists the files each Wiktionary dump derives (`("entries", lang)` or
  `("translations", from, to)`), `DUMPS[pair]` the files a pair reads by edition, `FLOORS[pair]` the
  coverage floors.
- Change 11's D3 (the acronym rows glossed by the pair's glosses of their expansions, else synonyms,
  else none; `acronym_gloss`), its D5 (`lexical.tsv` = dictionary words ∪ rows; `check_studied`
  exempting the rows); change 12's expression keys and its single words the pre-pass splits; change
  14's D4 (`words` passed to `estimated_levels`); change 5's D6 (the baseline handed to the tables by
  the pair that commits the glosses).
- Measured (proposal, Why) on kaikki's extract of the section (2026-10-22) with scratch scripts: the
  entries, the glossed headwords by part of speech, the expressions, the 43 senses misread as links,
  the 243 glosses repeating the headword, and the coverage on a prototype ranking (wordfreq's Italian
  order, a word skipped when the English section gives it as an inflection of another, the `di`
  family kept as lemmas and the other articulated prepositions dropped, as change 9 ranks them).

## Goals / Non-Goals

**Goals:**

- Every ranked Italian lemma the French Wiktionary glosses carries its French gloss, sense runs and
  expressions; the dictionary words and the levels follow.
- The coverage is measured against a floor fixed before the first commit, and shown.
- The Italian baseline runs on the real tables from this change on.

**Non-Goals:**

- The defects the dogfood and the review find in the glosses (change 17, as 48b for French).
- Listing it-fr in a package (change 20).
- it-en and it-es (stage 2).

## Decisions

### D1 — it-fr's native side, in the reducer that writes Italian's studied side

`reduce-it-fr.py` writes `tables/it-fr/` in the same run as `tables/it/`, as fr-en's reducer writes
French's two folders; `pack_sources.py split` files each table into its folder (*What a pack studies,
whatever it glosses*). The rules shared with es-fr are the shared modules', loaded as es-fr loads
them; nothing of `reduce_common.py` or `reduce_edition_fr.py` is edited.

### D2 — Three sources, definitions first, in es-fr's order

As es-fr, en-es and fr-es read theirs (`sources = [(direct, …), (inverted, …)]`):
1. **The section's definitions** (`kaikki-fr-Italien.jsonl`): the French Wiktionary's own glosses of
   its Italian entries, through `FR`'s rules (notes, typography, the form-of patterns) — 68,985
   headwords glossed before the shared filters, after D4's and D5's pre-pass;
2. **The Italian Wiktionary's French translations** (`kaikki-it-traductions-fr.jsonl`), the direct
   fallback: an Italian entry listing French translations, in the table's order, at most three per
   part of speech — 10,086 Italian entries with 14,410 French translations;
3. **The French Wiktionary's translation tables, read backwards** (`kaikki-fr-traductions-it.jsonl`):
   a French entry listing an Italian translation glosses that Italian word with the French headword,
   commonest French word first, at most three per part of speech — 71,723 French entries listing
   62,689 Italian words.
A typographic apostrophe in a headword, or in an Italian word of either table, is read as `'`, as
fr-en's and fr-es's requirements say. Sense runs (`senses.tsv`) and the grouping of a gloss by part
of speech are the shared rules'; a noun's gender comes from change 13's readings, as French's does.
Measured (the reviewer's re-run with the real shared rules, `reduce_gloss` and `FR`, letters out, D4
on): 585 top-20,000 lemmas glossed by a fallback; the order moves 38 of them (`decade` « Décennie »
from the Italian table first, « Décade » read backwards), none in or out.

### D3 — The editions' files, one release of it-fr's

`EDITIONS["fr"]["files"]` gains `kaikki-fr-Italien.jsonl` (`("entries", "it")`, the section as the
pipeline derives it: 769,483,152 B, 1,199,070 entries) and `kaikki-fr-traductions-it.jsonl`
(`("translations", "fr", "it")`); a new edition `EDITIONS["it"]` —
`https://kaikki.org/itwiktionary/raw-wiktextract-data.jsonl.gz`, 43,849,715 B gzipped, regenerated
2026-10-21; its translations sit on the entry, with `lang_code` and `word`, the shape `derive`
reads — derives `kaikki-it-traductions-fr.jsonl` (`("translations", "it", "fr")`), and will derive
the Italian Wiktionary's English section en-it reads (stage 3) and its French section fr-it reads
(stage 4). `DUMPS["it-fr"]` becomes `{"en": ("kaikki-Italian.jsonl",), "fr": ("kaikki-fr-Italien.jsonl",
"kaikki-fr-traductions-it.jsonl"), "it": ("kaikki-it-traductions-fr.jsonl",)}`. The update publishes
the derived files under `lingua-pack-sources-it-fr-<snapshot>`, as change 9's did; a re-reduction
fetches them, not the dumps. The cost, recorded: `read_edition` derives the edition's whole catalogue,
so every French-dump derivation (en-fr's, es-fr's, fr-en's and fr-es's updates, the monthly dry run)
writes about 0.77 GB more for the Italian section.

### D4 — A gloss read as a link is a gloss

kaikki files some French definitions as links: the gloss « Du. » of `del` becomes a form of « Du ».
A sense tagged `form-of` is a gloss when every target is a word no Italian entry of the section heads
— by its exact spelling, whatever its case — and the French edition's form-of pattern (`FR.form_of`,
`_FORM_OF` in `reduce_edition_fr.py`: « Pluriel de… », « Première personne… ») does not read its text
as an inflection; `_is_form_of` is not that test (it is true of every sense tagged form-of). Measured
with the exact-spelling test: 34 senses with a capitalised target (`del` « Du. … », `al` « Au. »,
`ad` « À, dans sa forme euphonique », `c’è` « Il y a », `coloro` « Celles et ceux », `accusatore`,
`motociclista`) — 43 without the text test, which keeps « Pluriel de storia » (target « Pluriel »)
a link — and the lowercase targets among 51 senses whose text the pattern does not read as an
inflection (`delle`, `degli`, `dello`, `li` « …: les »), counted at implementation. The lemmas of
change 9's tables gain a gloss (`del`, `degli`, `delle`, `dello`, `coloro`, `accusatore`); `c’è`
glosses the expression change 12 keys `ci essere`; `al`, no word of the tables, glosses nothing.
`della` is not caught — its sense « Article contracté souvent traduit par De la. » links the Italian
`la` — and takes the translation tables' gloss. `del`'s gloss is « Du. » with its usage note; the
hamlet `Del`'s proper noun is kept off it by D5's rule.

### D5 — Names and acronyms keep off common words; dictionary words, the acronym rows, the levels

**A pre-pass of it-fr's, as fr-es's (24b's D3).** The shared `reduce_gloss` merges a capitalised
entry into the lowercase lemma, and the archived *An acronym does not gloss the word it is spelled
like* holds only where the lowercase entry has a sense of its own. Measured on change-9-like ranks:
102 top-20,000 lemmas (36 of the 5,000) are glossed by a proper noun or an acronym alone although the
section has a lowercase entry — `te` « Province de Térame », `li`, `so`, `migliore`, `soldi`,
`fatta`, `tratta`, `resta`, `valori` « Nom de famille ». So a name's or an acronym's sense glosses no
lemma the section gives a lowercase entry of its own, even one of form-of senses only; a name's own
row keeps its gloss. The lemmas left with no gloss then take the translation tables', or none.

**Dictionary words.** `tables/it/lexical.tsv` = `dictionary_words(glosses, runs)` ∪ the acronym rows
— *Italian's dictionary words leave its names out* (change 11), by change 24c's rule in
`reduce_edition_fr`, plus `NATO`, `USA`. The acronym rows are glossed by *Italian's acronyms are a
reviewed table, glossed through their entry's expansions or synonyms* (change 11's `acronym_gloss`):
`stati uniti`, a synonym of `USA`, is a proper-noun entry, which `reduce_expressions` and the
translation readers skip, so for an acronym row a synonym's gloss is read from its entry in the
section whatever its part of speech — `USA` « États-Unis… »; `NATO` has none. **The levels** go to
the dictionary words alone (*Italian's estimated levels*, change 14): `estimated_levels` receives
`words`, and `committed_tables.rs` gains « every Italian levelled lemma is a dictionary word ».
Measured at implementation and recorded: the names count (24c counted English's 3,046), the lemmas
of the 1,425 commonest that end with no gloss and so no level (66 before the pre-pass, measured).

### D6 — Expressions

`mwe.tsv` holds, as `native_tables` writes it for every pair: the section's glossed headwords with a
space, through `reduce_common.reduce_expressions` with an Italian token pattern and `FR`'s rules —
10,903 entries, 10,880 distinct headwords, 10,704 after the shared filters (9,677 noun entries,
many species names: « pinguino del fiordland ») —, a typographic apostrophe read as `'` (436 glossed
headwords write ’: `essere d'accordo`, `l'altro ieri`, `uomo d'affari`, lost to an ASCII pattern);
then those the translation tables gloss (`fallback_expressions`: 7,411, `meccanica quantica`, `un
po'` « Un peu »), as es-fr's D4 includes them; and the words without a space that change 6's
pre-pass splits into two tokens or more (`d'accordo`, `all'improvviso`, `senz'altro`, `c’è`), read by
it-fr's own reader as fr-en's change 48 reads French's. Each is keyed at build by change 12's rules;
a headword longer than seven tokens is left out there. Change 12's D10 figures (keys, names, the
window, the `di` retry) are re-measured on the real `mwe.tsv` (task 2.3).

### D7 — M6's rule for it-fr: the floor fixed before the measurement

The prototype in the proposal ranked words its own way: it dropped what the English section holds
only with a capital or not at all — which change 9's D6 ranks, as fr-en's `ranks_for` does — and kept
what change 9 drops (elision stems, elided compounds, clitic forms: 160 / 626 / 2,070 items in its
5,000 / 10,000 / 20,000), and counted names apart. Re-measured with the real shared rules
(`reduce_gloss`, `FR`, letters out, D4 on), on a change-9-like ranking: **87.6 / 79.8 / 70.6** from
the definitions alone, **89.3 / 82.3 / 73.5** with both fallbacks; if change 9's D5 gives every word
that is also another lemma's form (`porta`, `stato`, `fatto`) to the other lemma: 80.5 / 72.4 / 63.4,
82.8 / 75.1 / 66.4 with fallbacks. About 1.6 points at the 5,000 are common words glossed only by a
name, which D5's pre-pass removes.

So the floor is not fixed on the prototype. **Task 0.1** re-runs the study on change 9's committed
ranks, with both fallbacks and D4 and D5's pre-pass, and fixes `FLOORS["it-fr"]` in
`gloss_coverage.py` at those figures less two points, up or down — the rule en-es's, fr-en's and
fr-es's floors follow; the owner settles it before the first committed measurement (M6; fr-es's « a
lower floor is a decision of the owner, recorded before the measurement it applies to »), and the
figures are written into the requirement. Below the floor at the first committed measurement, the
tables are not committed and no package will list it-fr until a later regeneration of the dumps
measures at or above it; after the first commit, any pull request that re-reduces it-fr under it
fails (the floor step of `lingua-extension-check.yml`'s `reduce` job). The floor is never lowered
after it is measured against.

### D8 — Coverage committed and shown

`tables/it-fr/README.md` holds the coverage against the floor, the glossed lemmas by source
(definitions, translations read backwards, the Italian Wiktionary's translations), the share of
definitions, a sample of 100 glosses marked by source and the pack's size; `SOURCES.md` the sources
and their licences (both Wiktionaries CC BY-SA); the programme's *Measured figures* the committed
coverage. Nothing of it is stored in a pack.

### D9 — The Italian baseline on the committed tables

Change 5's requirement keeps the fixture « until the Italian tables are committed »; change 9
committed tables and kept the fixture (its D9), so the trigger is read as « until it-fr's glosses are
committed », this change. `crates/lingua-wasm/tests/italian_baseline.rs` names `PackSource::Tables` for it-fr; the golden is
re-blessed once in this change's pull request, which lists what moved (every gloss, expression and
level probe; the `analyse` probes where the real forms differ from the fixture's); the fixture stays
for the tests that build it (changes 6, 7, 10, 12's unit fixtures). The engine still starts on the
real en-fr pack beside it.

### D10 — The checks

- `lingua-extension-check.yml`'s `reduce` job reduces it-fr after its studied side, and its floor step
  (« Gloss coverage of … against their floors ») runs `gloss_coverage.py --pair it-fr`;
  `test_gloss_coverage.py` holds the committed tables to `FLOORS["it-fr"]`, and a test holds `FLOORS`
  to the figures the requirement states.
- `committed_tables.rs`: the it-fr pack built from the committed tables to its pin, whole and in
  parts (change 4); `tables/it/lexical.tsv` held to it-fr's glossed lemmas less the names plus the
  rows (change 11's `check_studied`); every levelled lemma a dictionary word; the ranked-gloss and
  « every dictionary word is ranked » checks exempting the acronym rows.
- `test_reduce_it_fr.py`: each source's order and the three-word cap; D4's rule, its exception and
  the exact-spelling test; D5's pre-pass; the dictionary words, the acronym gloss through a
  proper-noun synonym; an expression with a typographic apostrophe, one from a table, one without a
  space.
- The six other pairs: tables, pins, packs and goldens byte for byte.

`reduce_edition_fr.py`'s docstring (« en-fr and es-fr, the only pairs that load it ») becomes stale
and is not edited: editing it would move en-fr's and es-fr's rule digests and pins.

## Known data defects

For change 17 (`refine-lingua-it-fr-glosses`), as 48b for French: what the review of the committed
glosses and the sample finds; already seen — the species names among the expressions; English loans
the section does not gloss (`web`, `team`); `della` linked to `la`; the usage notes kept in
`del`'s gloss; the lemmas left without a gloss by D5's pre-pass, and so without a level; `marco`
glossed « Marc » alone (a first name), so no dictionary word.

## Risks / Trade-offs

- [The fallbacks' quality] → definitions first; at most three translation words per part of speech;
  the sample marked by source shows them.
- [A floor fixed on a prototype] → it is not: task 0.1 re-runs the study and sets it, up or down,
  before the first committed measurement; never lowered after it.
- [Expressions dominated by species names] → kept as the section writes them; the filters and change
  17 read them.
- [The golden moves widely] → the hand-over is change 5's scenario; the pull request lists what moved.

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr until change 20. The tables arrive
by the update workflow's pull request; rollback is a revert with the golden's.

## Effort

3.5–5 ideal days, against the programme's 3–5: the sources and the editions 0.5; the native side
through the shared rules, D4 and the fallbacks 1–1.5; dictionary words, acronym glosses, levels
0.5; expressions 0.5; the floor, the coverage and the README 0.5; the baseline's hand-over and the
checks 0.5–1.
