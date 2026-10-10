# refine-lingua-fr-es-expressions — fr-es's expressions read where French text meets them

## Why

Change 48c of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-fr-en-translated-expressions`) measured fr-en's expressions on French text and found two
defects in fr-es on the way (its design, *Found while measuring*, and its open question 4): « sur le
champ de bataille » meets « sur le champ » "En el acto", and fr-es's French Wiktionary table gives
the key of « crème brûlée » to its « crème brulée ». The owner decided on 2026-10-10: fr-es's
expression defects get a change of their own. This is that change, outside the programme's 57, after
`refine-lingua-fr-es-glosses` (49b), whose reviewed lists it extends.

The owner asked for the same classes as fr-en's to be looked for on fr-es's committed `mwe.tsv`: a
shorter idiom swallowing the start of a longer compound noun, post-1990 spellings keyed apart, and the
partitive « de l' » readings. This change ran the engine's own phrase gloss over the same French
corpora as 48c (1,060,904 words in 51,790 lines: UD French-GSD, Gutenberg, Wikipedia, Tatoeba and the
baseline's pages) on the pinned fr-es pack, reproduced byte for byte from its sources (design,
*Measured*):

- fr-es meets **41,549 expressions** there. Of 200 drawn at random and read in their sentences,
  **126 read right, 7 partly, 67 wrong**. The partitive is the largest class by far: « de la »,
  glossed « Algo de, un poco de, una cierta cantidad de », is met 8,757 times — a fifth of all
  matches — and read wrong in all 41 of its sampled matches (« le maire de la ville »).
- The swallowing class is small: « sur le champ » meets 6 times, 4 of them the start of « champ de
  bataille ». Change 44f (`choose-lingua-overlapping-expressions`) does not reach it — its rule moves
  the last word of an expression built on a verb —, measured with and without it.
- Post-1990 spellings: 11 table headwords, 3 of them with no traditional spelling beside them
  (« crème brulée », « clé de voute », « ognon de roscoff »), which a page writing the traditional
  spelling meets under the rectified name.
- The rest is what 48c found in fr-en: words met as they are where the source names a set phrase
  (« mon cœur » "Amor", « à plus » "Hasta luego", « être à » "Ser de, pertenecer"), a form of another
  word met as the headword (« de fait » "De hecho" met by « de faire » 202 times of 223), and wrong
  translations (« se réveiller » "Despedirse", « se passer » "Pasarse").

## What Changes

Every rule is fr-es's own, in `reduce-fr-es.py`: no shared or edition module is edited, so en-fr,
es-fr, es-en, en-es and fr-en do not move, and fr-es's glosses of words do not move either (design D1).

- **A reviewed list of expressions left out** (D2): `LEFT_OUT` names the 65 expressions read wrong in
  at least 60 % of their matches, out of the 822 met five times or more (92.6 % of all matches), each
  with its reason — whatever source glosses it, so a later source cannot bring it back. By reason:
  the partitive « de la » (8,757 matches); « sur le champ », which swallows « champ de bataille »
  (6); 41 met as plain words where the source names one use or a set phrase (1,984: « être à »,
  « venir de » "Acabar de" before a place, « histoire de », « mon cœur », « et bien », « c'est moi »
  "A usted", « la ferme » "Cállate el hocico"); 12 forms of another word met as the headword (461:
  « de fait », « étant donné », « la tienne » met by « la tenue », « qui sait » by « qui savait »); 9
  wrong translations (178: « se passer » "Pasarse", « se moquer » "Jugar", « en ce moment » "Al
  momento, al instante, inmediatamente"); a name (« de gaulle » "Apellido"). 11,393 matches, 27.4 %.
- **Four slips corrected instead** (D3): where another source glosses the expression right, 49b's
  `CORRECTIONS` sets the wrong one aside or corrects its spelling — « se lever » « Levantarse » (the
  section's « Estar de pie uno » set aside), « se réveiller » « Despertarse » (the direct table's
  « Despedirse » set aside), « grand public » « Gran público », « effet spécial » « Efecto especial ».
- **Written as French pages write it** (D4): a translation-table expression listed only in its
  post-1990 spelling is written in its traditional spelling, the commoner one by wordfreq, when
  French's forms table reads both as the same word — « crème brûlée », « clé de voûte », « oignon de
  roscoff » —; the 8 listed beside their traditional spelling stay, for change 44e to name by the
  page; an elided piece written apart from its word is written against it (« huile d' olive », the
  inverted table's, is « huile d'olive », the direct table's row).
- **Measured and left** (D5): no filter by source (the inverted table reads 6 right of 11 sampled,
  but « plus de », « un autre », « se trouver » are among its commonest); no section-key rule (4 keys
  shared with the section, its headword winning all four); no self-translation rule (Spanish writes « statu
  quo », « a priori », « foie gras » alike); « en fait » and « de nouveau », wrong in 51 % and 53 %
  of their matches through the forms they are met by (« en faisant », « de nouveaux »), kept — a rule
  of the matcher, not of fr-es's data —, as is « ce que » met inside « est-ce que » (250 of 940).
- **Measured, the result** (D6): `mwe.tsv` 12,174 → 12,108 rows; on the corpora 41,549 → 30,410
  expression matches (38.3 → 28.0 per 1,000 words), 8,052 selections moved; a fresh sample of 100
  reads **91 right, 5 partly, 4 wrong**; of 50 matches the change newly shows, 48 read better than
  before and none worse. With change 44f the same 8,005 selections move, and 44f then moves 13 fr-es
  selections where it moved 86 (« être à » and « venir de », which it repaired, are left out here).
  The pack 1,969,484 → 1,968,375 B.
- **What moves** (D7): fr-es's `mwe.tsv`, `manifest.json` (`pack_version`), `pin.json` (rule digest
  and pack), `README.md`; `fr-es.golden` (its `pack` line) and six phrase probes added to the French
  scenario — « sur le champ de bataille », « une crème brûlée » (48c's, added by whichever lands
  first), « le maire de la ville », « Elle se leva. », « Il vient de Paris. », « de l'huile
  d'olive » —, which `fr-en.golden`, `word-card-fr-en.txt` and `word-card-fr-es.txt` gain as added
  lines, none of their lines moving. `gloss.tsv`, `senses.tsv`, coverage, `tables/fr/`, fr-en,
  en-fr, es-fr, es-en and en-es do not move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *fr-es leaves out, by name, the expressions read wrong where French text
  meets them* and *fr-es writes its expressions as French pages write them*. Nothing is modified. The
  requirements this change builds on are open changes': French's forms table
  (`add-lingua-french-forms-tables`), the keys the builder gives French headwords
  (`add-lingua-french-expression-keys`), fr-es's sources, pin and floor (`add-lingua-pack-fr-es`) and
  fr-es's reviewed corrections (`refine-lingua-fr-es-glosses`, *fr-es corrects, by name, the slips its
  sources write*, read as written: four more rows). All four are in `archiveAfter`.
- `lingua-analysis`, `lingua-browser-extension`: none. The matcher and the card read expressions as
  the pack holds them; what they show moves with the data.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce-fr-es.py` (`LEFT_OUT`, four `CORRECTIONS` rows, the
    spelling pass) and `test_reduce_fr_es.py`, `test_reduce_editions.py` (fr-es's digest alone moves),
    `tables/fr-es/` (`mwe.tsv`, `manifest.json`, `pin.json`, `README.md`), `SOURCES.md`; *consumed*:
    `reduce_common.py`, `reduce_edition_es.py`, `tables/fr/forms.tsv`, unchanged.
  - `crates/lingua-wasm/tests` — `support/french.rs` (six phrase probes), `baseline/fr-es.golden`
    and `baseline/fr-en.golden` re-blessed.
  - `apps/lingua-extension/test/baseline/word-card-fr-es.txt` and `word-card-fr-en.txt` re-blessed,
    their specs' probe count following; no source, bundle or package changes.

  ID, Music, Live, the back office, the site, the backend, the engine and lingua-core, the Apple host
  app and the agent plugin are untouched.
- **Licences, sources, snapshot.** None change: fr-es is reduced again from
  `lingua-pack-sources-fr-es-2026.10.10`, nothing fetched.
- **Release.** Silent: no package lists fr-es before change 52, which ships it after this change, as
  it ships fr-en after 48c (the owner's answer to 48c's question 2).
- **Order.** After 49 and 49b (on `main`). Either side of 44e, 44f, 48c, 43d and 43e: each re-pins
  fr-es or adds probes, and whichever lands second re-reduces or re-blesses on the other (design D8).
- **Effort**: 1–1.75 ideal days (design, *Effort*).
