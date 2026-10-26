# Design — refine-lingua-en-it-glosses

## Context

See proposal.md (Why). What exists, on `main` at `c9a08c784` (change 36, #1081; its tables committed
by the owner's update of 2026.10.26, run 40324469757):

| Where | What |
|---|---|
| `reduce-en-it.py` | en-it's reducer: the native side alone, against `tables/en/` as committed — the Italian Wiktionary's English section (`without_letters` → `italian.read_as_meanings`), then `native_side`: the section's definitions, then the English Wiktionary's Italian table (direct), then the Italian Wiktionary's English translations read backwards (inverted), the first that glosses a lemma glossing it whole; en-es's table readers copied (change 36's D3) |
| `reduce_edition_it.py` | The Italian Wiktionary's rules (change 35): read by en-it alone among the committed pairs, and by fr-it and es-it (changes 47 and 51, proposed, not implemented) |
| `reduce_common.py` | The shared rules, every pair's digest: not edited here |
| `tables/en-it/` | 21,526 glossed lemmas (definitions 13,677, 55 through a pointer; the direct table 6,125; the inverted 1,724); 13,423 expressions; coverage **91.4 / 82.8 / 70.3 %** against `FLOORS["en-it"]` **89.4 / 80.8 / 68.3**; `gloss.tsv` `34ecdd39…`, `senses.tsv` `9e384393…`, `mwe.tsv` `646293cf…` |
| `tables/en/` | English's studied side, en-fr's, read as committed; no rule here writes it |
| The requirement this change touches | Change 36's *English is glossed in Italian from the Italian Wiktionary's English section and the two Wiktionaries' translation tables*, open, held by `add-lingua-pack-en-it`; changes 47 and 51 (proposals) each hold a MODIFIED copy of it |
| The lists for this change | Change 36's *Known defects, for change 37* (its D10), its README's; the specialist review of the committed tables (2026-10-26: error rates by band, the 23 wrong rows of the 300 commonest with the right Italian, the partly right rows worth fixing, systematic issues A–J, with its scripts and samples) |
| Change 36's open task 4.3 | `test/word-card-en-it.spec.ts` and `test/baseline/word-card-en-it.txt`, the full card in Italian over en-it's golden, and its line in `lingua-pack-update.yml`'s bless step: it waited for changes 33 and 34, both now on `main` (#1080, #1082) |

## Goals / Non-Goals

**Goals:**
- en-it's glosses open on the Italian word an English word's commonest use calls for, before change 45
  lists en-it — by rules of en-it's own modules, each measured alone on the committed tables and kept
  only where it reads better (the criteria below), and short reviewed tables for what no rule reaches.
- Every item of the lists above decided: a rule, a reviewed row, left with its reason, or put to the
  owner in plain words with examples (tasks group 0).
- Change 36's open task 4.3 done here: the card in Italian, its snapshot and its bless line.

**Non-Goals:**
- Any byte of `tables/en/` or of the other committed pairs (en-fr, es-fr, es-en, en-es, fr-en, fr-es,
  it-fr, it-en, it-es): no rule of `reduce_common.py` is edited, and `reduce_edition_it.py` only under
  the criteria's condition below.
- A new source: en-it's pin keeps its three files.
- The coverage floor: `FLOORS["en-it"]` stays 89.4 / 80.8 / 68.3.
- Shipping en-it (change 45).

## How this was measured

**Criteria, written on 2026-10-26 before any figure of this change's rules was read.** Change 36's own
figures (its design, its README) and the specialist review of its tables (2026-10-26) are this change's
inputs, read before: they are the list, not a measurement of a rule.

- *The base.* en-it's committed tables on `main` at `c9a08c784` (change 36, #1081: `gloss.tsv`
  `34ecdd39…`, `senses.tsv` `9e384393…`, `mwe.tsv` `646293cf…`), reproduced byte for byte by
  `reduce-en-it.py` over the pinned release `lingua-pack-sources-en-it-2026.10.26`, fetched by
  `pack_sources.py fetch-pinned` into a scratch folder and read as data, with Python 3.12 and
  `requirements-reduce.txt` (wordfreq 3.1.1). No figure is read before that reproduction holds.
- *The bands.* A row is counted by its lemma's rank in `tables/en/freq.tsv`, the file
  `gloss_coverage.py` and the review read: 1–1,000, 1,001–5,000, 5,001–20,000, past 20,000; the 300
  commonest are the review's band.
- *Each rule alone, then together.* Each rule is the committed reducer with that rule alone added, in a
  scratch copy; then every rule kept, together. Each is compared row by row with the committed
  `gloss.tsv`: rows changed by band, rows whose first sense changes, lemmas gained and lost,
  expressions changed, and the coverage of the 5,000 / 10,000 / 20,000 commonest.
- *What counts as better.* A row reads **better** when its first sense moves to the Italian word a
  bilingual dictionary gives first for the English word's commonest use — the review's « right Italian »
  where it names one —, or when it loses a sense that is wrong (not Italian, a description where a
  translation exists, a pointer's wording, a name's note on a common word, heraldry, a repeat, a
  misspelling, a false friend) and keeps a right one; **worse** when its first sense moves to a rarer,
  wrong or non-Italian word, or when it loses its only right sense or a common sense no other sense
  says; **no worse** otherwise (a reordering of right senses, a synonym for a synonym).
- *Which part of speech is the commonest.* en-it's pin holds no English treebank, and no rule reads one
  it does not record. The evidence a rule may use is what the pin and the committed tables hold: the
  part of speech of each sense run (`senses.tsv`), English's readings (`tables/en/grammar.tsv`), the
  words both translation tables list under each part of speech, and wordfreq's Italian list, already
  credited in en-it's notice. Each such evidence is measured as a variant, by the criteria below, and
  the commonest grammatical words, where no evidence is safe, are a reviewed table.
- *Who reads.* Every changed row among the 1,000 commonest, and, in each of 1,001–5,000 and
  5,001–20,000, every changed row when a rule changes at most 60 there, else 60 drawn by the seed
  20261026 (the review's). The reader is an agent, not a native speaker; the owner's specialist reads the
  gloss diff of the implementation (tasks).
- *What keeps a rule.* A rule is kept when, in every band read, its better rows outnumber its worse
  rows at least four to one, and no row of the 300 commonest reads worse unless a reviewed row (below)
  restores it; it is narrowed — and the narrowing recorded, with the rows it cost — when it fails that
  and a narrower wording passes; dropped otherwise. A variant of a rule (a threshold, a tie-break, a
  part of speech's handling, a band it is limited to) is measured the same way, and the variant kept is
  the one with the most better rows among those that pass.
- *What bars a rule, whatever it fixes.*
  - It takes the only gloss of a ranked lemma of the 20,000 commonest, unless that gloss is pure noise
    (a name's note on a common word, a description with no Italian word for the lemma, a word that is
    not Italian, another word's meaning, heraldry).
  - It puts the coverage of the 5,000 / 10,000 / 20,000 commonest under the floor, **89.4 / 80.8 /
    68.3** (`FLOORS["en-it"]`, settled by the owner, never lowered).
  - It needs `reduce_common.py`, or a reducer or module another committed pair loads, to change. Such a
    fix is reported, not made. `reduce_edition_it.py` may change only while en-it is its only committed
    reader, shown by every other committed pair's pin and rule digest, byte for byte; a change to it is
    measured on fr-it's and es-it's prototypes too (changes 47 and 51) and reported to them.
  - It reads, when the reducer runs, a source en-it's pin does not record, or another pair's committed
    tables (en-es's order, `tables/it/`): their next reduction would move en-it without its pin saying
    so. Such a table may be read by the measurement, outside the reducer, to list candidates for a
    reviewed row (`tables/it/forms.tsv` for misspellings), never by a rule.
  - It writes an Italian word no source gives, outside a reviewed row marked as a person's.
- *A reviewed row* (`FIRST_RUN`, `CORRECTIONS`, the false friends) is written only for a clear error of
  the 300 commonest, of an item change 36 or the review handed to this change, or of a class a rule
  cannot tell apart from right rows, that no kept rule reaches; each with its reason, keyed so that it
  fails or names itself when the source no longer writes what it corrects. Its words come from a source
  where one gives them; a gloss no source gives is **written by a person** and marked so, in the code
  and in the pull request, as it-es's `della` « De la » and it-fr's `dati` « Données », and needs the
  owner's approval. A matter of taste goes to the owner, with examples.
- *The precedents already settled apply*: a name opening a row is accepted when the word is only a name
  (it-en's D6); a given name read as its own language's name in short form (it-es's D7b, the owner's
  answer of 2026-10-25); a displaced first word that is the right cognate stays as a sense of its own
  (it-es's D2g, round 2); a list ordered by its sources' support, then by frequency, before it is cut
  (it-es's D7c); a repeat dropped where it is the same words (it-es's D9b), not where only a parenthesis
  differs.
- *The target.* On the review's judged rows — the 299 glossed rows of the 300 commonest, its 150 of
  301–5,000 and its 100 of 5,001–20,000 (seed 20261026), read again on the new tables by this design's
  reader —, the 300 commonest read wrong in at most **3 %** of their rows, the level of the review's
  bands past 300 (3.3 % and 3.0 %), against 7.7 % now; the rows read « partly right » fewer than now
  (19.7 %); the two samples past 300 no worse. A target missed is reported with the rows that miss it,
  not met by lowering a criterion.

**The data.** en-it's pinned release, `lingua-pack-sources-en-it-2026.10.26`, fetched by
`pack_sources.py fetch-pinned` into a scratch folder (every asset checked by the sha256 its pin records);
read as data, by scripts outside the repository, with Python 3.12 and wordfreq 3.1.1
(`requirements-reduce.txt`).

**The prototype.** A scratch copy of `reduce-en-it.py` with each rule as a switch. Every figure below is
re-measured by the real reduction at implementation (task 3.2).
