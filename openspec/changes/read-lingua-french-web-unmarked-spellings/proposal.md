# read-lingua-french-web-unmarked-spellings — the web's French written without its accents, read by a reviewed list

## Why

Row 43e of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes: the follow-up change 43b (`read-lingua-french-ca-as-ca-cedilla`) recorded as
its Q4 and the owner settled on 2026-10-10.

Many people write French on the web without its accents, cedilla or diaeresis: `francais`, `tres`,
`deja`, `meme`, `facon`, `garcon`, `recu`, `apres`. The English Wiktionary's French section, French's
dictionary, names none of these spellings, so French's tables read each as a word of its own —
ranked, unglossed, levelless — and a reader of « c'etait deja tres bien » meets three unknown words
with no gloss (`etait`, `deja`, `tres`). Change 39's own corpus writes « jte rappelle apres le
boulot » and « c'etait vraiment genial !! ». wordfreq, which reads the web, subtitles and social
media, rates `meme` at Zipf 4.77, `tres` 4.52, `apres` 4.33, `francais` 4.07, `deja` 4.06: common
words, 2–3 % as frequent as their accented word.

The owner decided (2026-10-10): **link them to their accented word one by one, in a reviewed list —
never a blanket rule**, which would read `video`, `hotel` and `these` (*thèse*) as French words they
are not on a page, and never `ou`/`où`, `a`/`à`, `la`/`là`, `sur`/`sûr`, `du`/`dû`, `des`/`dès`,
`cote`, `tache`, `pale`, `foret`, `aine`, which are words of their own. A change of its own, after
change 43b, whose reviewed table (`UNMARKED_SPELLINGS` in `reduce-fr-en.py`) is the mechanism, and
beside the parallel proposal `read-lingua-french-dictionary-unmarked-spellings` (43d), which reads the
spellings the dictionary itself gives (`etre`, « obsolete spelling of être »; `etat`): this change
takes only the spellings the dictionary does **not** name.

The owner answered this proposal's questions on 2026-10-10 (design, *Settled by the owner*): the
longer list, down to Zipf 2.5; every spelling English writes more often left out; rows that are also
names or other languages' words kept, `dela` and `nes` out; a spelling two words share read as the
commoner; hyphenated words and wrong accents a reviewed list of their own, later. On change 43d the
owner sent `sure`, `surs`, `sures` and `entrainant` to this list's review.

## What Changes

- **A candidate list, measured** (design D1): every French form of the tables written with marks,
  its marks dropped; kept when wordfreq attests the spelling at Zipf ≥ 2.5 (Q1), the French section
  gives it no meaning of its own and does not give it as a spelling of its accented word (change
  43d's kind), its accented word is a French dictionary word, it is less than half as frequent as that
  word, and English does not write it more often than French does (Q2). Of the 1,218 spellings at
  Zipf ≥ 2.5 the dictionary names nowhere, 440 pass, 22 more read as the commoner of two words (Q4),
  `dela` and `nes` stay out (Q3), and `tete` and `tetes`, which the dictionary names only as forms of
  `teter` (« alternative form of téter ») and change 43d hands over, join them: **464 rows** — 156
  at Zipf ≥ 3 (`meme`, `tres`, `apres`, `francais`, `deja`), 308 from 2.5 (`deteste`, `appele`,
  `leger`, `biere`) —, every one listed with its frequencies in the design. Left out, each group
  listed: 239 that English writes more often (`video`, `hotel`, `these`, `role`, `mere`,
  `premiere`), 308 whose accented word is a name or no dictionary word (`jesus`, `quebec`), 97 as
  common as their accented word or more (`the`, `peter`, `media`), 136 hyphenated words wordfreq
  cannot count (`peut-etre`), and 19 of the dictionary's own spellings (43d's).
- **The owner's four exceptions** (D2): `sure`, `surs`, `sures` → *sûr* and `entrainant` →
  *entraîner*, words the section gives a sense of their own (*sur* « sour », the participle of the
  post-1990 `entrainer`), each a row with its reason: in the corpora 38 of their 41 occurrences are
  the accented word (« Je suis sure que… » 31 times), 3 the sour adjective. 468 rows in all.
- **The rows join change 43b's table** (D2): one literal of `spelling → word` pairs, merged into
  `UNMARKED_SPELLINGS` with one shared reason, each read as 43b reads its rows — a form of its word
  alone —, the exceptions as rows of their own beside 43b's. The reduction fails when the French
  section gives a row of the web's list a meaning of its own or gives it only as a spelling of its
  accented word, which keeps this list and 43d's apart and catches a dictionary update. An entry
  pointing at a row's spelling follows the row (`patisseries`, « plural of patisserie », reads as
  *pâtisserie*).
- **A spelling carries its word's readings** (D3): the readings of the forms of its word it writes
  without their marks — `recu` the past participle of *recevoir*, `arrete` those of `arrête` and
  `arrêté` together, `etais` the imperfect of *être* —, so the card says what the form is, as it does
  for the accented one. 867 readings for 442 rows (the 26 adverbs and prepositions have none); change
  43b's rows gain theirs too (`age`, `ages`, `ca`: 3).
- **What moves, measured on a prototype that reproduces the committed tables byte for byte** (D4):
  - `tables/fr/`: the 468 spellings read as their words; the 465 that were ranked leave the ranks
    (12 in the top 5,000: `meme` 1,018, `tres` 1,697 …; 165 in the top 20,000), 465 lemmas enter at
    the cut's end (59,533–59,997), every rank between moves up by 1 to 465; `forms.tsv` 124,101 →
    124,669, `grammar.tsv` 125,177 → 126,295, `lexical.tsv` 26,484 → 26,570 (89 entering lemmas in,
    `surs`, `sures` and `entrainant` out); **`level.tsv` byte for byte** (no spelling was a levelled
    word); the tag pool byte for byte;
  - fr-en: 101 glosses gained (entering lemmas: `désarçonner`, `enceint`, `mocassin` …), 3 lost
    (`surs` and `sures` « sour », `entrainant`'s pointer), `mwe.tsv` byte for byte; coverage 93.6 /
    86.9 / 76.3 → 93.8 / 87.2 / 76.8 %; the pack 2,537,588 → 2,548,334 B;
  - fr-es: re-reduced on the new `tables/fr/`: four glosses out — `tres` « Tres » (the French
    Wiktionary's Cuban guitar, until now shown for « tres »), `comite` « Cómitre », `expres` and
    `reviser`, the last two now read through *exprès* and *réviser* —, 39 in, every other byte for
    byte; coverage 83.0 / 70.7 / 56.7 → 83.1 / 70.8 / 57.0 %; the pack 1,971,114 → 1,980,699 B;
  - UD French-PUD 99.13 / 96.41 / 99.90 → 99.13 / 96.40 / 100.00 % (« Etant » now *être*;
    « Evidemment », which PUD lemmatises unaccented, now *évidemment*); GSD's test section 98.89 /
    95.86 / 99.72 → 98.91 / 95.94 / 99.72 % (« Eglise », « Equipe »); no row of the lower band stands
    in either.
- **In running text** (D5): on the corpora at hand (UD, Tatoeba, Wikipedia, Gutenberg, change 39's
  pages; 6.8 M words) the 468 rows occur 270 times; 217 are the accented word (80.4 %; 132 of 138 in
  Tatoeba's user-written sentences), 16 names (« Noel Murray », « Shimon Peres », « Depeche Mode »),
  36 other languages' words in French text or the sour adjective (Occitan « Lou Tresor dóu
  Felibrige » ×10, Catalan « tres », « Cette pomme est sure »), 1 an English title. **The 308 rows from
  Zipf 2.5 add 19 false readings in 70 occurrences** (4 foreign names, 15 other languages' words, ten
  of them one Occitan title).
- **The goldens** (D6): `fr-en.golden` and `fr-es.golden` move 4 of their 221 probes each — the pack
  line, « apres » and « genial » on the `informel` page read as *après* and *génial*, the two
  vocabulary universes 26,484 → 26,570 —; five phrase probes are added (« c'etait deja tres bien »,
  « une facon de parler francais », « j'ai recu ta reponse », « L'Ecole est fermée. », « regarde
  cette video ») and two grammar probes (`recu`, `arrete`). The word-card snapshots gain those seven
  cards (27 and 26 lines) and move nothing else; the lower band moves no probe.
- **Not here, recorded** (D7): hyphenated words (`peut-etre`, `c'est-a-dire`) and spellings with a
  wrong accent (`trés`, `déja`, `aprés`: 14 at Zipf ≥ 3) — a reviewed list of their own each, later
  (the owner); `entrainer` read as a word of its own, found while measuring, for 43d.
- **en-fr, es-fr, es-en and en-es cannot move**: the rows, the check, the redirect and the readings
  live in `reduce-fr-en.py`, which only fr-en's rule digest names; no shared or edition module, no
  core code and no other pair's tables change; their goldens and `cross_native`, `parity`,
  `card_gloss_language` and `deck` passed unmoved over the prototype.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The web's French spellings without their marks read as the words a
  reviewed list names* (the owner's rows beside 43b's, each a spelling the dictionary gives no meaning
  and does not give as its accented word's spelling, or one of the owner's four exceptions; a form of
  its word alone carrying its word's readings; the never-list; levels unmoved; nothing else moves).

No requirement is modified. The new one extends change 43b's *A French spelling without its marks
reads as the word a reviewed table names* (open), adds readings change 45's *French's word grammar*
does not list (open), and moves what changes 43, 46, 48, 48b and 49 write from the forms — all open,
some held by change 52 too — so it says so rather than MODIFY them; they are in `archiveAfter`, with
change 41 (whose cascade looks every token up lowercased) and change 44 (whose expressions a spelling
meets).

## Impact

- **Products.** Cymbra Lingua's data and its tests only:
  - `scripts/lingua-data` — *changed*: `reduce-fr-en.py` (the rows, the exceptions, the boundary
    check, the redirect, the readings), `test_reduce_fr_en.py`, `test_reduce_editions.py`, `tables/fr/` (`forms.tsv`,
    `freq.tsv`, `grammar.tsv`, `lexical.tsv`), `tables/fr-en/` (`gloss.tsv`, `senses.tsv`,
    `manifest.json`, `pin.json`, `README.md`), `tables/fr-es/` (`gloss.tsv`, `senses.tsv`,
    `manifest.json`, `pin.json`, `README.md`), `SOURCES.md`; *consumed*: `reduce_common.py`,
    `reduce_edition_en.py`, `reduce-fr-es.py`, unchanged.
  - `crates/lingua-pack` — `tests/committed_tables.rs`: the spellings read as their words in both
    French packs.
  - `crates/lingua-wasm` — five phrase probes and two grammar probes in the French scenario; the
    probe-count test 31 → 33; `fr-en.golden` and `fr-es.golden` re-blessed.
  - `apps/lingua-extension` — `word-card-fr-en.txt` and `word-card-fr-es.txt` gain seven cards, their
    specs' counts 92 → 94 and 42 → 47; no code.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host app
  and the agent plugin are untouched; French's analyser version does not move.
- **Release.** Silent: no package lists a French pair. It ships after change 43b (merged, whose
  table it extends) and, like 43b, before change 52 lists French.
- **Compatibility.** No reader holds a French pack, so no status, card or level names a spelling of
  the list; nothing to migrate.
- **Effort, against no programme estimate (outside the 57)**: 1.25–2 ideal days (design, *Effort*),
  plus the owner's reading of the list.
