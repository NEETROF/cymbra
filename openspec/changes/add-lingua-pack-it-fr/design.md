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
  `("translations", from, to)`), `DUMPS[pair]` the files a pair reads by edition; `gloss_coverage.py`:
  `FLOORS[pair]`, the coverage floors.
- Change 11's D3 (the acronym rows glossed by the pair's glosses of their expansions, else synonyms,
  else none; `acronym_gloss`), its D5 (`lexical.tsv` = dictionary words ∪ rows; `check_studied`
  exempting the rows); change 12's expression keys and its single words the pre-pass splits; change
  14's D4 (`words` passed to `estimated_levels`); change 5's D6 (the baseline handed to the tables by
  the pair that commits the glosses).
- Measured (proposal, Why) on kaikki's extract of the section (2026-10-22) with scratch scripts, then
  re-measured by the review with the real shared rules: the entries, the glossed headwords by part of
  speech, the expressions, the senses read as links (D4), the 243 glosses repeating the headword, and
  the coverage on a change-9-like ranking (D7).

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
   headwords glossed before the shared filters and before any pre-pass;
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
on): 585 top-20,000 lemmas glossed by a fallback; the order moves 38 of them (`app` « Appli » from the
Italian table first, « Application » read backwards), none in or out.

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
— by its exact spelling, whatever its case; an entry heads its word only by a meaning, a sense that
is no pointer (the review of the implementation: `toi`, whose entry is *togliere*'s form alone, heads
no word, so `te`'s « Toi : pronom disjoint… » glosses `te`) — and the French edition's form-of pattern (`FR.form_of`,
`_FORM_OF` in `reduce_edition_fr.py`: « Pluriel de… », « Première personne… ») does not read its text
as an inflection; `_is_form_of` is not that test (it is true of every sense tagged form-of). Measured
with the exact-spelling test: 34 senses with a capitalised target (`del` « Du. … », `al` « Au. »,
`ad` « À, dans sa forme euphonique », `c’è` « Il y a », `coloro` « Celles et ceux », `accusatore`,
`motociclista`) — 43 without the text test, which keeps « Pluriel de storia » (target « Pluriel »)
a link — and, among 51 senses with a lowercase target whose text the pattern does not read as an
inflection, those of a headword with no other glossed sense once D5 has cleared the name notes and acronyms (`degli`,
`delle`, and `li` « Accusatif masculin de loro : les », whose province `LI` would otherwise count),
counted at implementation. Without that condition « Article contracté souvent traduit par … » would join
`pelle` (skin), `collo` (neck) and `colle` (hill), and `dello` a near-duplicate of its own gloss.
The lemmas of change 9's tables gain a gloss (`del`, `degli`, `delle`, `coloro`, `accusatore`); `c’è`
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
`fatta`, `tratta`, `resta`, `valori` « Nom de famille ». So an acronym's sense, and a proper noun's
sense that only says the word is a surname, a first name or a hamlet (« Nom de famille », « Prénom… »,
« Diminutif de… », « Hameau de… »), gloss no lemma the section gives a lowercase entry of its own, even
one of form-of senses only, as fr-es's pass keeps a name's note off `pierre` (24b's D3); any other
proper noun's sense stays beside the lemma's own, and a name's own row keeps its gloss. The lemmas
left with no gloss then take the translation tables', or none.

Not every proper noun's sense: measured on the change-9-like ranking (the amendment's review), that
wording took the name from 1,044 of the 1,051 rows of the 20,000 that mix a name with a common sense
— 827 of them only noise (640 « Nom de famille », 153 hamlets, 187 places), but with them 59 names a
reader wants: `terra` « La Terre, planète tellurique », `luna` « Lune, satellite de la Terre »,
`cristo` « Christ », `madonna` « Notre-Dame », the zodiac's signs, the centuries (`novecento` « XXᵉ
siècle »), `argentina` « Argentine », Vénus, Saturne, Homère — and it left eight name-alone rows with
nothing (`vittorio` « Victor », `como` « Côme », `nasa` « NASA »). The notes-and-acronyms rule keeps
all 59; of the 101 rows glossed by a name alone (`li` is glossed by D4), 58 gain a real gloss (`soldi`
« Sou; Solde du soldat; Argent… », `fatta`, `so`, `migliore`, `bianca`), 26 keep a name and 17 end
with none; coverage moves by at most 23 rows of the 20,000. `roma`, `italia`, `milano`, `marco` and
`francesco` have no lowercase entry in this section and move under no wording.

**Dictionary words.** `tables/it/lexical.tsv` = `dictionary_words(glosses, runs)` ∪ the acronym rows
— *Italian's dictionary words leave its names out* (change 11), by change 24c's rule in
`reduce_edition_fr`, plus `NATO`, `USA`. The acronym rows are glossed by *Italian's acronyms are a
reviewed table, glossed through their entry's expansions or synonyms* (change 11's `acronym_gloss`):
`stati uniti`, a synonym of `USA`, is a proper-noun entry, which `reduce_expressions` and the
translation readers skip, so for an acronym row a synonym's gloss is read from its entry in the
section whatever its part of speech — `USA` joins `america`'s first sense « Amérique (continent) »
and `stati uniti`'s « États-Unis », in the row's order, as change 11 requires; `NATO` has none. **The levels** go to
the dictionary words alone (*Italian's estimated levels*, change 14): `estimated_levels` receives
`words`, and `committed_tables.rs` gains « every Italian levelled lemma is a dictionary word ».
Change 14 levelled names through a sense of another part of speech, from a lowercase or a capitalised
entry (`tables/it-fr/README.md`, *The levels*, *Known weaknesses*: `roma`, `marco`, `napoli`,
`francesco`, `maria`, `gesù`, `adam`, `palestina`, `reich`, the initialisms `pc`, `cd`, `ps`, `tar`…;
59 levelled lemmas take their only senses that are no name's from a capitalised entry): this change's
pull request confirms which of them drop out once the levels go to the dictionary words.
Measured at implementation and recorded: the names count (24c counted English's 3,046), the lemmas
of the 1,425 commonest that end as no dictionary word and so with no level (measured with D4, D5 and
the fallbacks: 52 — 42 unglossed, 10 glossed by a name alone).

### D6 — Expressions

`mwe.tsv` holds, as `native_tables` writes it for every pair: the section's glossed headwords with a
space, through `reduce_common.reduce_expressions` with an Italian token pattern and `FR`'s rules —
10,903 entries, 10,880 distinct headwords, 10,704 after the shared filters (9,677 noun entries,
many species names: « pinguino del fiordland ») —, a typographic apostrophe read as `'` (436 glossed
headwords write ’: `essere d'accordo`, `l'altro ieri`, `uomo d'affari`, lost to an ASCII pattern);
then those the translation tables gloss (`fallback_expressions`: 7,411, `meccanica quantica`, `un
po'` « Un peu »), as es-fr's D4 includes them; and the words without a space that change 6's
pre-pass splits into two tokens or more, beginning with an elided piece — an articulated preposition
written whole excepted (`alla`, `nel`: its pieces are a preposition and an article, and offered alone
it would answer every « alla » with « à la », change 12's D2) —, from the section (`all'improvviso`,
`c’è`) and from the
translation tables (`d'accordo` « d’accord », `senz'altro`, which `fallback_expressions` skips for
having no space), read by it-fr's own reader as fr-en's translated expressions read a word the
tokenisation splits alone. Each is keyed at build by change 12's rules;
a headword longer than seven tokens is left out there. Change 12's D10 figures (keys, names, the
window, the `di` retry) are re-measured on the real `mwe.tsv` (task 2.3).

### D7 — M6's rule for it-fr: the floor fixed before the measurement

The prototype in the proposal ranked words its own way: it dropped what the English section holds
only with a capital or not at all — which change 9's D6 ranks, as fr-en's `ranks_for` does — and kept
what change 9 drops (elision stems, elided compounds, clitic forms: 160 / 626 / 2,070 items in its
5,000 / 10,000 / 20,000), and counted names apart. Re-measured with the real shared rules
(`reduce_gloss`, `FR`, letters out, D4 and D5 on), on a change-9-like ranking: **87.5 / 79.6 /
70.5** from the definitions alone, **89.2 / 82.2 / 73.4** with both fallbacks; if change 9's D5 gives
every word that is also another lemma's form (`porta`, `stato`, `fatto`) to the other lemma: 80.3 /
72.3 / 63.3, 82.7 / 75.0 / 66.3 with fallbacks. D5 moves them by 0.2 points at most: most lemmas it
clears regain a real gloss (`soldi` « Sou; … », `so` « Savoir », `fatta` « Fait… », `bianca`
« Blanc », `migliore` « Meilleur » from the table).

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
glossed « Marc » alone (a first name), so no dictionary word; `te`, cleared of its province by D5,
glossed « Te » read backwards from the French entry « te », a letter's name (fr-es's rule that a
letter glosses no word in either table would stop it) — fixed at implementation, after the review:
an entry heads its word only by a meaning, so `te` reads « Toi : pronom disjoint… » (D4); 27
top-20,000 fallback glosses repeating a word across parts of speech (« Décade; Décade », `beige`),
which fr-es's listed-once rule fixes — fixed at implementation for a sense repeated up to case
(*The review's fixes*); 327 rows of the 20,000 still holding a name beside their common senses, about
268 of them noise (`vita` and `ora` « …, commune d'Italie » — fixed at implementation, a commune's note
being a name's —, `era` « Héra », `qui` and `qua` « Riri » and « Loulou »,
`regina` « Régine », constellations) — the measured candidate fix, a name spelled exactly as the word
carrying no translation, would also lose `riga`, `congo`, `risorgimento`, `pinocchio`, `panama`,
`trani` and `lugo`; `renato` and `nasa` left with no gloss (`nasa` better as a row of change 11's
acronym table).

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

## Implementation — what ships, what departs, and the predicted figures (2026-10-24)

Implemented on `main` after changes 5 to 15 and 11a (analyser `1.4.0`) and change 12. The pull request
holds the reducer, the sources' registration, the floor and the checks; **it-fr's tables are not in
it**: they arrive with the owner's dispatch (task 3.1, mode `update`, `keep: en`), which publishes a new
sources release. Every figure below is therefore **predicted** — reduced locally on the proposal-time
copies of the French Wiktionary's Italian section (kaikki's own extract, 1,047 MB, laid out
differently from the 769,483,152 B the dump derives) and of the two translation files — and
re-measured after the dispatch. The review of the implementation reduced the same rules from the
dumps served on 2026-10-24 (the French one regenerated 2026-10-23) and obtained the same tables byte
for byte, so a dispatch on those dumps writes what is predicted here.

**Committed (2026-10-24).** The owner settled the floor at 87.2 / 79.8 / 71.3 and approved the update:
`lingua-pack-update` run 40130354122 (mode `update`, `keep: en`), the release
`lingua-pack-sources-it-fr-2026.10.24`, the commit `40aafd84f`. Its tables are the prediction byte for
byte — the pack `0e6795d9…` (2,645,342 B), its base `f8c78228…`, its layer `0927f745…`; coverage
89.2 / 81.8 / 73.3 —, so every figure below, measured again on the committed tables (change 12's
D10 figures, the names count, the golden's diff), stands as measured. UD Italian-PUD reads 99.26 /
91.31 / 99.90 on them, as on the tables before: the glosses move no lemma.

**Main reproduced first.** Reduced again locally from it-fr's pinned release assets (Python 3.12,
`requirements-reduce.txt`), `main`'s reducer gives `main`'s committed `tables/it/` and `tables/it-fr/`
byte for byte. This change's reducer, over the same English section and the local French and Italian
files, leaves `tables/it/{forms,freq,grammar,tags}.tsv` byte for byte and moves `lexical.tsv` and
`level.tsv` alone; no table, pin or golden of the six other pairs moves.

### What departs from D1–D10, or settles what they leave open

- **D1, step by step.** The native side is called as fr-es's is (`native_side`): the shared
  `reduce_gloss`, `reduce_expressions` and `fallback_glosses` one source at a time, so that the lemmas
  each source glosses are kept for the report; a test holds its tables equal to
  `reduce_common.native_tables`'. Neither shared module is edited.
- **D2, Italian for the shared rules.** `IT = common.Studied(...)`: change 9's token pattern, and as
  coordinators the conjunctions UD Italian-MarkIT's training and development sections tag CCONJ at
  least twice, CCONJ the commonest of their tags (`e`, `ed`, `o`, `ma`, `sia`, `oppure`, `né`, `cioè`,
  `ovvero`, `ossia`, `nonché`, `bensì`) — a `conj` entry's run is CCONJ when its word is one. The
  French tables read backwards are ordered by wordfreq `fr` (`reduce(native_frequency=…)`), as es-fr's.
- **D4, as implemented** (`section_as_read`, `SectionReading`): letters are left out first, as es-fr's
  are; « no other glossed sense » counts a meaning of the word's own, a proper noun's sense D5 keeps
  and a link to a capitalised word D4 reads as a gloss. Measured: **34** links to a capitalised word
  read as glosses (D4's 34), **42** to a lowercase word, **9** lowercase links left forms beside
  another gloss (42 + 9 = D4's 51). On change 9's committed ranks six lemmas gain a gloss (`ad` « À,
  dans sa forme euphonique », `degli`, `coloro` « Celles et ceux », `der`, `ar`, `pegli`) and six
  change it (`del` « Du. … » where the tables gave one, `delle`, `cogli`, `motociclista`, `accusatore`,
  `caco`); coverage +0.1 point. **Since the review, an entry heads its word only by a meaning**: `toi`,
  whose entry is *togliere*'s form alone, heads none, so `te`'s « Toi : pronom disjoint… » glosses `te`
  (it was « Te », a letter's French name read backwards); and the pointer wordings `FR.form_of` misses
  — « Agglutination de/du… » (a clitic compound: 104,895 senses link the pronoun `gli`, whose entries
  are forms alone), « Forme pronominale de… », « Ancienne orthographe de… », « Seconde personne… » —
  are inflections too (`_POINTER_WORDING`): 33 capitalised and 36 lowercase links read as glosses, 11
  left forms. `li` is no ranked lemma of change 9's (a form of *lo*).
- **D5, as implemented.** « An acronym's sense » is a sense of an entry whose headword the shared
  rules read as an acronym (written in capitals, `TE`, `LI`, `SO`) or a sense opening « Abréviation
  de », « Sigle de », « Acronyme de » (`A` « Abréviation de Azione… », kept off `a`). A name's note is
  read after the headword repeated too (« Del, hameau de Saint-Denis », off `del`), and since the review
  a commune's note is one (« Ora, commune d’Italie… », « Commune, commune d’Italie… »). « A lowercase entry
  of its own » is an entry headed in lowercase that is no proper noun's, of whatever senses. Measured:
  3,884 name notes and 83 acronym senses set aside, 3,883 entries left with no sense (dropped). On the
  20,000 commonest: of the 1,058 rows mixing a name with a common sense, 324 still hold one (D5's 327);
  of the 2,039 rows glossed by a name alone without the rule — nearly all real names (`italia`, `roma`)
  —, 64 change: 49 gain a real gloss (`te`, `migliore`, `tratta`, `piani`, `fiori`), 1 keeps a name
  (`cassino`), 14 end with none (`terzi`, `renato`, `napolitano`, `baffi`, `bassano`, `coppe`, `ge`,
  `calvino`, `paro`, `sperduto`, `pilato`, `ventola`, `ghe`, `ze`); coverage moves by under 0.1 point.
  `soldi`, `so`, `fatta`, `bianca` and `li`, which the design names, are no ranked lemmas of change 9's
  committed tables (forms of *soldo*, *sapere*, *fare*, *bianco*, *lo*): they gloss nothing either way.
- **D5, the acronym rows.** `acronym_glosses` passes change 11's `acronym_gloss` the pair's glosses
  and expressions, a row word the pair does not gloss read from its entry in the section whatever its
  part of speech (`section_words`): `USA` « Amérique (continent); États-Unis », one `PROPN` run of 2;
  `NATO` none.
- **D6, which single words are expressions.** A word without a space is offered when it is an Italian
  token beginning with an elided piece (`_ELIDED_START`) and no form: `all'improvviso`, `c'è`,
  `d'accordo`, `senz'altro`, `mezz'ora`. An articulated preposition written whole (`alla`, `nel`),
  which the pre-pass splits too, is not: its pieces are a preposition and an article, and offered alone
  it would answer every « alla » with « à la » (change 12's D2). Predicted: **22** from the section and
  **53** from the tables (change 12 counted 77 such words in the English section).
- **D8, the report.** The reduction writes `measures.json` beside its tables — en-es's and fr-es's shape
  (`top`, `glossed`, `share`, `direct`, `inverted`), which `pack_report.py --measures` shows in the
  update's report, plus `coverage`, `by source` and `sample`: 100 glossed lemmas of the 20,000
  commonest drawn by a fixed seed (`SAMPLE_SEED`), each with its rank, source and gloss — and prints
  them. `pack_sources.py split` files it nowhere and no pack stores it. `tables/it-fr/README.md` and the
  programme's *Measured figures* record the dispatch's figures (task 3.1), not these.
- **D8, the credits.** The NOTICE adds the French Wiktionary's Italian section and translation tables
  and the Italian Wiktionary, with their licences' addresses; the manifest's kaikki licence names
  `enwiktionary, frwiktionary, itwiktionary`.
- **D9 and D10, after the dispatch.** Tasks 4.1 and 4.2 need the committed tables: done on a scratch
  copy of the predicted tables and reverted (below); they land when the dispatch's branch is
  fast-forwarded and the golden re-blessed. Until then the Italian baseline keeps its fixture, and
  `test_gloss_coverage.py` holds the committed tables to the floor once `gloss.tsv` is not empty.

### The review's fixes (2026-10-24), each measured alone

Before the dispatch, the review asked for it-fr-only fixes to its pre-pass — never the shared modules,
so that no other pair moves —, each criterion written before its outcome was counted. Each fix's
effect is the reduction with that fix off against all on, over the same local sources:

| Fix | Criterion | Glosses moved (of the 20,000) | Examples |
|---|---|---|---|
| D4: headed by a meaning, compounds' wordings | above | 2 (2): 1 changed, 1 gained | `te` « Toi : pronom disjoint… », `pel` « Pour le » |
| Leaks (`read_sense`) | an example filed as a sense (`#*`, `#:`, « La traduction en français de l’exemple manque ») or a link's address goes; a colour's hexadecimal codes are taken out; a head line merged into a sense (its transcription between backslashes) gives the meaning after its last « * », or goes | 32 (23) changed; 198 expressions | `io`, `stesso` « Même; Le même », `età`, `signore` « Monsieur », `bianco`, `verde`, `sedere` « Derrière; Asseoir », `natale`, `caffè` |
| Communes (D5) | « commune d' / commune de » is a name's note, after the headword or « Commune, » | 151 (108): 144 changed, 7 lost (names alone: `pella`, `vaglia`, `trino`); 13 of the 1,000 | `ora` « Heure; Maintenant; Or », `vita` « Vie », `lei`, `paese`, `mese`, `sale`, `cento`; `terra`, `luna`, `cristo` keep their names |
| Letters written bare (`LETTER_NAMES`) | a noun sense that is only the French name of the Greek or Cyrillic letter its word also is, from a word holding a common word's meaning in another sense | 4 (4) changed | `ci`, `ce`, `chi` (the pronouns alone), `gamma` « Gamme »; `nu`, `psi`, `pi`, `ro`, `rho` keep their letter |
| Repeated senses (`without_repeated_senses`) | a sense equal to an earlier one of the word up to case, composition and its closing period goes, its run shortened | 87 (39) changed | `italiano` « Italien », `passato` « Passé », `diritto`, `franco`, `beige`, `bordato` |

Two criteria were narrowed after a first count, and the narrowing is recorded: D4's « headed by a
meaning » alone unblocked 107,226 compounds' links (« Agglutination du verbe… avec le pronom gli »), so
the compounds' wordings were read as inflections; and the letters' « other meaning » first counted a
province's acronym or a commune, which took the letter from `nu`, `psi`, `pi`, `ro` and `rho`, so it
counts a common word's meaning alone. None of the fixes moves the coverage at one decimal.

### The study (task 0.1, for the owner)

On change 9's committed ranks (`tables/it/freq.tsv`), D4 and D5 on, letters out, the shared rules
unedited, `gloss_coverage.measure`'s count:

| Sources | 5,000 | 10,000 | 20,000 |
|---|---|---|---|
| The section's definitions alone | 87.6 | 79.5 | 70.5 |
| + the Italian Wiktionary's French translations | 88.0 | 80.0 | 71.0 |
| + the French tables read backwards (both fallbacks) | **89.2** | **81.8** | **73.3** |
| **Proposed floor** (less two points) | **87.2** | **79.8** | **71.3** |

The same figures before and after the review's fixes. 558 lemmas of the 20,000 are glossed by a
fallback (108 direct, 450 read backwards); reading the tables in the other order moves 33 of them,
none in or out (D2's 585 and 38 on its ranking). Before the fixes, D5 off: 89.2 / 81.9 / 73.3; D4 off:
89.1 / 81.8 / 73.2. The proposed floor is written into the requirement and held by
`test_gloss_coverage.py`; the owner settles it before the dispatch's measurement.

### Predicted dispatch output

`tables/it-fr/`: `gloss.tsv` 32,691 rows — 31,006 from the section, 279 from the Italian Wiktionary,
1,405 read backwards, the `USA` row; 94.8 % from a definition; 2.8 % of the glossed 10,000 commonest
from a table —, `senses.tsv` as many, `mwe.tsv` 18,296 (10,704 of the section with a space, 7,517 of
the tables, 22 + 53 split words), `NOTICE`, `manifest.json` (`pack_version` the snapshot), `pin.json`
(the English record kept, a French and an Italian record under the new release); `tables/it/`:
`lexical.tsv` 24,216 dictionary words (8,476 glossed lemmas are names alone — change 11's names
count), `level.tsv` 8,302 levels, **578 lemmas leaving (458 unglossed, 120 names alone) and 578
joining**, 1,162 changing band. Of the
1,425 commonest, 102 are no dictionary word (49 unglossed — `facebook`, `ok`, `web`, `post`, `club` —,
53 glossed by a name alone — `italia`, `roma`, `milano`); D5's 52 was measured on a ranking that drops
the names change 9 ranks. The pack 2,645,342 B (base 1,551,534 B, sha256 `f8c78228…`; layer 1,094,048
B), under the 5 MiB budget: built with `pack_version` `2026.10.24`, so a dispatch on another day moves
the whole pack's and the layer's sha256, never the base's. Of the names and initialisms change 14 levelled, `roma`, `marco`, `napoli`, `francesco`,
`maria`, `gesù`, `luigi`, `adam`, `assunta`, `palestina`, `bellini`, `reich`, `mm`, `cd`, `ps`, `ac`,
`tar` and `cup` leave the levels; `svizzera` (A2, a noun « Suisse »), `alzheimer` (C1, a noun), `pc`
(A2, « PC, ordi personnel ») and `pg` (B2, « Abréviation de personaggio giocante », a noun's lowercase
entry) stay, glossed as words; `bibbia`, `rinascimento` and `corano` leave too, glossed by a proper
noun alone (change 17's list).

### Change 12's D10 figures, re-measured (task 3.2: predicted; re-measured after the dispatch)

The real builder over the predicted tables, as change 12's measurement read its candidates: 18,296
headwords, **14,776 keyed**, 3,520 left out (a word the pack does not hold, or past seven tokens);
by tokens 2: 9,976, 3: 6,057, 4: 1,376, 5: 488, 6: 240, 7: 76, beyond seven 61; of the 75 words
without a space, 62 keyed. D2: 1,750 hold a determiner the key writes, 1,039 keys differ from the
every-token-lemmatised one. D4: 4,677 headwords named; 140 keys several headwords reach (14,634 keys);
`expr.names.zst` 45,560 B, `expr.others` 1,140 B. D5: 157 keys end on `di` (70 on `a`, 13 on `da`, 13
on `su`, 7 on `con`); over change 11a's corpus (1,011,235 blocks), 296,509 matches and **3,000 `di`
retries on 91 expressions** (`avere bisogno di` 564, `a causa di` 561, `alla fine di` 233, `servirsi di`
203, `nel corso di` 197), against change 12's 3,129 on 76 — counted, not labelled. D6: 689 headwords
hold an elision, 7 a truncation, 820 an articulated preposition. The review's fixes move none of these:
they change senses, not headwords.

**The predicted golden** (tasks 4.1, 4.2, on a scratch copy of the predicted tables, reverted; the
checks are committed on the side branch `impl/add-lingua-pack-it-fr-task4`, to apply after the
dispatch): 150 of the 176 sections of `it-fr.golden` move (phrase-gloss 49, gloss 40, word-grammar 31,
analyse 17 — 205 of 1,403 tokens change lemma, the real forms replacing the fixture's —, the pack
line, the vocabulary estimate's universe 121 → 24,214, the ladder's bands); the fixes move 27 of them
against the first prediction (`sale` and `vita` lose their communes). Four of the fixture's checks
change: `Lo` read *lo*, `città` « Cité, ville », and six names of the nomi page read as words of the
pack — `Rossi` *rosso*, `Bianchi` *bianco*, `Conti` *conto*, `Bari` *barare*, `Esposito`, `Ferrari` —, a
known defect for change 17. The six other baselines and `cross_native` pass unblessed on it.

### Known data defects, for change 17

Left for change 17 by the review of the implementation (its examples and counts), beside the
design's list:
- **the wrong sense first, by the entries' order**: `gente`, `fino`, `fra`, `spesso`, `pure`, `spezie`,
  `loro` — the round-robin opens on the first entry's sense, not the commonest one;
- **participles borrowing their infinitive's gloss**: `assicurato`, `avvolto` — a form-of sense in the
  same part of speech lends the verb's senses;
- **pointers that are no `form_of`**: `lettere`, `signor` (« Variante de signore », an `alt_of`), about
  18 lemmas of the 20,000 left unglossed;
- **a name's sense blocking a common word**: `rose`, `panni`, `gemelli`;
- **names losing their level**: `bibbia`, `rinascimento`, `corano`, glossed by a proper noun alone, so
  no dictionary word;
- **`che` « Quoi; Quel »**, with no « Que »;
- **the inherited leaks of the other French-glossed pairs**: es-fr and en-fr carry the same kinds of
  example, head-line and colour-code leaks it-fr's pre-pass now reads; the shared rules are not edited
  here, so that no other pair moves.

Seen in the predicted tables besides: an example the section splits into senses with no marker
(`volere` « …; Il faut dix heures pour aller à Brindisi; Ci vuole un uovo per la torta; … »; `merli`
« murailles d'une forteresse » followed by its Italian); split words that are no Italian word, from the
section (`d'n'b` « Drum and bass ») and the tables (`c'hwerty`, `l'wha`); a mythological name kept
beside a pronoun (`io` « …; Io, fille d'Inachus… »), D5's rule keeping any proper noun's sense that is
no note; surnames ranked by change 9 and glossed « Nom de famille » alone (`lexa`, `micheletto`),
names, so no dictionary word; and the 458 unglossed lemmas that leave the levels (`entro`, `supporto`,
`verifica`: no entry in the sources); commune notes the review's wording does not read (`vittoria`
« …; Vittoria, commune et ville du libre consortium… », `ro` « …; Ro (commune italienne… »).
