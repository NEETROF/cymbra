# read-lingua-italian-unmarked-spellings — the apostrophe that stands for an accent, and the web's Italian without its accents

## Why

Change 10 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Italian
marks its final stressed vowels — *perché*, *più*, *città*, *è*, *può*, *così* — and the web
writes them three ways: with the accent, with an apostrophe in its place (`perche'`, `piu'`,
`citta'`, `e'`, the keyboards of the 1990s had no accented capitals and many have no `È` still:
« E' vero »), and with nothing (`perche`, `piu`, `citta`). The corpus's `informale` page holds all
three. Change 6 reads a trailing apostrophe for a listed elided form or truncation and leaves any
other outside the word, as the rule of no language does, so `perche'` is `perche` today — an
unknown word; `e'` is `e`, the conjunction, glossed wrong; `E'` is `E`. French met the second and
third kinds (matrix changes 43d and 43e: the dictionary's own accent-less spellings, then the
web's by a reviewed list of 468 rows); Italian meets a first kind of its own, the apostrophe,
which French never writes for an accent.

## What Changes

- **The apostrophe that stands for an accent is read by rule** (design D1): an Italian word
  ending in a vowel and an apostrophe, with no letter after it, whose vowel accented is a form the
  tables list and which the tables list under no word as written, is read as that form — `perche'`
  → `perché`, `piu'` → `più`, `citta'` → `città`, `e'` → `è`, `E'` → `È`, `puo'` → `può`,
  `cosi'` → `così`, `caffe'` → `caffè`, `sara'` → `sarà` —, one token spanning the apostrophe, by
  change 6's words seam, which keeps the apostrophe for it as for a truncation. A word the tables
  list as written with its apostrophe (`po'`, `da'`) keeps change 6's reading; a word whose
  accented form the tables do not list is read as today, the apostrophe outside it.
- **The dictionary's own accent-less spellings read as their word** (D2, change 43d's rule): a
  form the Italian section gives only as a misspelling or alternative spelling of a marked word —
  every sense of every entry of it such a spelling — is a form of that word in the tables when
  wordfreq rates the marked word commoner than the spelling and than each of the spelling's forms;
  nothing of its own remains.
- **The web's accent-less spellings read as the words a reviewed list names** (D3, change 43e's
  method): every Italian form of the tables ending in an accented vowel, its accent dropped, kept
  when wordfreq attests the spelling at Zipf ≥ 2.5, the section gives it no meaning of its own,
  its accented word is an Italian dictionary word, it is less than half as frequent as that word,
  and neither English nor another language of the matrix writes it more often; the list is in the
  design with each spelling's frequencies, the words left out listed by group — `e` (*and*), `da`,
  `si`, `se`, `ne`, `la`, `li` (words of their own beside *è*, *dà*, *sì*, *sé*, *né*, *là*, *lì*),
  `papa` (*pope* beside *papà*), `eta` (the letter), and every spelling English writes more.
- **A spelling carries its word's readings** (D4) and nothing of its own: no rank, no level, no
  gloss, no dictionary word; the card shows the accented word.
- **Italian's analyser version bumps** (`1.1.0` → `1.2.0`): the apostrophe rule is a tokenisation
  rule; the tables move (the dictionary's spellings and the list are rows of `tables/it/forms.tsv`),
  it-fr re-reduced, its pin and `pack_version` moved; the Italian golden re-blessed — the
  `informale` page's words resolve.
- **English, Spanish, French and the six goldens do not move.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *The apostrophe that stands for an accent reads as the accented
  word*; MODIFIED — *Italian tokenisation pre-pass* (change 6's, open: its sentence on any other
  trailing apostrophe now names this rule; every scenario kept, its apostrophe-accent scenario
  rewritten).
- `lingua-data-packs`: ADDED — *An Italian spelling the dictionary gives only as a marked word
  reads as that word when the marked word is the commoner* and *The web's Italian spellings without
  their marks read as the words a reviewed list names*; both read change 9's tables, after which
  this change archives.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/tokenize.rs` (the apostrophe rule in Italian's words seam, with
  the lexicon), `analysis/mod.rs` (`1.2.0`), `tests/italian_fixtures.rs`.
- `scripts/lingua-data/reduce-it-fr.py` (the dictionary's spellings, the reviewed list as a
  literal with its reasons), `tables/it/forms.tsv`, `tables/it-fr/{pin.json, manifest.json,
  README.md}` (re-reduced), `SOURCES.md`; `scripts/lingua-data/measure/it-ud.sh`'s figures.
- `crates/lingua-wasm/tests/italian_baseline.rs` and `baseline/it-fr.golden`;
  `apps/lingua-extension/test/packs.spec.ts` (`1.2.0`).
- Untouched: the six pairs' tables, pins and goldens; the extension's bundle; the agent; the
  server; the site.
