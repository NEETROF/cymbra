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

### D2 — Three sources, definitions first

1. **The section's definitions** (`kaikki-fr-Italien.jsonl`): the French Wiktionary's own glosses of
   its Italian entries, through `FR`'s rules (notes, typography, the form-of patterns) — 68,985
   headwords glossed before the shared filters.
2. **The French Wiktionary's translation tables, read backwards** (`kaikki-fr-traductions-it.jsonl`):
   a French entry listing an Italian translation glosses that Italian word with the French headword,
   commonest French word first, at most three per part of speech — for what the definitions leave out.
3. **The Italian Wiktionary's French translations** (`kaikki-it-traductions-fr.jsonl`): an Italian
   entry listing French translations, the direct fallback, at most three per part of speech.

Measured at the prototype: the definitions alone give 87.7 / 81.8 / 71.9 %; the two fallbacks are
measured at task 0.1 and in the update, as fr-es measured its three. Sense runs (`senses.tsv`) and
the grouping of a gloss by part of speech are the shared rules'; a noun's gender comes from change
13's readings, as French's does.

### D3 — The editions' files, one release of it-fr's

`EDITIONS["fr"]["files"]` gains `kaikki-fr-Italien.jsonl` (`("entries", "it")`) and
`kaikki-fr-traductions-it.jsonl` (`("translations", "fr", "it")`); a new edition `EDITIONS["it"]` —
`https://kaikki.org/itwiktionary/raw-wiktextract-data.jsonl.gz`, 43,849,715 B gzipped, regenerated
2026-10-21 — derives `kaikki-it-traductions-fr.jsonl` (`("translations", "it", "fr")`), and will
derive the Italian section fr-it needs (stage 3). `DUMPS["it-fr"]` becomes `{"en":
("kaikki-Italian.jsonl",), "fr": ("kaikki-fr-Italien.jsonl", "kaikki-fr-traductions-it.jsonl"), "it":
("kaikki-it-traductions-fr.jsonl",)}`. The update publishes the derived files under
`lingua-pack-sources-it-fr-<snapshot>`, as change 9's did; a re-reduction fetches them, not the dumps.

### D4 — A gloss read as a link is a gloss

kaikki files some French definitions as links: the gloss « Du. » of `del` becomes a form of « Du ».
A sense tagged `form-of` is a gloss when every target is a capitalised word no Italian entry of the
section heads and the French edition's form-of patterns (`_is_form_of`, « Pluriel de… »,
« Première personne… ») do not read its text as an inflection. Measured: 43 such senses, among them
`le` « Les », `al` « Au », `ad` « À, dans sa forme euphonique », `del` « Du », `sul` « Sur le »,
`c’è` « Il y a », `coloro` « Ceux; Celles », `altri` « Autre… », `accusatore` « Accusateur »,
`motociclista` « Motocycliste ». The lemmas of change 9's tables among them gain a gloss (`del`,
`della` « De la », `coloro`, `accusatore`); `c’è` glosses the expression change 12 keys `ci essere`;
`al`, `sul`, `le`, no words of the tables or forms of another, gloss nothing. A few targets are a
parse's word (« Pluriel » of « Pluriel de storia ») that the form-of patterns keep as links.

### D5 — Italian's dictionary words, the acronym rows, the levels

`tables/it/lexical.tsv` = `dictionary_words(glosses, runs)` ∪ the acronym rows — it-fr's glossed
lemmas less those every sense run of which is `PROPN`, by change 24c's rule in `reduce_edition_fr`,
plus `NATO`, `USA` (change 11's D5). The acronym rows are glossed by `acronym_gloss` (change 11's D3):
`USA` from it-fr's glosses of `stati uniti` and `america`, `NATO` none. The levels are narrowed to
the dictionary words: `estimated_levels` receives `words` (change 14's D4), and `committed_tables.rs`
gains « every Italian levelled lemma is a dictionary word ». The names counted, as 24c counted
English's 3,046 and Spanish's.

### D6 — Expressions

`mwe.tsv` holds the section's glossed headwords with a space through `reduce_common.reduce_expressions`
with an Italian token pattern (change 9's letter class) and `FR`'s rules — 10,903 headwords with a
space are glossed before the filters (9,677 nouns, many of them species names: « pinguino del
fiordland »), the counts after them recorded —, and the words without a space that change 6's
pre-pass splits into two tokens or more (`d'accordo`, `all'improvviso`, `senz'altro`, `c’è`), read by
it-fr's own reader as fr-en's change 48 reads French's, since `reduce_expressions` takes only
headwords with a space. Each is keyed at build by change 12's rules; a headword longer than seven
tokens is left out there.

### D7 — M6's rule for it-fr: the floor fixed before the measurement

`FLOORS["it-fr"] = (85.7, 79.8, 69.9)`: the prototype's 87.7 / 81.8 / 71.9 less two points, the rule
en-es's, fr-en's and fr-es's floors follow. Task 0.1 re-runs the prototype on change 9's committed
ranks with the two fallbacks, before the dispatch: the floor is then those figures less two points
if higher, never lower — the owner settles it on this proposal's pull request or at task 0.1. Below
the floor at the first committed measurement, the tables are not committed and no package will list
it-fr until a later regeneration of the dumps measures at or above it; after the first commit, any
pull request that re-reduces it-fr under it fails (the reduce job). The floor is never lowered.

### D8 — Coverage committed and shown

`tables/it-fr/README.md` holds the coverage against the floor, the glossed lemmas by source
(definitions, translations read backwards, the Italian Wiktionary's translations), the share of
definitions, a sample of 100 glosses marked by source and the pack's size; `SOURCES.md` the sources
and their licences (both Wiktionaries CC BY-SA); the programme's *Measured figures* the committed
coverage. Nothing of it is stored in a pack.

### D9 — The Italian baseline on the committed tables

`crates/lingua-wasm/tests/italian_baseline.rs` names `PackSource::Tables` for it-fr; the golden is
re-blessed once in this change's pull request, which lists what moved (every gloss, expression and
level probe; the `analyse` probes where the real forms differ from the fixture's); the fixture stays
for the tests that build it (changes 6, 7, 10, 12's unit fixtures). The engine still starts on the
real en-fr pack beside it.

### D10 — The checks

- The reduce job reduces it-fr after its studied side and holds it to `FLOORS["it-fr"]`.
- `committed_tables.rs`: the it-fr pack built from the committed tables to its pin, whole and in
  parts (change 4); `tables/it/lexical.tsv` held to it-fr's glossed lemmas less the names plus the
  rows (change 11's `check_studied`); every levelled lemma a dictionary word; the ranked-gloss and
  « every dictionary word is ranked » checks exempting the acronym rows.
- `test_reduce_it_fr.py`: each source's order, D4's rule and its exception, the dictionary words, the
  acronym gloss, an expression without a space.
- The six other pairs: tables, pins, packs and goldens byte for byte.

## Known data defects

For change 17 (`refine-lingua-it-fr-glosses`), as 48b for French: what the review of the committed
glosses and the sample finds; already seen — the species names among the expressions, English loans
the section does not gloss (`web`, `team`), the parse's links of D4 kept as forms.

## Risks / Trade-offs

- [The fallbacks' quality] → definitions first; at most three translation words per part of speech;
  the sample marked by source shows them.
- [A floor fixed on a prototype] → it may only rise at task 0.1; never lowered.
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
