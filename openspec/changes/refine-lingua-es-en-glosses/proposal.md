# refine-lingua-es-en-glosses — es-en's glosses read as meanings before English speakers read them

## Why

Change 23 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-english-card-wording`, merged with #800) read the es-en pack through the engine and the
English card, and listed what reads wrong and is data, not wording: its design's *Known data
defects*, "the input of the es-en reducer fix planned after this change". This is that fix. It is
outside the programme's 57 changes, and it has to land before change 34
(`enable-lingua-english-speakers`) ships es-en: the defects sit on the commonest words an English
speaker will open.

Measured on the committed `tables/es-en/` (31,876 glossed lemmas, 8,646 of the top 10,000) and on
the pinned extract's senses (design, *Measured*):

- **« su »** (rank 15) reads « apocopic form of suyo »; **« mi »** (22) « mu, the Greek letter Μ, μ;
  mi » — "my" is lost to a skipped pointer; **« tu »** (31) « yours, your »; **« muy »** (32) « much, a
  lot, far, way, many times; very; … », borrowed whole from « mucho ».
- **« lo »** (14) reads only its article: « him, you (formal), it, that » sits in a skipped pointer,
  « accusative of él and usted …; him, … »; **« nos »** (35) reads only two archaic senses, **« les »**
  (71) only « the (plural) ».
- **« como »** (17) opens on the Italian city of Como, twice, before « as ».
- **« venir »** (881) is glossed by two sense-group labels, « Senses relating to literal movement;
  Figurative senses », and none of its sixteen senses.
- **« ni »** (48), **« se »** (9), **« a »** (6) open a sense on the edition's own description
  (« Used when negating… », « A reflexive or reciprocal pronoun… ») in a capital, where 98.3 % of the
  edition's senses open in lower case.
- Ellipses written three ways (« not...anything », « both ... and », « either … or »), straight
  quotes beside curly ones, a source's sense number (« difference from sense 4 depends on
  context »), an example sentence carried after a line break.

Each is the English Wiktionary's page layout read as a meaning, or its typography read as written:
rules of the English edition can fix them without touching any other pair. Others are not the
rules' to fix — upstream text (« indiference »), an order no committed table can judge (« hasta »
opening on « even »), labels the packs do not carry (« o » « where », obsolete) — and are measured
and listed here for the owner or for a named follow-up, so that what ships with change 34 is known.

## What Changes

- **Nested senses read by their own gloss** (D2): a sense the edition nests under a sense-group
  label (« Figurative senses. »), a list's introduction (« places in Peru: ») or a pointer
  (« apocopic form of suyo », « prepositional form of se », « diminutive of casa » over « small
  house ») is read by its own gloss, not by its parent's; a parent that is a meaning (« to make »
  over « to create ») still glosses as today.
- **Shortened and respelled forms, and a pronoun's case forms, read as their meaning** (D3): a sense
  worded as an apocopic, apheretic, syncopic or prepositional form, or a pronunciation or eye-dialect
  spelling — known by that wording, never by a tag alone — is read as the meaning its pointer carries
  (« apocopic form of mío, my » → « my »; « apocopic form of cincuenta y uno (“fifty-one”) » →
  « fifty-one », its target named whole), else as its target's senses in the same part of speech,
  from a target of three letters or more, in its place — even when the word has other senses. A
  pronoun's case form that carries its meaning after a colon or a semicolon is read as that meaning
  (« lo » « him, you (formal), it, that »).
- **A function word does not open on a place** (D4): the proper-noun entries of a headword with an
  initial capital — not all capitals: an acronym's lines keep their place — go after the entries of
  a lower-case preposition, conjunction, pronoun, determiner or article spelled like it (« como »).
- **One English typography** (D5): the edition's descriptions in lower case (a closed list of
  openers: « used when negating… », never « The » before a capital nor the grade « A »), one ellipsis
  « … », straight double quotes paired as “ ”, no reference to a numbered sense, no text after a line
  break.
- **Where they live** (D1): a pre-pass of `reduce_edition_en.py` that `reduce-es-en.py` runs before
  the etymology merging — the English edition's rules, which only es-en loads. `reduce_common.py` is
  not edited: en-fr, es-fr and en-es are not re-pinned. The shared defects are named and left out
  (« etc » loses its period in every pair, D6).
- **es-en re-pinned alone at its snapshot** (D7): reduced again from
  `lingua-pack-sources-es-en-2026.10.08`, no new fetch, the pin's snapshot, studied record and
  sources byte for byte; coverage held to `FLOORS["es-en"]`.
- **Every moved line read** (D8): `baseline/es-en.golden` and `test/baseline/word-card-es-en.txt`
  re-blessed and each changed line reviewed in the pull request, the French rows' snapshot run as the
  gate and never re-blessed, with a before/after sample of every changed row of the top 10,000 for
  the owner.
- **Measured and decided, not fixed here** (D6): labels (obsolete, regional, register), the part of
  speech a row opens on, proper nouns before a common word, single letters borrowing an
  abbreviation's target, IPA, upstream typos and wording — each with its counts and where its fix
  belongs.
- **M20 not settled here** (D9): the long-parenthesis bound and the etymology merging (change 21,
  tasks 2.2 and 5.1) stay the owner's; this change composes with either answer.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *es-en's glosses read a Spanish word's meanings, not its page's layout*
  and *es-en's glosses are written in one English typography*. *Spanish is glossed in English from
  the English Wiktionary's Spanish section* (held by `add-lingua-pack-es-en`, not archived) is read as
  written: these rules are more of the English edition's, and its scenario *A setting of the English
  edition* (es-en re-pinned, en-fr's and es-fr's pins unchanged) holds for them. *A word that is only
  a form of another takes that word's gloss* holds as written: D3 lends only from a target of three
  letters or more, the bound it sets for every lending (design D3, *Three letters*).

## Impact

- **Products.** Cymbra Lingua only: `scripts/lingua-data` (`reduce_edition_en.py`,
  `reduce-es-en.py`, `test_reduce_editions.py`, `tables/es-en/`, its README, `SOURCES.md`),
  `crates/lingua-wasm/tests` (`baseline/es-en.golden`, `es_en_baseline.rs`'s shown-gloss constant),
  `apps/lingua-extension/test/baseline/word-card-es-en.txt`. ID, Music, Live, the back office and the
  site are untouched.
- **What moves**, measured on the real reduction with a prototype of the rules (design,
  *Measured*): 268 of es-en's 31,876 rows, 111 of the top 10,000 (250 / 104 glossed by their own
  senses, 18 / 7 borrowed from a pointer's target); 9 lemmas and 25 expressions gain a gloss, none
  loses one; 332 expressions change, nearly all for D5's lower case.
- **No other pair moves**: en-fr's, es-fr's and en-es's tables, pins, packs and goldens, and the
  French snapshot of rows, are byte for byte unchanged; `tables/es/` is untouched.
- **Nothing shown to anyone**: `packs.json` does not list es-en until change 34.
- **Order.** After changes 21 (the tables) and 23 (the golden, the snapshot, the defect list);
  before change 34, whose prerequisites list it. After change 38
  (`migrate-lingua-pack-sources-to-raw-dumps`, merged with #804), through its legacy-record path: 38
  re-pins no pair and keeps es-en's legacy `kaikki` record as recorded, and this change re-pins es-en
  from that same pinned extract with its `sources` byte for byte. 38 measured es-en's tables
  identical from the extract and from the English dump under today's rules; the implementation
  measures them again under these rules while kaikki still serves that dump, else es-en's next
  update names any difference. The implementation builds on 38's `SOURCES.md`,
  `tables/es-en/README.md` and `test_reduce_editions.py` as merged (design D9).
- **Not here.** M20's two settings (change 21); the shared fixes (« etc », a case-aware card for
  proper nouns); the studied side (which forms are lemmas, change 23's « al », « hecho », enclitics);
  the card's wording (changes 23, 33).
