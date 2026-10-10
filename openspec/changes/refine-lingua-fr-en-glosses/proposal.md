# refine-lingua-fr-en-glosses — fr-en's glosses read as French meanings, and French's dictionary words hold no names, before English speakers read French

## Why

Change 48 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-pack-fr-en`, implemented with #855) committed fr-en's glosses from the English
Wiktionary's French section through the English edition's rules and 23b's `read_as_meanings`, and
listed what still reads wrong (its design's D7, *Known data defects*) as « the input of
`refine-lingua-fr-en-glosses`, outside the 57, before change 52 ships fr-en — as 23b was before 34 ».
This is that change. Change 52 (`enable-lingua-french`) lists it among its prerequisites. Change 51
(`add-lingua-french-word-card`) adds the card's view of the same rows: `pas` opening on « step »
before the negation, `son` on « sound », `leur` on « (to) them », `le` ending on « a surname from
Vietnamese ».

The owner read change 48's sample and defects (task 6.1) and decided, on 2026-10-10:
1. fr-en's floor stays 91.9 / 85.1 / 74.4 % (`gloss_coverage.py`): this change must not take fr-en
   under it.
2. **Names are not words to learn.** fr-en is French's reference pair, so every lemma it glosses is
   a French dictionary word — counted by the vocabulary estimate (a universe of 30,095) and kept as a
   word by change 41's names rule. 3,586 of them (914 of the top 10,000) are glossed by a name's
   senses alone: `france`, `paris`, `lyon`, `françois`, `durand` « a surname ». They stop being
   French's dictionary words.
3. **No level without a gloss.** 40 levelled lemmas have no fr-en gloss (`parce`, `quant`, `x`,
   `pme`, `stp`, `rsa`…): a card seeded from their level carries none. They lose their level.
4. The French Wiktionary's English translations for expressions are not part of this change.

Then each defect of change 48's list is measured on the committed tables and fixed or left, as 23b
did for es-en (design, *Measured*). The owner read this proposal the same day, settled its five
questions — names out of English's and Spanish's dictionary words later, in a change of their own;
the treebank reorders only a function word's row or a row opening on a name; « female equivalent
of » is no word's meaning; `ca` read as `ça` in a forms-table change of its own; the rows still
opening on a name accepted until a card reads the token's capital — and approved the golden's
re-bless before the implementation (design, *Settled by the owner*). On the committed rows (30,055
glossed lemmas):

- **« des »** (rank 6) reads « of the; some, the feminine partitive article », borrowed from `de la`:
  its own pointers carry « some » and « of the, from the, some ». **« mieux »** (116) loses « better »,
  **« moins »** (81) « less, fewer », **« ouais »** (371) « yeah, yep, yes », and the expression
  **« il y a »** « there is, there are »: each sits on a pointer the shared rules skip.
- **« pas »** (9) opens on « step, pace, footstep », **« son »** (29) on « sound », **« leur »**
  (46) on « (to) them », **« marche »** (404) on the department of France: the page's entry order.
- **« le »** (2) ends on « a surname from Vietnamese », **« on »** (19) on « a village in Luxembourg ».
- `et des`, `que de`, `sur ce`, `et si`, `un coup` are expressions whose one sense needs a context
  their key does not hold: « du pain et des œufs » meets `et des` « or thereabouts, and change ».
- `à priori`, `à postériori`, `et cétéra` and three expressions on `sur son trente-et-un` meet
  no expression: change 48 left their post-1990 pointers out, and the forms table keys them apart from
  their traditional spelling.
- The page's notes to its reader: « see usage notes » (`en`, `dans`, `ne`), « (all senses) » (98
  rows), « in its various senses », « (Folk etymology: …) », a citation inside `liberté`, a sense
  number « (2) »; descriptions in a capital outside 23b's list (`que` « Substitutes for… », `il`
  « Impersonal subject, it »); « etc » without its period (117 rows).

## What Changes

- **Every rule is fr-en's own** (design D1): `reduce-fr-en.py` gains a pre-pass, `read_as_french`,
  after 23b's `read_as_meanings` and before the etymology merging, a post-pass for « etc. », two
  expression rules, French's dictionary words and the levels' new rule. `reduce_common.py` and
  `reduce_edition_en.py` are not edited: en-fr, es-fr, es-en and en-es do not move, es-en included,
  which reads the same edition.
- **Names out of French's dictionary words** (D2): fr-en's reduction writes `tables/fr/lexical.tsv`
  itself — its glossed lemmas less those whose every sense run is a proper noun's (3,581) — and
  `pack_sources.py split` files it; the lemmas stay glossed, so coverage does not move. fr-en's pack
  now carries a lexical table (+7.5 KB with the levels). The vocabulary estimate's universe goes from
  30,095 to 26,486 words, and change 41's names rule, as written, sets `Paris`, `Durand`, `Lyon`,
  `Jean-Pierre`, `Saint-Étienne` aside on the baseline's `noms` page (11 names; `Lot`, `Aube`,
  `Orange`, `Nice` keep their cards). The lingua-pack check of a studied folder accepts the
  reference's glossed lemmas or those less its names-only lemmas, nothing in between.
- **A French level only for a dictionary word** (D3): the level rule leaves out a lemma French's
  dictionary words do not list, English's sizes kept: 45 lemmas lose their level (36 unglossed, 9
  glossed as names alone), 45 at C2's end gain one, 122 move one band up.
- **A pointer that carries its meaning** (D4): for a word, when it names a degree of comparison, a
  synonym, a plural or a contraction (« des » « some; of the, from the, some », « mieux » « better;
  best; … », « ouais »), never a female equivalent (`directrice` keeps « director; school
  principal »); for an expression, any pointer (« il y a » « there is, there are; ago »). 40 rows
  (18 of the top 10,000), 12 lemmas and 155 expressions gained; eleven expressions it would make of
  no meaning (`à le`, `de le`, `l'a`, `j'suis`…) left out by name (the owner, 2026-10-10).
- **The part of speech a row opens on, by UD French-GSD** (D5): fr-en's pin already holds GSD's
  training and development sections; a function word's row (`ADP`, `DET`, `PRON`, `CCONJ`, `SCONJ`,
  `PART` and, measured, `ADV`) or a row the page opens on a name opens on the part of speech they
  read the word as at least ten times and twice as often as the page's first, never a proper noun's:
  `pas`, `son`, `leur`, `bien`, `quand`, `pendant`; `marche`, `réunion`, `somme` no longer open on a
  place. 13 rows; a noun, verb or adjective ahead of another stays (`ferme` « firm », `mort`
  « dead »).
- **No name under a function word** (D6): `le`, `on`.
- **Expressions** (D7): the five above left out by name (`LEFT_OUT`, each with its reason: of 204
  « et des » in GSD's text, none means « or thereabouts »); a post-1990 spelling keyed apart lends its
  traditional spelling's gloss (6 expressions).
- **The page's notes and typography** (D8): the notes out (106 rows), fr-en's description openers in
  lower case (14 rows), citations (2), sense numbers (2), « etc. » (117 rows, 7 expressions).
- **Measured and left** (D9): an acronym's pointers (the cure loses `svp` « please »; `ca`, settled
  as a spelling of `ça`, is a forms-table change of its own), the 217 rows still opening on a proper
  noun (accepted until a card reads the token's capital), sentence-case definitions, labels (23b's
  Q1), es-en's own notes, the shared « etc »; names out of English's and Spanish's dictionary words,
  a change of its own.
- **fr-en re-reduced at its pin** (D10): 291 rows change (117 of the top 10,000), 12 lemmas gain a
  gloss, none loses one; 26 expressions change, 161 are gained, 5 left out; coverage 93.6 / 86.9 /
  76.3 % against the 91.9 / 85.1 / 74.4 floor; `tables/fr/` moves in `lexical.tsv` and `level.tsv`
  alone; the pack 2,527,222 → 2,537,386 B.
- **The golden re-blessed, every move read** (D11): `fr-en.golden` moves on 47 of its 147 probes,
  each with its cause; `french_baseline.rs`'s names assertions follow D2; the owner approved the
  re-bless on 2026-10-10, before the implementation, and reads the moved rows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *fr-en's glosses read a French word's meanings in the order French uses
  them*, *fr-en's glosses carry none of the page's notes* and *A French level is given only to a
  French dictionary word*. MODIFIED *A pack's dictionary words do not depend on its glosses* (French's
  dictionary words leave out the lemmas fr-en glosses as names alone) and *A studied language's tables
  are kept once* (the check accepts either set, as the reference's reduction writes it); both are
  archived requirements no open change holds. The requirements this change builds on are open
  changes': fr-en's (`add-lingua-pack-fr-en`), French's tables and reference pair
  (`add-lingua-french-forms-tables`), French's levels (`add-lingua-french-levels`, implemented, not
  archived: its requirement is not modified, the level rule is ADDED beside it), the names rule
  (`add-lingua-french-analysis`) and the pre-pass fr-en reads first (`refine-lingua-es-en-glosses`):
  all five are in `archiveAfter`. Sentences of 46, 48, 49 and 52 that this change makes true of their
  time are listed for their archives (design, *Settled by the owner*, Q6).
- `lingua-analysis`, `lingua-knowledge-model`: none. Change 41's names rule and the vocabulary
  estimate read dictionary words as written; what they show moves with the data.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce-fr-en.py` (the rules, the dictionary words, the
    levels' order) and `test_reduce_fr_en.py`, `pack_sources.py` (`split` files a reference's own
    `lexical.tsv`) and `test_pack_sources.py`, `tables/fr-en/` (`gloss.tsv`, `senses.tsv`, `mwe.tsv`,
    `manifest.json`, `pin.json`, `README.md`), `tables/fr/lexical.tsv` and `level.tsv`, `SOURCES.md`;
    *consumed*: `reduce_common.py`, `reduce_edition_en.py`, `gloss_coverage.py`, unchanged.
  - `crates/lingua-pack` — *changed*: `src/tables.rs` (a studied folder's dictionary words: all the
    reference's glossed lemmas, or all but its names-only ones), `tests/committed_tables.rs`
    (French's dictionary words and levels, fr-en's lexical section).
  - `crates/lingua-wasm/tests` — *changed*: `french_baseline.rs` (the names on the `noms` page),
    `support/french.rs` (its doc), `baseline/fr-en.golden` re-blessed.
  - ID, Music, Live, the back office, the site, the backend, the engine and lingua-core, the
    extension's source and snapshots, the Apple host app and the agent plugin are untouched.
- **What does not move.** en-fr, es-fr, es-en and en-es — tables, pins, packs, goldens, the
  extension's snapshots — byte for byte: no file of their rule digests is edited, `split` writes
  English's and Spanish's dictionary words as before, and the builder is unchanged. `tables/fr/`'s
  forms, ranks, readings and tag pool byte for byte. The French interface: nothing in the extension
  changes, and no package lists fr-en before change 52.
- **What moves.** fr-en's tables, manifest (`pack_version`) and pin (rule digest, pack);
  `tables/fr/lexical.tsv` (30,055 → 26,486 words) and `level.tsv` (212 rows); `fr-en.golden` (47 of
  147 probes, design D11).
- **Release.** Silent: no package lists fr-en before change 52.
- **Order.** After 48 (required), on main with 41, 43, 45 and 46; before 52, which waits for it.
  Change 49 (fr-es), if it lands first, reads French's dictionary words and levels from these
  tables once this change re-reduces it; change 51's snapshots, if they land first, are re-blessed
  here (design D12).
- **Effort**: 2–3.5 ideal days (design, *Effort*).
