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
read with the English Wiktionary's Italian section (the extract change 9 reduces), each word ranked by
its own string as change 9's D6 ranks lemmas, French's rules ported to Italian (design, *Measured*):
- **8,302 lemmas levelled**, A1 from rank 1 to 1,092 and C2 from 9,442 to 10,760 (French's committed
  table: 1–1,080 and 9,413–10,762); within that span left out: 906 written only as names, 1,519
  unknown to the section, 18 that only spell another word, 15 single letters that are no word.
- **Against the English level of each word's translation**, one script over the three languages
  (the first word of the English glosses that English's committed CEFR table levels): Italian's
  estimate agrees exactly for 23.1 % and within one level for 54.9 %; French's committed table, by
  the same script, 23.4 % and 54.9 %; Spanish's, 24.6 % and 56.1 %. Skipping a leading "to", "a",
  "an" or "the" in the gloss: Italian 25.3 % and 59.7 %, French 26.4 % and 62.6 %, Spanish 27.5 % and
  62.8 %. Of Italian's A1 words, 55.2 % translate to an English A1 word and 77.5 % to A1 or A2
  (French 58.2 % and 80.1 %, Spanish 55.0 % and 76.7 %).

Measured one way, Italian's estimate reads like French's and Spanish's, which readers already have.
(French's change 46 reported 28.3 % and 66.7 % by another method; the script above reproduces neither
that nor Spanish's 29.4 % and 65.9 %, so the figures compare only with each other.)

## What Changes

- **it-fr's reducer writes `tables/it/level.tsv`** (design D1, D2): Italian's commonest lemmas, in
  rank order, take English's level sizes — 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2,
  the constants fr-en's and es-fr's reducers hold, kept as constants of `reduce-it-fr.py`. A lemma
  takes no level when a CEFR list would leave it out, read from the Italian section — the source
  it-fr's forms come from, so the table does not wait for it-fr's glosses (change 16):
  - a word the section does not know (`km`) or knows only as a name (`milano`, `giovanni`, written
    there only with a capital);
  - a single character that is no word — every sense a letter's, a symbol's, a name's, an
    abbreviation's, or only another word's spelling (`b`, `k`, `r` « alternative spelling of er »,
    `à` « obsolete spelling of ha ») —, while `a`, `e` and `o` are A1 (`è` and `i` are no lemmas of
    their own: change 9 reads them as *essere* and *il*);
  - a word whose every sense only spells another, by French's prefixes (`the` « misspelling of tè »,
    `perchè`) — not a contraction or an apocope (`del`, `della`, `un` are A1);
  - a ranked lemma whose own form the forms table reads as another lemma: a guard, as French's is
    now — the builder files a level by the lemma's id and refuses such a lemma, and change 9 ranks
    none, so it is expected to leave none out.
  The acronym rows of change 11 (`NATO`, `USA`) have no rank, so the rank order never reaches them.
- **Every pack studying Italian says its levels are estimated**: it-fr's manifest carries
  `levels_estimated: true`, as es-fr's and fr-en's do; it-fr's NOTICE says so; it-en and it-es read
  the table as committed and say the same (D3).
- **Handed to change 16** (D4): until it-fr's glosses are committed, no pair's glosses are read; once
  change 16 commits them, the levels go to Italian's dictionary words alone, as French's go to
  French's since `refine-lingua-fr-en-glosses` (the reducer's optional `words`, which change 16
  passes), and the check that every Italian levelled lemma is a dictionary word lands with it. Until
  then `roma` (« a type of rice »), `marco` (the coin) and `maria` take a level through their
  lowercase senses.
- **Nothing to build in the core, the engine or the extension** (D5): the ladder already gives a
  non-English language whose levels are estimated English's typical vocabularies and says so; the
  extension already labels estimated levels from the pack's flag. Italian is shown by those paths
  once change 20 widens the extension's studied languages.
- **Nothing else moves**: the six pairs — tables, pins, packs, goldens — byte for byte; the Italian
  golden runs over its fixture pack, whose levels are hand-written, until change 16 hands it to the
  committed tables (D6); if change 16 lands first, this change re-blesses it.

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
  ranked lemma, the rows without a level; the Italian folder test widened by `level.tsv`);
  `crates/lingua-wasm/tests/languages.rs` (every Italian pack says its levels are estimated, as
  French's test does).
- Order: change 13 also edits `reduce-it-fr.py` and re-records it-fr's pin; either order, the second
  re-reduces on top of the first.
- Untouched: lingua-core, the builder, the extension, the six pairs.
- **Effort, against 1.5–3 ideal days**: 1.5–2.25 (design, *Effort*).
