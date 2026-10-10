# refine-lingua-fr-en-expressions — fr-en's own expressions read where they are met

## Why

fr-en glosses French expressions from the English Wiktionary's French section (change 48,
`add-lingua-pack-fr-en`; its rules for expressions refined by 48b, `refine-lingua-fr-en-glosses`).
While measuring the French Wiktionary's translations, change 48c
(`add-lingua-fr-en-translated-expressions`, proposal #888, implementation #893) sampled the section's
own expressions where a reader meets them: **73 of 100 matches read right, 26 wrong** — « il m'a
sauvé » glossed "I'm going", « en faisant » meeting « en fait » "actually", « de l'équipe » read as
the partitive "some". It left them to the owner (its open question 3). **The owner decided on
2026-10-10: the section's own noisy expressions get a cleanup change of their own.** This is that
change, row 48d of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57.

Measured on `main` (`2d6ec09f`: fr-en's committed tables, the pack builder and the engine's own phrase
gloss) over 48c's 1,060,904 words of French text — UD French-GSD's sentences, four Gutenberg novels,
French Wikipedia, Tatoeba and change 39's pages, 51,790 selections:

- fr-en meets **60,361 expressions**. A sample of **300 matches** (seed 4900), each read in its
  sentence: **221 right, 13 partly, 66 wrong** — 48c's 73 % again.
- **The wrong ones are classes**, not noise: « de l' » glossed by the partitive alone ("some") where
  it means "of the" (29 of 30); a sense the section labels — informal, slang, Quebec, Internet,
  derogatory — met as plain words (« pas de » "without", « sa mère » "(so) fucking", « j'aime » "a
  like"); a headword met through another word's form (« en fait » on « en faisant », « de fait » on
  « de faire », « de nouveau » on « de nouveaux »); common words met as they are where the headword
  names a set phrase or a cry (« au pied » "to heel!", « première fois » "the first instance of sexual
  intercourse"); a sense that only notes a form (« puis-je » "an interrogative form of je peux").
- **They are concentrated**: 50 of the 66 wrong are expressions met at least 100 times; the 734
  expressions met ten times or more carry 85 % of the matches.
- **No structural rule separates them** — a labelled sense, a headword keyed through another word's
  reading, « de » + one word: each removes the wrong for a like share of the right (design D5). Read
  where they are met, they separate: of the 734, **111 read wrong in most of their contexts**.

## What Changes

- **A reviewed list** (design D1): `LEFT_OUT` gains the 111 section expressions read wrong in most of
  the contexts they are met in, each with its reason, as 48b's and 48c's rows give theirs — 47 a
  labelled sense met as plain words, 21 a headword met through another word's form, 41 words met as
  they are, 2 colloquial spellings met as standard ones (« m'a » "I'm going", « t'es »): 7,197 of the
  60,361 matches.
- **A sense that only notes a form is no meaning** (D2): 48b's rule for an expression's senses that
  only point gains five wordings — « compound of … and … », « (an) interrogative form of », « a form
  of avoir … in the third-person », « for non-idiomatic uses », « equivalent to “…” », « used only
  in the … ». « j'ai » is glossed "I've, I have"; « a-t-il », « puis-je » and « colin-tampon » keep
  no sense and leave (149 matches).
- **« de l' » reads "of the" first** (D3): the section glosses it by the partitive alone; it is « de
  la » or « du » before a vowel, and both are glossed "of the" first. It is lent « de la »'s first
  sense, before its own: "of the; some; the singular prevocalic partitive article". 29 of 30 sampled
  matches turn right, « de l'eau » keeps its partitive.
- **Measured, the result** (D4): `mwe.tsv` 17,635 → 17,521 rows; the pack 2,536,702 → 2,535,598 B;
  60,361 → 53,624 matches (7,377 lost, 640 gained where a shorter right expression is no longer
  hidden — « en faisant du bruit » now meets « faire du bruit »). A fresh sample of 100 (seed 4903)
  reads **95 right, 2 partly, 3 wrong**.
- **Rules measured and rejected** (D5): every sense labelled (it removes « au début », « en ligne »,
  « s'il te plaît »: 3 right per wrong in the tail), a headword keyed through another word's reading,
  « de »/« à » + one word, a pronoun first, and 48c's repeating translation applied to the section
  (it removes « coup d'état », « crème brûlée »).
- **48c's table stays out of them** (D6): a `LEFT_OUT` name and a pointer-only headword are not read
  from the French Wiktionary's table (48c's D2): 49 of the 111 have a table row (« en fait » "in
  fact", « de rien » "you're welcome") that would otherwise come back, met the same way.
- **What moves** (D7): fr-en's `mwe.tsv`, `manifest.json` (`pack_version`), `pin.json` (`reducer`,
  `pack`), README; `fr-en.golden` its `pack` line and « de l'eau »'s gloss; the extension's
  `word-card-fr-en.txt` « de l' — of the ». fr-es, en-fr, es-fr, es-en and en-es cannot move: one
  file of fr-en's own rules is edited, which no other pair loads.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *fr-en's own expressions are read where they are met*. Nothing is
  modified: the requirements this change builds on are open changes' — the keys the builder gives
  French headwords (`add-lingua-french-expression-keys`), fr-en's glosses (`add-lingua-pack-fr-en`),
  its rules for expressions (`refine-lingua-fr-en-glosses`) and the table that completes them
  (`add-lingua-fr-en-translated-expressions`), all four in `archiveAfter`.
- `lingua-analysis`, `lingua-browser-extension`: none. The matcher and the card read expressions as
  written; what they show moves with the data.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce-fr-en.py` (`LEFT_OUT`, the pointer wordings, the
    lent sense) and `test_reduce_fr_en.py`, `tables/fr-en/` (`mwe.tsv`, `manifest.json`,
    `pin.json`, `README.md`); *consumed*: `reduce_common.py`, `reduce_edition_en.py`, unchanged.
  - `crates/lingua-wasm/tests/baseline/fr-en.golden` re-blessed.
  - `apps/lingua-extension/test/baseline/word-card-fr-en.txt` re-blessed; no source or bundle.

  ID, Music, Live, the back office, the site, the backend, the engine and lingua-core, the Apple host
  app and the agent plugin are untouched. No source is fetched: fr-en is re-reduced at its pin.
- **Licences.** None change.
- **Release.** Silent: no package lists fr-en before change 52.
- **Order.** After 48c (`add-lingua-fr-en-translated-expressions`): its proposal is merged, its
  implementation (#893) is not. This change is implemented on top of it, its figures re-measured on
  48c's tables (task 1.1); before change 52 lists fr-en.
- **Effort**: 1.5–2.5 ideal days (design, *Effort*).
