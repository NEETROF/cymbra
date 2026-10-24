# Design

## Context

See proposal.md — Why. What constrains the shape:

- `reduce-fr-en.py` holds French's levels (matrix change 46): `ENGLISH_BANDS` (1,020 A1, 1,158 A2,
  2,015 B1, 2,347 B2, 886 C1, 876 C2 — English's CEFR table's sizes, es-fr's constants),
  `no_level(lemma, forms, senses, words=None)` — unknown or name-only, a lemma whose own form reads as
  another (`elsewhere`), a single character that is no word (`letter`), only another word's spelling
  (`spelling`), and, when `words` is given, not a dictionary word (`unlisted`, the later
  `refine-lingua-fr-en-glosses` D3) — and `estimated_levels(ranks, forms, senses, words, bands)`,
  a left-out lemma giving its slot to the next. `reduce-fr-en.py` is a hyphen-named file no other
  reducer imports; `reduce_common.py` is not edited.
- Change 9 commits `tables/it/forms.tsv` and `freq.tsv` (60,000 lemmas by wordfreq, every rank on
  its own lemma), an empty `lexical.tsv`; change 10 adds spelling rows (a form of its word, no rank);
  change 11 the acronym rows (`NATO`, `USA`: forms in capitals, no rank, in `lexical.tsv`); change
  16 writes it-fr's glosses and `lexical.tsv` as the dictionary words plus the rows.
- The extension: `levelLadder` gives a studied language other than English whose levels are
  estimated English's typical vocabularies (`ENGLISH_TYPICAL_VOCABULARY`) and says so; the card and
  the statistics label estimated levels from the pack's `levels_estimated` flag. The extension's
  studied languages are widened to Italian by change 20.
- The Italian fixture (change 5) carries hand levels for the lemmas the probes and cards name and no
  `levels_estimated` flag; the Italian golden runs over it until change 16.
- Measured (proposal, Why) on a prototype: wordfreq 3.1.1's Italian list, each word ranked by its own string,
  inflected forms skipped, read with the section's own entries; English's committed `tables/en/
  level.tsv` (8,302 lemmas) for the translation check.

## Goals / Non-Goals

**Goals:**

- Italian has a level table, labelled estimated, of English's sizes, read from the section, before
  its glosses exist.
- The rules are French's, so a reader of Italian meets the same ladder as a reader of French.

**Non-Goals:**

- A licensed Italian CEFR list (the *Profilo* is not free; a licence request is the owner's, as
  FLELex's and ELELex's were).
- The interface's wording of the Italian level (« Niveau d'italien estimé »): its words land in
  change 15's catalogue entry, and the level block names Italian from change 20, which widens the
  extension's studied languages.
- Dictionary words (change 16).

## Decisions

### D1 — English's level sizes, over Italian's commonest lemmas (I8 = M7 = D1)

`reduce-it-fr.py` holds `ENGLISH_BANDS` as a constant equal to fr-en's and es-fr's, and
`estimated_levels` — French's, transposed: in rank order then lemma, each band's size given to the
ranked lemmas a CEFR list would hold (D2), a lemma left out giving its slot to the next, so the
bands keep their sizes. A test holds the constant equal to fr-en's (read from `reduce-fr-en.py`'s
source, as French's test reads es-fr's). The report line — each level's rank span and the count
each rule left out — goes in the reduction's output and it-fr's README.

*Rejected — Italian sizes of its own* (a corpus-coverage rule): the ladder's typical vocabularies
are English's (D5), so the sizes must be English's for the two to mean the same.

### D2 — What a CEFR list would leave out, read from the section

French's D2 rules, for Italian, read from the section the forms come from:
1. **unknown or name-only**: the section gives the lemma no sense that is not a form of another word
   (`km`), or only a name's (`milano`, `giovanni`, written there only with a capital) — 1,523 unknown
   and 907 names in the prototype's span;
2. **a single character that is no word**: every sense a letter's — a letter's name read by its
   gloss, as French's `_LETTER_GLOSS` reads it, so `u` « the name of the Latin script letter U/u »,
   filed as a noun, is a letter —, a symbol's, a name's, an abbreviation's, or only another word's
   spelling (`b`, `k`, `u`, `r` « alternative spelling of er », `à` « obsolete spelling of ha ») — while `a`, `e`, `o` are words (the preposition, the conjunctions);
   `è` and `i` are no lemmas (change 9 reads them as *essere* and *il*) — 18;
3. **only a spelling**: every sense only spells another word, by French's `_SPELLING_OF` prefixes
   (`_only_spells_another`), not kaikki's `alt_of` flag — `the` « misspelling of tè », `perchè` —,
   so a contraction or an apocope stays a word (`del` « contraction of di + il », `un` « apocopic
   form of uno »: A1) — 18;
4. **elsewhere**: the lemma's own form reads as another lemma in the forms table — a guard, as
   French's rule 4 now is: since `fix-lingua-lemma-lookup` the builder files a level by the lemma's
   id and refuses a ranked lemma whose own spelling reads as another (`LemmaReadsAsAnother`), and
   change 9 ranks none, so it is expected to leave none out; a level for such a string would land on
   no lemma, which *A level reaches the lemma it is written for* names;
5. **not a dictionary word**: read only once change 16 passes `words` (D4).
The acronym rows of change 11 have no rank, so the rank order never reaches them; the checks hold
them without a level (D7).

### D3 — Every pack studying Italian says its levels are estimated

it-fr's `manifest.json` carries `levels_estimated: true`, written by `reduce-it-fr.py` as es-fr's
and fr-en's are; it-fr's NOTICE says the levels are estimated from word frequency, no CEFR list
being read; it-en and it-es (stage 2) read `tables/it/level.tsv` as committed and say the same.

### D4 — Change 16 narrows the levels to dictionary words

Until it-fr's glosses are committed, `tables/it/lexical.tsv` holds the acronym rows alone (change 11)
and the levels read no pair's glosses. Change 16 commits the glosses and the dictionary words in one
pull request and passes them as `words`, as `refine-lingua-fr-en-glosses` did for French: a level
then goes only to a word a card can gloss (names levelled before through a sense of another part of
speech, from a lowercase or a capitalised entry — `roma`, `marco`, `napoli`, `francesco` from a
lowercase one; `maria`, `gesù`, `adam`, `assunta`, `tommy`, `palestina`, `cambogia`, `bellini`,
`reich`, `alzheimer` and the initialisms `mm`, `pc`, `cd`, `ps`, `ac`, `tar`, `pg`, `cup` from a
capitalised one, 59 lemmas in all from a capitalised entry — keep a level only if they are dictionary
words), and `committed_tables.rs`
gains « every Italian levelled lemma is a dictionary word », as Spanish's and French's checks. The
table moves then, in change 16's pull request; afterwards, glosses that add or remove no dictionary
word, and another pair's glosses, leave it byte for byte (French's test).

### D5 — What the reader sees: nothing to build here

The engine reports estimated levels from the flag; `levelLadder` gives English's typical
vocabularies for an estimated non-English language and says they are English's; the card and the
statistics label the level estimated. None of it is wired for Italian until change 20 widens the
extension's studied languages; nothing of the core, the engine or the extension changes here.

### D6 — The golden does not move

The Italian baseline runs over its fixture pack until change 16 (change 9's D9); the fixture's levels
are hand-written, and no `levels_estimated` flag is set on it (change 5's D5). Change 16's hand-over
re-blesses on the committed tables, which then show the estimated levels and the flag; if change 16
lands before this change, this change re-blesses the golden, naming the level probes (French's 46
D7). Under the split (changes 3, 4), `levels_estimated` lives in the base's metadata, held per
language by change 3's check (its D6), and the level table in the base: the base grows by it (measured at
task 2.1, as French's D9 measured +60,035 B).

### D7 — Tests and what cannot move

- `test_reduce_it_fr.py`: each rule on fixture entries (unknown, a name, a letter, `e` a word,
  `perchè` a spelling, `km` unknown, `milano` a name, `k`, `u` and `r` no words, `del` levelled, a lemma whose form reads as another left out by the guard); the bands' sizes; a left-out lemma
  giving its slot; the constant equal to fr-en's.
- `committed_tables.rs`: Italian's 8,302 levels, as many at each level as English's, each a ranked
  lemma of `tables/it/freq.tsv`; it-fr's pack gives each its level (*A level reaches the lemma it is
  written for*); its manifest says estimated.
- The six other pairs' tables, pins and packs rebuild byte for byte; their baselines pass without
  re-blessing.

## Risks / Trade-offs

- [An estimate, not a list] → measured by one script beside French's and Spanish's committed
  tables (proposal); I8 accepted estimated levels with the label.
- [Names with a sense of another part of speech, from a lowercase or a capitalised entry] → `roma`
  (rice), `marco` (the coin), `napoli`, `francesco`, `maria`, `gesù`, `reich`, initialisms levelled until
  change 16 narrows the levels to dictionary words (D4).
- [A common noun read as a verb's form] → `porta` is no lemma of change 9's tables (its D5, M8), so
  it has no level; French's analogue is `porte`.

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr. The table arrives by the update
workflow's pull request; rollback is a revert.

## Measured

A scratch script over `kaikki-Italian.jsonl` and wordfreq 3.1.1's Italian list: each word ranked by
its own string (change 9's D6), a word skipped when the section gives it only as an inflection of
another, French's rules of D2 ported; the same translation check run on French's and Spanish's
committed `level.tsv` and their English glosses (`fr-en/gloss.tsv`, `es-en/gloss.tsv`) — the figures
of the proposal. Folding inflected forms into their lemma by summed frequency (which French's D1
rejected) moves the agreement figures by under a point and the A1 figure from 55.2 % to 60.1 %. Re-run on change 9's committed ranks at task 2.1.

## As implemented

`reduce-it-fr.py` holds the section *Italian's estimated levels*: `ENGLISH_BANDS`, French's
`_LETTER_POS`, `_NOT_A_WORD_TAGS`, `_LETTER_GLOSS` and `_SPELLING_OF`, `word_senses` and
`read_level_senses` (a pass of their own over the section, the forms' `_is_form_of` deciding what is a
form of another word), `no_level`, `estimated_levels` and `levels_report` (which takes the bands, for
its test). `reduce` writes `level.tsv` after `freq.tsv` from the ranks, the forms just chosen and the
section — no `words` until change 16 —, the report line after the acronyms', and the summary's
`levels` and `levels left out`; `manifest` writes `levels_estimated: true`, `notice` the levels'
sentence. `reduce_common.py` is not edited.

**The local reduction** (task 2.1's figures, made as the update makes it: Python 3.12 and
`requirements-reduce.txt`, `build.sh --reduce it-fr` from the pinned 2026.10.23 sources, the release
assets `lingua-pack-sources-it-fr-2026.10.23` and `lingua-pack-sources-morph-it-0.4.8`, MarkIT at
`e210079d`). Reducing `main` first gave its committed tables byte for byte. With this change:
8,302 levels, A1 1–1,080, A2 1,081–2,380, B1 2,381–4,855, B2 4,857–8,052, C1 8,053–9,371, C2
9,377–10,690; left out 1,728 unknown, 635 names, 16 letters, 9 spellings, none by the guard; `forms.tsv`,
`freq.tsv`, `grammar.tsv`, `lexical.tsv`, `tags.tsv` and `studied.json` byte for byte; the base grows
by 60,037 B (1,491,875 → 1,551,912 B), the layer by 265 B (the NOTICE's sentence), the pack to
1,553,605 B. The figures, the measurement and the weaknesses are recorded in `tables/it-fr/README.md`,
*The levels*.

**Departures from the proposal and this design**, each recorded in the pull request:

1. *The spans and counts moved* from the prototype's: A1 ends at 1,080 (1,094), C2 spans
   9,377–10,690 (9,446–10,768); 272 fewer names — exactly the name-only words among wordfreq's 14,000
   commonest that change 9's names rule reads as another lemma's form (`sua` → *suo*, `grandi` →
   *grande*), which the prototype ranked by their own string —, 205 more unknown words (none has an
   entry in the section), 9 spellings instead of 18 (change 10 reads most as their word), 16 letters
   instead of 18 (wordfreq's elision stems are no ranks).
2. *The measurement's script is not the proposal's*: no variant of 72 tried on French and Spanish
   reproduces the proposal's figures (the closest is up to 2.3 points off), so the three languages are
   measured again by one script, its criteria written before Italian's figures were read. Italian's
   figures read like French's and Spanish's, but two criteria are missed: C2, skipping articles,
   within one level 57.9 % against 60.3 % and 61.1 % (the lower less two points, missed by 0.4 point;
   the prototype measured the same gap); C3, the plain proxy's mean English level flat from B1 to B2
   (2.38 → 2.37), rising when skipping.
3. *The vocabulary estimate*: `Pack::dictionary_words` counts a ranked lemma that is a dictionary word
   or carries a level (add-lingua-cefr-levels), so it-fr's estimate counts its 8,302 levelled lemmas
   until change 16's dictionary words join them; no reader sees it before change 20. The committed
   check that held it empty now holds it to the levelled lemmas.
4. *`porta` and `perchè`* are no ranked lemmas of the committed tables (M8; change 10): the spec's
   scenario of a ranked lemma whose own form reads as another, and `perchè`'s spelling rule, are held
   on fixture entries, as French's `donnée` was; on the committed tables, `perchè` reads as *perché*
   and `porta` as *portare*, each with its word's level.

## Open Questions

None blocking: the numbers' and the C2 edge's treatment follow French's 46 as implemented; a licence
request for an Italian CEFR list (the *Profilo*) is the owner's, as FLELex's was, and nothing here
waits for it.

## Effort

1.5–2.25 ideal days, against the programme's 1.5–3: the reducer's bands, rules and report 0.5–0.75;
their tests and the engine's test 0.5–0.75; the reduction, the manifest, the NOTICE and the README 0.25; the committed
checks 0.25; the measurement on change 9's ranks 0.25–0.5.
