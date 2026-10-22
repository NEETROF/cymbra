# add-lingua-italian-levels — Italian's levels, estimated from word frequency (I8)

## Why

Change 14 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1, decision
I8. A level table drives the reader's level declaration, the words presumed known below it, the
statistics' ladder and « Renforcer un niveau ». A pack without one falls back to the frequency
slider. Change 9 commits Italian's forms and ranks in `tables/it/` and leaves the levels to this
change.

No Italian CEFR word list can be shipped: the *Profilo della lingua italiana* is not free, and no
openly licensed Italian list was found. Decision **I8** (settled 2026-10-21) settles it as M7 did
French and D1 did Spanish: **Italian levels are estimated** from word frequency and labelled as such.

The derivation is French's (matrix change 46). Measured on a prototype — wordfreq's Italian list
read with the English Wiktionary's Italian section (the extract change 9 reduces), surface words in
wordfreq's order rather than change 9's lemma ranks, which do not exist yet:
- **8,302 lemmas levelled**, A1 from rank 1 to 1,531 and C2 to 17,784; within that span 2,849 ranked
  words left out — 2,613 unknown to the section or written there only with a capital (names: `roma`,
  `giovanni`), 212 that only spell another word, 24 single letters that are no word.
- **Against the English level of each word's translation** (the first word of its English glosses
  that English's committed CEFR table levels): Italian's estimate agrees exactly for 23.1 % of 6,756
  lemmas and within one level for 54.9 % (French on its prototype ranks: 28.3 % and 66.7 %; Spanish's
  committed estimate: 29.4 % and 65.9 %). Of Italian's A1 words, 55.2 % translate to an English A1
  word and 77.5 % to A1 or A2 (French 51.6 % and 76.5 %, Spanish 49.6 % and 73.8 %).

The A1 band reads like French's and Spanish's; the wider spread above it is the prototype's
surface-word ranks (a verb's forms rank apart from its infinitive), which change 9's lemma ranks
fold — re-measured on them at implementation (task 2.1).

## What Changes

- **it-fr's reducer writes `tables/it/level.tsv`** (design D1, D2): Italian's commonest lemmas, in
  rank order, take English's level sizes — 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2,
  the constants fr-en's and es-fr's reducers hold, kept as constants of `reduce-it-fr.py`. A lemma
  takes no level when a CEFR list would leave it out, read from the Italian section — the source
  it-fr's forms come from, so the table does not wait for it-fr's glosses (change 16):
  - a word the section does not know or knows only as a name;
  - a single character that is no word (`b`, `k`), while `a`, `e`, `è`, `o`, `i` are A1;
  - a word the section gives only as another word's spelling (`perchè`, change 10's kind);
  - a ranked lemma whose own form the forms table reads as another lemma: the builder keys a level
    by the form, so its level would land on that other word;
  - an acronym row of change 11 (`NATO`, `USA`): no rank, no level.
- **Every pack studying Italian says its levels are estimated**: it-fr's manifest carries
  `levels_estimated: true`, as es-fr's and fr-en's do; it-fr's NOTICE says so; it-en and it-es read
  the table as committed and say the same (D3).
- **Handed to change 16** (D4): once it-fr's dictionary words exist, the levels go to them alone,
  as French's go to French's (the reducer's optional `words`, which change 16 passes), and the check
  that every Italian levelled lemma is a dictionary word lands with it.
- **Nothing to build in the core, the engine or the extension** (D5): the ladder already gives a
  non-English language whose levels are estimated English's typical vocabularies and says so; the
  extension already labels estimated levels from the pack's flag. Italian is shown by those paths
  once change 20 widens the extension's studied languages.
- **Nothing else moves**: the six pairs — tables, pins, packs, goldens — byte for byte; the Italian
  golden runs over its fixture until change 16, whose levels are the fixture's own (D6).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *Italian's estimated levels*. *A level reaches the lemma it is
  written for* (French's change 46, language-neutral) holds Italian's table as it holds the others';
  this change archives after it, after changes 5 and 9 whose tables it reads, and after changes 10
  and 11 whose spelling and acronym rows it leaves without a level.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `scripts/lingua-data/reduce-it-fr.py` (the bands, the rules, `estimated_levels`, the report
  line), `test_reduce_it_fr.py`.
- `scripts/lingua-data/tables/it/level.tsv` (new), `tables/it-fr/{manifest.json, NOTICE, pin.json,
  README.md}`; `SOURCES.md`.
- `crates/lingua-pack/tests/committed_tables.rs` (Italian's levels: 8,302, English's sizes, each a
  ranked lemma, the rows without a level).
- Untouched: lingua-core, lingua-wasm, the builder, the extension, every golden, the six pairs.
- **Effort, against 1.5–3 ideal days**: 1.5–2.25 (design, *Effort*).
