# Design — refine-lingua-it-es-glosses

*Work in progress: this file holds, first, the criteria this change measures its rules by, committed
before any figure of a rule of this change was read (as change 27's D8 did). The measurements and the
decisions follow in a later commit.*

## How this was measured

**Criteria, written on 2026-10-25 before any figure of this change's rules was read.** Change 27's own
figures (its design, its README) and the specialist review of its tables (2026-10-25) are this
change's inputs, read before: they are the list, not a measurement of a rule.

- *The base.* it-es's committed tables on `main` at `16da30a05` (change 27, #1068: `gloss.tsv`
  `0ab46596…`, `senses.tsv` `53e6e980…`), reproduced byte for byte by `reduce-it-es.py` over the pinned
  release `lingua-pack-sources-it-es-2026.10.25`, fetched by `pack_sources.py fetch-pinned` into a
  scratch folder and read as data, with Python 3.12 and `requirements-reduce.txt` (wordfreq 3.1.1).
- *The bands.* A row is counted by its lemma's rank in `tables/it/freq.tsv`: 1–1,000, 1,001–5,000,
  5,001–20,000, past 20,000; the 300 commonest are the review's band.
- *Each rule alone, then together.* Each rule is the committed reducer with that rule alone added, in a
  scratch copy; then every rule kept, together. Each is compared row by row with the committed
  `gloss.tsv`: rows changed by band, rows whose first sense changes, lemmas gained and lost, expressions
  changed, and the coverage of the 5,000 / 10,000 / 20,000 commonest.
- *What counts as better.* A row reads **better** when its first sense moves to the Spanish word a
  bilingual dictionary gives first for the word's commonest use — the review's « right gloss » where it
  names one —, or when it loses a sense that is wrong (not Spanish, another word's, a pointer's or a
  description's wording, a name's note on a common word, a repeat) and keeps a right one; **worse** when
  its first sense moves to a rarer, regional, wrong or non-Spanish word, or when it loses its only right
  sense or a common sense no other sense says; **no worse** otherwise (a reordering of right senses, a
  synonym for a synonym).
- *Who reads.* Every changed row among the 1,000 commonest, and, in each of 1,001–5,000 and
  5,001–20,000, every changed row when a rule changes at most 60 there, else 60 drawn by the seed
  20261025. The reader is an agent, not a native speaker; the owner's specialist reads the gloss diff of
  the implementation (tasks).
- *What keeps a rule.* A rule is kept when, in every band read, its better rows outnumber its worse
  rows at least four to one, and no row of the 300 commonest reads worse unless a reviewed row (below)
  restores it; it is narrowed — and the narrowing recorded, with the rows it cost — when it fails that
  and a narrower wording passes; dropped otherwise. A variant of a rule (a threshold, a tie-break, a
  part of speech's handling) is measured the same way, and the variant kept is the one with the most
  better rows among those that pass.
- *What bars a rule, whatever it fixes.*
  - It takes the only gloss of a ranked lemma of the 20,000 commonest, unless that gloss is pure noise
    (a name's note on a common word, a description, a word that is not Spanish, another word's meaning).
  - It puts the coverage of the 5,000 / 10,000 / 20,000 commonest under the floor, **83.2 / 74.0 /
    63.1** (`FLOORS["it-es"]`, settled by the owner, never lowered).
  - It needs a module another pair reads to change: `reduce_common.py`, `reduce_edition_es.py`,
    `reduce_italian_acronyms.py`, the treebank modules, `reduce-fr-es.py`, `reduce-en-es.py`. Such a fix
    is reported, not made. `reduce_spanish_tables.py` may change only while it-es alone imports it,
    shown by every other pair's pin and by fr-es's own reduction, byte for byte.
  - It reads a source it-es's pin does not record, or another pair's committed tables (whose next
    reduction would then move it-es without its pin saying so).
  - It reads UD Italian-PUD or MarkIT's test section.
- *A reviewed row* (`FIRST_RUN`, `CORRECTIONS`) is written only for a clear error of the 300 commonest,
  or of an item change 27 handed to this change, that no kept rule reaches; each with its reason, keyed
  so that it fails or names itself when the source no longer writes what it corrects. Its words come
  from a source where one gives them; a gloss no source gives is **written by a person** and marked so,
  in the code and in the pull request, as change 27's `ne` « De allí, de ahí » and it-fr's `dati`
  « Données ». A matter of taste goes to the owner, with examples.
- *The precedents already settled apply*: a treebank reorders a function word's row, never a content
  word ahead of another (fr-en's Q2, it-fr's and it-en's D2a); a female equivalent is no word's meaning
  (fr-en's Q3); a name opening a row is accepted (fr-en's Q5); a name alone keeps its note (it-en's D6).
- *The target.* On the review's judged rows (its tally, the same rows read again on the new tables),
  the 300 commonest read wrong in at most **3 %** of their rows — the level of the review's bands
  past 300 (4.7 % for 301–5,000, 2 % for 5,001–20,000) and of it-en's 1,000 commonest before change 23
  (about 29 of 1,000) —, against 8.6 % now, with the three unglossed articulated prepositions glossed;
  and the rows read « partly right » fewer than now (11 %). The 301–5,000 and 5,001–20,000 bands judged
  wrong by the review read no worse. A target missed is reported with the rows that miss it, not met by
  lowering a criterion.
