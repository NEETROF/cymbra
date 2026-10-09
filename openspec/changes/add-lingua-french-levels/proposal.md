# add-lingua-french-levels — estimated CEFR levels for French

## Why

Change 46 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en for English speakers, fr-es for Spanish speakers). A level table
drives the reader's level declaration, the words presumed known below it, the statistics' ladder
and « Renforcer un niveau ». A pack without one falls back to the frequency slider. Change 43
(`add-lingua-french-forms-tables`) commits French's forms and ranks in `tables/fr/` and leaves the
levels to this change (its D10 and D12: « the ranks (estimated, M7) »).

No French CEFR list can be shipped. FLELex, the French list of the CEFRLex project, is licensed
CC BY-NC-SA 4.0: non-commercial, which the pack builder's licence guard refuses. No openly licensed
French CEFR word list was found. Decision **M7** of the programme (settled 2026-10-09) settles it
the way D1 of the Spanish programme settled Spanish: **French levels are estimated** from word
frequency and labelled as such, and the owner sends a licence request for FLELex together with
ELELex's.

The derivation is Spanish's, and so is its accuracy where the truth is known: given English's
8,302 CEFR lemmas in their own rank order, English's level sizes agree with the lists for **39.8 %**
of them and within one level for **82.6 %**; the mean true level of each estimated level rises
from 1.67 at A1 to 5.03 at C2 (measured again on the committed English tables, the study's
figures). On French itself, measured with open data only on change 43's prototype ranks:

- **Against the English level of each word's translation** (the first word of its English
  Wiktionary glosses that CEFR-J or Octanove levels), French's estimate agrees exactly for
  28.3 % of 5,950 lemmas and within one level for 66.7 %; Spanish's committed estimate, the
  control, by the same method on the same edition: 29.4 % and 65.9 % of 6,140. Of French's A1
  words, 51.6 % translate to an English A1 word and 76.5 % to A1 or A2 (Spanish: 49.6 % and 73.8 %).
- **On running text** — UD PUD, the same 1,000 sentences in the three languages, names and numbers
  left out — French's A1 covers 78.4 % of the words, Spanish's 76.6 %, English's lists 65.8 %; a reader declaring B1 is
  presumed to know 86.2 % of the French words, 84.7 % of the Spanish, 78.0 % of the English.

French's levels read like Spanish's, which readers already have.

## What Changes

- **fr-en's reducer writes `tables/fr/level.tsv`.** French's commonest lemmas, in rank order, take
  English's level sizes: 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2, the constants
  es-fr's reducer holds. A lemma takes no level when a CEFR list would leave it out, read from the
  English Wiktionary's French section — the source fr-en's forms already come from, so the table
  does not wait for fr-en's glosses (change 48) and does not move when they land:
  - a word the section does not know (`the`, `etc`, `km`) or knows only as a name (`paris`,
    `france`);
  - a single letter that is no word (`b`, `e`: a letter's name, a symbol, an abbreviation), while
    `à` and `y` are A1;
  - a word the section gives only as another word's spelling (`etre`, `etat`, `parceque`);
  - a ranked lemma whose own form the forms table reads as another lemma (`donnée`, read as
    *donner*): the builder keys a level by the form, so its level would land on *donner*.

  On change 43's prototype ranks: 8,302 lemmas, A1 from rank 1 to 1,080, C2 from 9,419 to 10,768;
  2,466 ranked lemmas left out within that span (1,390 unknown to the section, 1,022 names, 33
  spellings, 17 letters, 4 lemmas whose own form reads as another).
- **Every pack studying French says its levels are estimated**: fr-en's manifest carries
  `levels_estimated: true`, as es-fr's and es-en's do; fr-es (change 49) reads the table as
  committed and says the same. fr-en's NOTICE says the levels are estimated.
- **A level reaches the lemma it is written for.** The committed-tables checks build every pair's
  pack and require each lemma of its studied language's level table to carry that level. The four
  committed pairs pass today (8,302 levels each, measured); French's table on change 43's prototype
  ranks would not without the last rule above. Change 43's implementation in progress ranks no such
  lemma any more (measured on its branch), so the rule stays as a guard.
- **Nothing to build in the core, the engine or the extension.** `levelLadder` already gives a
  non-English language whose levels are estimated English's frozen typical vocabularies
  (`ENGLISH_TYPICAL_VOCABULARY`) and says so; the extension already labels estimated levels from
  the pack's flag. French is shown by those paths once change 52 widens the extension's studied
  languages — « Estimated French level » in English, « Nivel de francés estimado » with « MCER »
  in Spanish (M19); the French interface, whose scale is « CEFR », never shows French levels, a
  French-native reader not studying French.
- **Nothing else moves.** en-fr, es-fr, es-en and en-es — tables, pins, packs, goldens — byte for
  byte; the French golden too: it runs over its fixture until change 48 (if 48 merges first, this
  change re-blesses it, naming the level probes — design D7).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *French's estimated levels* (the derivation, what takes no level,
  the flag on every pack studying French) and *A level reaches the lemma it is written for* (the
  check over every committed pair). No requirement is modified. *French's forms and frequencies*,
  which this table reads, is change 43's, and *A French invariance baseline runs beside the
  English and Spanish ones*, whose golden this change leaves alone, change 39's: both are in
  `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua's data pipeline only:
  - `scripts/lingua-data/` — *changed*: `reduce-fr-en.py` (change 43's) and its tests, the
    re-reduced `tables/fr/level.tsv`, `tables/fr-en/manifest.json`, `NOTICE`, `pin.json` and
    `README.md`, the French rows of `SOURCES.md`; *consumed*: change 43's forms, ranks and pinned
    sources, unchanged; `reduce_common.py` and every other reducer untouched.
  - `crates/lingua-pack` — *new*: committed-tables tests (French's level table, the check over every
    pair); the builder unchanged.
  - `crates/lingua-wasm` — *new*: one test of the French ladder over the committed tables; the
    engine unchanged.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the extension, the Apple
  host app and the agent plugin are untouched.
- **Sources and licences**: none new. The table derives from the sources fr-en already pins — the
  English Wiktionary's French section (CC BY-SA 4.0 + GFDL) and wordfreq (CC BY-SA 4.0). No FLELex
  data is read, measured against or committed.
- **Size.** Each French pack grows by its levels section, one byte per lemma of its pool: fr-en
  1,239,104 → 1,299,139 B on the prototype (+60,035 B, the flag included), as Spanish's packs carry
  theirs; well within the 5 MiB budget.
- **Order.** After change 39 and change 43's implementation (required: `reduce-fr-en.py` and
  `tables/fr/` reach `main` with it); beside change 45 in either order, the second to merge
  re-reducing fr-en on top of the first (both edit `reduce-fr-en.py`); before change 48 (planned:
  its hand-over then keeps the golden's level probes).
- **Release.** Silent. No package lists a French pair before change 52.
- **Owner.** The licence request for FLELex, sent with ELELex's (`[manual]`); the design says what a
  granted licence would change: a data change, not a format change.
- **Effort, against 1.5–3 ideal days**: 1.75–2.75 (design, *Effort*).
