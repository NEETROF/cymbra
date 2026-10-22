# read-lingua-italian-unmarked-spellings — the apostrophe that stands for an accent, and the web's Italian without its accents

## Why

Change 10 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Italian
marks its final stressed vowels — *perché*, *più*, *città*, *è*, *può*, *così* — and the web
writes them three ways: with the accent, with an apostrophe in its place (`perche'`, `piu'`,
`citta'`, `e'`, `c'e'`; the keyboards of the 1990s had no accented capitals and many have no `È`
still: « E' vero »), and with nothing (`perche`, `piu`, `citta`). The corpus's `informale` page
holds all three. Change 6 reads a trailing apostrophe for a listed elided form or truncation and
leaves any other outside the word, as the rule of no language does, so `perche'` is `perche` today
— an unknown word —; `e'` is `e`, the conjunction, glossed wrong; `c'e'` is `ci` + `e`; `E'` is
`E`. French met the second and third kinds (matrix changes 43d and 43e: the dictionary's own
accent-less spellings, then the web's by a reviewed list of 468 rows); Italian meets a first kind
of its own, the apostrophe, which French never writes for an accent.

Measured with wordfreq 3.1.1 (Zipf, Italian): `perche` 4.95 against `perché` 6.18, `piu` 5.22 /
`più` 6.63, `citta` 4.16 / `città` 5.77, `cosi` 5.12 / `così` 6.14, `puo` 4.61 / `può` 6.01, `gia`
4.59 / `già` 5.95, `caffe` 3.75 / `caffè` 4.83 — and wordfreq strips an edge apostrophe, so
`perche'` and `perche` are one token to it, as `e'` and `e`: the apostrophe spellings are counted
inside the accent-less figures. The English Wiktionary's Italian section gives `perche` as the
plural of *perca* (the perch, a fish), `cosi` as the plural of *coso* (*thingy*), `citta` as
*citto*'s feminine, `caffe` as *caffo*'s — words of their own to 43e's method, which would leave
the four commonest web spellings unread; and it marks `perchè`, `nè` and `caffé` as misspellings of
*perché*, *né* and *caffè*, 43d's kind.

## What Changes

- **The apostrophe that stands for an accent is read by rule** (design D1): in Italian's words
  seam, after change 6's elided pieces are split off, what remains of a word — ending in a vowel
  and an apostrophe with neither a letter nor a digit after it, no opening quote before the written
  word — is read as the accented form the pack lists when the pack lists none of it as written:
  `perche'` → `perché`, `piu'` → `più`, `citta'` → `città`, `e'` → `è`, `E'` → `È`, `c'e'` → `ci` +
  `è`, `dov'e'` → `dove` + `è`, `puo'` → `può`, `cosi'` → `così`, `caffe'` → `caffè` —, one token
  spanning the apostrophe. When the pack lists both `è`- and `é`-spellings of a word (`perchè` as a
  form of *perché* beside `perché`), the one the pack files under itself wins, else `è` before `é`:
  the tables decide, never a frequency, since wordfreq cannot tell the writings apart. Change 6's
  eight truncations keep their reading (`da'` → `dai`, not `dà`); the literary apocopes `ne'`, `de'`,
  `co'`, `pe'`, `a'`, `su'` are never read for an accent.
- **The dictionary's own accent-less spellings read as their word** (D2, change 43d's rule without
  its condition that the spelling be the word with its marks dropped, on purpose): a form the
  Italian section gives only as a misspelling or alternative spelling of a marked word — every
  sense of every entry of it such a spelling — is a form of that word in the tables when wordfreq
  rates the marked word commoner than the spelling and than each of the spelling's forms;
  `perchè`, `nè`, `caffé`, `fà`, `stà`, `và`, `sù`, `pò` and the doubled-consonant misspellings
  (`sopratutto`) come in.
- **The web's accent-less spellings read as the words a reviewed list names** (D3, change 43e's
  method): every Italian form of the tables ending in an accented vowel, its accent dropped, kept
  when wordfreq attests the spelling at Zipf ≥ 2.5, the section gives it no meaning of its own, its
  accented word has a sense of its own in the section that is no name's (until change 16's
  dictionary words, which re-check the rows), it is less than half as frequent as that word, and
  neither English, French nor Spanish writes it more often than Italian does; **four exception rows
  the owner settles**, outside the method as 43e's `sure` → *sûr*: `perche` → *perché*, `cosi` →
  *così*, `citta` → *città*, `caffe` → *caffè*, each the section's rare homograph against the web's
  commonest spelling. Left out, by group: the words of their own (`e`, `da`, `si`, `se`, `ne`,
  `la`, `li`), `papa` (commoner than *papà*: 5.15 against 4.66), `eta` and `pero` (the Greek letter, the pear
  tree; Spanish writes them more besides: 4.10 against 3.73, 6.57 against 4.42), `cafe` (English's, 4.09 against 3.33).
- **A spelling carries nothing of its own** (D4): no rank, no level, no gloss, no dictionary word;
  the card shows the accented word; once change 13 gives Italian readings, a spelling carries its
  word's.
- **Italian's analyser version bumps** (`1.1.0` → `1.2.0`, or the next minor if change 8 lands
  second): the apostrophe rule is a tokenisation rule; the tables move (the dictionary's spellings
  and the list are rows of `tables/it/forms.tsv`), it-fr re-reduced, its pin and `pack_version`
  moved; the it-fr fixture — which the Italian golden runs over until change 16 — gains the
  accented words the `informale` page needs and two spelling rows standing in for D2 and D3, its
  manifest at the new version; the Italian golden re-blessed on the `informale` page and the
  version line.
- **English, Spanish, French and the six goldens do not move.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *The apostrophe that stands for an accent reads as the accented word*;
  MODIFIED — *Italian tokenisation pre-pass* (change 6): the apostrophe sentence, the version
  sentence (« `0.2.0` with this pre-pass » becomes « bumped with this pre-pass », the literal being
  stale since change 7) and the « an apostrophe that stands for an accent, and a closing quote »
  scenario; every other scenario kept.
- `lingua-data-packs`: ADDED — *An Italian spelling the dictionary gives only as a marked word
  reads as that word when the marked word is the commoner* and *The web's Italian spellings without
  their marks read as the words a reviewed list names*; both read change 9's tables, after which
  this change archives.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/tokenize.rs` (the apostrophe rule in Italian's words seam, on
  the rest after the elisions, with the lexicon; `is_elided`'s invariant widens to the accent
  spellings, which change 12's phrase gloss inherits), `analysis/mod.rs` (the version),
  `tests/italian_fixtures.rs`.
- `scripts/lingua-data/reduce-it-fr.py` (the dictionary's spellings, the reviewed list and the
  exception rows as literals with their reasons, the boundary check), `tables/it/forms.tsv`,
  `tables/it-fr/{pin.json, manifest.json, README.md}` (re-reduced), `SOURCES.md`;
  `scripts/lingua-data/measure/it-ud.sh`'s figures.
- `scripts/lingua-data/testdata/it-fr/{manifest.json, forms.tsv, gloss.tsv}` (the fixture the golden
  runs over); `crates/lingua-wasm/tests/italian_baseline.rs` and `baseline/it-fr.golden`;
  `apps/lingua-extension/test/packs.spec.ts` (Italian's version by its pattern).
- Untouched: the six pairs' tables, pins and goldens; the extension's bundle; the agent; the
  server; the site.
