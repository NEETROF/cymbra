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
- Measured (proposal, Why) on a prototype: wordfreq 3.1.1's Italian list, surface words in its order,
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
   (`km`), or only a name's (`milano`, `giovanni`, written there only with a capital) — 1,519 unknown
   and 906 names in the prototype's span;
2. **a single character that is no word**: every sense a letter's, a symbol's, a name's, an
   abbreviation's, or only another word's spelling (`b`, `k`, `r` « alternative spelling of er », `à`
   « obsolete spelling of ha ») — while `a`, `e`, `o` are words (the preposition, the conjunctions);
   `è` and `i` are no lemmas (change 9 reads them as *essere* and *il*) — 15;
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
then goes only to a word a card can gloss (`roma`, `marco`, `maria`, levelled through their
lowercase senses before, keep a level only if they are dictionary words), and `committed_tables.rs`
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
language by change 4's check, and the level table in the base: the base grows by it (measured at
task 2.1, as French's D9 measured +60,035 B).

### D7 — Tests and what cannot move

- `test_reduce_it_fr.py`: each rule on fixture entries (unknown, a name, a letter, `e` a word,
  `perchè` a spelling, `porta` elsewhere, `NATO` an acronym row); the bands' sizes; a left-out lemma
  giving its slot; the constant equal to fr-en's.
- `committed_tables.rs`: Italian's 8,302 levels, as many at each level as English's, each a ranked
  lemma of `tables/it/freq.tsv`; it-fr's pack gives each its level (*A level reaches the lemma it is
  written for*); its manifest says estimated.
- The six other pairs' tables, pins and packs rebuild byte for byte; their baselines pass without
  re-blessing.

## Risks / Trade-offs

- [An estimate, not a list] → measured by one script beside French's and Spanish's committed
  tables (proposal); I8 accepted estimated levels with the label.
- [Names with a lowercase homograph] → `roma` (rice), `marco` (the coin), `maria` levelled until
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
rejected) moves them by under a point. Re-run on change 9's committed ranks at task 2.1.

## Open Questions

None blocking: the numbers' and the C2 edge's treatment follow French's 46 as implemented; a licence
request for an Italian CEFR list (the *Profilo*) is the owner's, as FLELex's was, and nothing here
waits for it.

## Effort

1.5–2.25 ideal days, against the programme's 1.5–3: the reducer's bands, rules and report 0.5–0.75;
their tests 0.25–0.5; the reduction, the manifest, the NOTICE and the README 0.25; the committed
checks 0.25; the measurement on change 9's ranks 0.25–0.5.
