# Design

## Context

See proposal.md — Why. What constrains the shape:

- Change 6's words seam re-attaches a trailing apostrophe only for a listed word — an elided form
  alone or a truncation — and never after an opening quote; any other trailing apostrophe stays
  outside the word by the rule of no language. So `perche'` is `perche`, `e'` is `e`, `E'` is `E`
  at `1.1.0`.
- Change 9's tables list Italian's forms with their accents (`perché`, `più`, `città`, `è`, `può`,
  `così`); the English Wiktionary's Italian section marks some accent-less spellings as
  misspellings or alternative spellings of their word (`perche` « misspelling of perché »), as the
  French section marked `etre`; wordfreq rates each spelling and its word.
- French's two precedents: matrix change 43d reads a spelling the section gives only as a marked
  word when the word is the commoner (a rule over the tables, nothing of the spelling's own left);
  change 43e reads the web's spellings by a reviewed list of 468 rows kept by four conditions,
  merged into one literal with its reasons (`UNMARKED_SPELLINGS`), each row a form of its word.
- The Italian corpus's `informale` page holds `perche'`, `e'`, `E'`, `piu'`, `citta'`, `perche`,
  `cosi`.
- Several accent-less spellings are Italian words of their own: `e` (*and*) beside *è*, `da`
  beside *dà*, `si` beside *sì*, `se` beside *sé*, `ne` beside *né*, `la`/`li` beside *là*/*lì*,
  `papa` beside *papà*, `eta` (the letter) beside *età*.

## Goals / Non-Goals

**Goals:**

- The three ways the web writes a final stressed vowel resolve to the accented word, with its
  gloss, rank and readings, wherever the reading is safe.
- A spelling that is also a word keeps its own reading: Italian recall of *e*, *da*, *si* comes
  first.

**Non-Goals:**

- Accents inside a word (`perchè` with a grave instead of the acute is the dictionary's own
  spelling, read by D2 if marked; otherwise left), capitals without accents in running text other
  than `E'`.
- A spelling of a word the tables do not rank.

## Decisions

### D1 — The apostrophe that stands for an accent: a rule over the lexicon, in the words seam

In Italian's words seam, a word ending in a vowel followed by an apostrophe (straight or
typographic) with no letter after it, and no opening quote before the word, is a candidate: the
word with its final vowel accented — `a` → `à`, `e` → `è` or `é`, `i` → `ì`, `o` → `ò`, `u` → `ù`
— is looked up in the lexicon; when one accented form is listed and the word as written (with its
apostrophe) is not, the token is that form, spanning the word and its apostrophe, its written
capital kept (`E'` → `È`). `e` tries `è` then `é` (`perche'` → `perché`, `e'` → `è`; `caffe'` →
`caffè`): the lexicon decides; a word the tables list under both accents is measured, and the
first listed wins with the order fixed in the doc. Change
6's listed words come first: `po'`, `da'`, `fa'`, `va'`, `sta'`, `di'` keep their truncation
reading (`pò`, `dà`, `fà` are not tried). A word whose accented form the tables do not list is
left as before.

*Rejected — a reviewed list of apostrophe spellings.* They are the accented words' tail, one for
one; the lexicon is the list, and change 9's tables update it.

### D2 — The dictionary's own accent-less spellings: change 43d's rule

`reduce-it-fr.py` reads a form the section gives only as a misspelling, alternative, obsolete or
nonstandard spelling of a marked word — every sense of every entry of it such a spelling — as a
form of that word, when wordfreq rates the marked word strictly commoner than the spelling and
than each of the spelling's forms; its inflections and the entries pointing at it follow; nothing
of its own remains. A spelling with a sense of its own beside the marking keeps its entry.
Measured on the tables: the spellings read, the ones refused and why, as 43d listed them.

### D3 — The web's accent-less spellings: change 43e's method, Italian's list

Candidates: every form of the tables ending in an accented vowel, its accent dropped. Kept when
wordfreq attests the spelling at Zipf ≥ 2.5; the section gives it no meaning of its own and does
not give it as a spelling of its word (D2's kind); its accented word is an Italian dictionary
word; the spelling is less than half as frequent as the word; and no other language of the matrix
(English, French, Spanish — wordfreq's lists) writes it more often than Italian does. Left out, by
group and listed in the design when measured: the words of their own (`e`, `da`, `si`, `se`, `ne`,
`la`, `li`, `papa`, `eta`, `pero` — the pear tree beside *però*), the spellings English or French
write more (measured: `cafe` likely among them),
those as common as their word or more, hyphenated words wordfreq cannot count, and D2's
spellings. The rows join `reduce-it-fr.py`'s table as one literal of `spelling → word` with one
shared reason, each a form of its word carrying its readings.

### D4 — Nothing of a spelling's own; the readings follow the word

A read spelling has no rank, level, gloss or dictionary-word bit; the pack keys everything by the
accented lemma, and the card shows it. Its readings are the word's (change 13's grammar tables
file them under the word).

### D5 — Version, tables, goldens

`ITALIAN_ANALYZER_VERSION` `1.1.0` → `1.2.0` (D1 is a tokenisation rule); it-fr re-reduced
(D2 and D3 are rows), `pin.json`, `manifest.json`, `README.md` moved; `it-ud.sh`'s figures
re-measured and recorded; the Italian golden re-blessed — the `informale` page's `perche'`,
`e'`, `E'`, `piu'`, `citta'` resolve by D1, `perche` and `cosi` by D2 or D3; nothing else moves.
The six goldens and pins do not move.

## Risks / Trade-offs

- [`e'` read as *è* where a writer meant *e* with a stray apostrophe] → vanishingly rare against
  « E' vero »; measured on the corpus and Tatoeba.
- [A spelling that is a word] → the list excludes every form the tables list as a word; the rule
  of D1 never applies to a word the tables list as written.
- [English writes `citta`? `caffe`?] → the cross-language frequency condition decides, as 43e's
  Q2 did for French.
- [A quoted word ending in a vowel] → no re-attachment after an opening quote (change 6's D5).

## Migration Plan

Nothing deploys and nothing is released; no package carries it-fr. Rollback is a revert with the
tables', the pin's and the golden's.
