# refine-lingua-en-es-glosses — en-es's glosses read as meanings before Spanish speakers read them

## Why

Change 24 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-spanish-card-wording`, merged with #805) read the en-es pack through the engine and the
Spanish card, and listed what reads wrong and is data, not wording: its design's *Known data
defects*, "the input of an en-es reducer fix after this change, which re-pins en-es alone". This is
that fix, the sibling of `refine-lingua-es-en-glosses` (row 23b, #801). It is outside the
programme's 57 changes, and it has to land before change 35 (`enable-lingua-spanish-speakers`) ships
en-es: the defects sit on the commonest words a Spanish speaker will open.

Measured on the committed `tables/en-es/` (21,965 glossed lemmas, 8,495 of the 10,000 commonest)
and on the pinned sources as `reduce-en-es.py` reads them (design, *Measured*):

- **« will »** (rank 37) opens on « Deseo, inclinación, disposición » — a sense the Spanish
  Wiktionary marks outdated — then two obsolete ones, and ends on « Apellido; Hipocorístico de
  William »: its « Voluntad, albedrío » and « Testamento » are crowded out. **« go »** (89) opens on
  the obsolete « Andar, marchar, caminar » before « Ir ».
- **« smith »** (1083) reads « Apellido; Herrero », **« mike »** (1285) two nicknames before « Micro,
  micrófono », **« small »**, **« white »**, **« king »** end on « Apellido »: a surname's or a given
  name's note, from the capitalised proper noun spelled like the word, on 61 common words' cards.
- **« a »** (5) reads « Un, una. A veces se omite en la traducción » on its row; **« stage »** (674)
  and **« unit »** (964) carry « .^([cita requerida]) », **« favor »** (1868) « [sentido del
  sustantivo] », **« hardcore »** « (definiciones [1,2]) ».
- **« her »**, **« my »**, **« its »**, **« their »**, **« that »** are headed « adjetivo » where en-fr's
  card says « déterminant ».
- **« orchestra »** (4805) reads « Orquesta, orquestra (disused) », **« hall »** (1121) « hall (hol) »,
  **« app »** (1134) « app (ap) »: the English Wiktionary's labels and respellings, carried with the
  Spanish words its translation tables list. **« lengthy »** (6076) reads « Largo; Largo; Largo;
  Largo », the Spanish « largo »'s four parts of speech read backwards.
- **« be »** (15) names the -ing form « participio presente » where the card's line says « forma en
  -ing » (M10); 32 rows quote with straight quotes (« it's » « ("ello") »), four write « ... ».

Each is the Spanish Wiktionary's page, or the translation tables' notes, read as a meaning: rules of
the Spanish edition and of en-es's reducer can fix them without touching any other pair. Others are
not the rules' to fix — a period lost by every pair (« etc »), labels the packs do not carry
(« do » « Follar », colloquial), an order no committed table can judge (« up » opening on
« Construido »), regional words read backwards (« sup » « KLK »), upstream text (« huh » « !Um¡ ») —
and are measured and listed here for the owner or for a named follow-up, so that what ships with
change 35 is known.

## What Changes

- **The edition's notes to its readers taken out** (D2): its maintenance templates (« ^([cita
  requerida]) », « ^([definición imprecisa]) »), a disambiguation note (« [sentido del
  sustantivo] »), a reference to numbered senses (« (definiciones [1,2]) »), the expansion notice
  (« Este lema en este idioma es ampliable… »), and a usage note written after the meaning, from a
  closed list of openers (« Un, una. A veces se omite… » → « Un, una »).
- **A name does not gloss the common word spelled like it** (D3): a capitalised proper noun's sense
  that only says the word is a surname or a given name (« Apellido », « Nombre de pila… »,
  « Nombre personal… », « Hipocorístico de… ») glosses no word that has an entry of its own in lower
  case — the condition of *An acronym does not gloss the word it is spelled like*. A word that is
  only a name keeps its notes (« wayne » « Apellido »).
- **Possessives and demonstratives are determiners** (D4): an adjective section the edition tags
  possessive or demonstrative (« her », « my », « its », « their », « that », « such ») reads as a
  determiner, as en-fr's card heads them.
- **Current senses first** (D5): the senses the edition marks obsolete or outdated go after the other
  senses of their entry, in their order — « will » opens on « Voluntad, albedrío », « go » on
  « Ir ». Nothing is left out; whether the labels show is the owner's (Q1).
- **One Spanish typography** (D6): one ellipsis « … », straight double quotes paired « » as the RAE
  advises, and M10's « forma en -ing » where an English entry names the -ing form « participio
  presente ».
- **The translation tables' words, without their translators' notes** (D7): a Spanish word the
  English Wiktionary labels disused is left out (« orquestra »); a note in a translation goes when
  it respells a loanword (« hall (hol) »), holds no Spanish word (« (despective) »), numbers a sense
  (« [4] ») or opens on « with »; read backwards, a Spanish word is listed once, under the part of
  speech the English word's readings name (« lengthy » « Largo », « seaman » « [sustantivo]
  Marinero »).
- **Where they live** (D1): `reduce_edition_es.py` for the Spanish Wiktionary's writing — the notes,
  the order of its senses, its typography — and `reduce-en-es.py` for which senses gloss an English
  word and for the translation tables. Both are read by en-es alone today (the pins' `reducer.files`);
  `reduce_common.py` is not edited: en-fr, es-fr and es-en are not re-pinned. The shared defects are
  named and left out (« etc » loses its period in every pair, D8).
- **en-es re-pinned alone at its snapshot** (D9): reduced again from
  `lingua-pack-sources-en-es-2026.10.08`, no new fetch, the pin's snapshot, studied record and sources
  byte for byte; coverage held to `FLOORS["en-es"]`.
- **Every moved line read** (D10): `baseline/en-es.golden` and `test/baseline/word-card-en-es.txt`
  re-blessed and each changed line reviewed in the pull request, the French rows' snapshot run as the
  gate and never re-blessed, with a before/after sample of every changed row of the top 10,000 for
  the owner.
- **Measured and decided, not fixed here** (D8): « etc », the labels, the part of speech a row opens
  on, the quantifiers' heading, the inverted table's regional words, the direct table's repeated word
  across the English word's own parts of speech, upstream wording — each with its counts and where
  its fix belongs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *en-es's glosses read an English word's meanings, not its page's
  notes*, *en-es's glosses are written in one Spanish typography* and *en-es's translation-table
  glosses hold the translators' words, not their notes*. *English is glossed in Spanish from the
  Spanish Wiktionary's English section and the English Wiktionary's translation tables* (held by
  `add-lingua-pack-en-es`, not archived) is read as written: these are more of the Spanish edition's
  and the pair's rules, its fallbacks still the tables' words in their order, at most three per part
  of speech. *A gloss holds none of the Wiktionary's notes to its readers* and *An acronym does not
  gloss the word it is spelled like* hold as written for every pair; this change adds en-es's own
  cases beside them rather than widening a requirement every pair holds.

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (`reduce_edition_es.py`, `reduce-en-es.py`,
  `test_reduce_editions.py`, `tables/en-es/`, its README, `SOURCES.md`), `crates/lingua-wasm/tests`
  (`baseline/en-es.golden`), `apps/lingua-extension/test/baseline/word-card-en-es.txt`. ID, Music,
  Live, the back office and the site are untouched.
- **What moves**, measured on the real reduction with a prototype of the rules (design, *Measured*):
  295 of en-es's 21,965 rows, 149 of the top 10,000 (159 / 121 glossed by the Spanish Wiktionary,
  78 / 18 by the direct table, 58 / 10 by the inverted one); the first sense of 140 / 63. One lemma
  loses its gloss — « malign » (33,911), whose one translation is labelled disused — and none gains
  one; 64 expressions change and 2 lose theirs (« deep end », « with both hands », disused words
  alone). Coverage stays 93.0 / 85.0 / 71.7 %: no lemma of the top 20,000 moves in or out.
- **No other pair moves**: en-fr's, es-fr's and es-en's tables, pins, packs and goldens, and the
  French snapshot of rows, are byte for byte unchanged; `tables/en/` is untouched (read, not written).
- **Nothing shown to anyone**: `packs.json` does not list en-es until change 35.
- **Order.** After changes 22 (the tables) and 24 (the golden, the snapshot, the defect list); before
  change 35, whose prerequisites list it. After change 38 (`migrate-lingua-pack-sources-to-raw-dumps`,
  merged with #804): 38 re-pins no pair and keeps en-es's records as its first update wrote them, and
  this change re-pins en-es from those same derived files, its `sources` byte for byte; en-es's move
  to the dumps stays its next update's (38's per-pair rule), whose report names the 186 glosses 38
  measured (design D11).
- **Not here.** Change 22's floor (task 5.1, the owner's); the shared fixes (« etc », a case-aware
  card for proper nouns); the card's wording (changes 24, 33); the studied side (`tables/en/`, en-fr's
  reduction).
