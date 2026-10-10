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

## What Changes

- **A candidate list, measured** (design D1): every French form of the tables written with marks,
  its marks dropped; kept when wordfreq attests the spelling at Zipf ≥ 3 (once per million words),
  the French section gives it no meaning of its own and does not give it as a spelling of its
  accented word (change 43d's kind), its accented word is a French dictionary word, it is less than
  half as frequent as that word, and English does not write it more often than French does. 563
  unmarked spellings at Zipf ≥ 3 are named nowhere by the dictionary; 151 pass, 6 more read as the
  commoner of two words (`ecoute`, *écoute* or *écouté*), 2 are left out as doubtful (`dela`,
  `nes`), and `tete`, which the dictionary names only as a form of `teter` (« alternative form of
  téter ») and change 43d hands over, joins them: **156 rows proposed, every one listed with its
  frequencies in the design for the owner to read** (task 0.1). Left out, each group listed with its
  frequencies: 103 that English writes more often (`video`, `hotel`, `general`, `these`), 160 whose
  accented word is a name or no dictionary word (`jesus`, `quebec`, `eric`), 48 as common as their
  accented word or more (`the`, `peter`, `media`), 95 hyphenated words wordfreq cannot count
  (`peut-etre`), and below the threshold 307 more (`deteste`, `appele`, `leger`).
- **The rows join change 43b's table** (D2): one literal of `spelling → word` pairs, merged into
  `UNMARKED_SPELLINGS` with one shared reason, each read as 43b reads its rows — a form of its word
  alone. The reduction fails when the French section gives a row's spelling a meaning of its own or
  gives it only as a spelling of its accented word, which keeps this list and 43d's apart and catches
  a dictionary update.
- **A spelling carries its word's readings** (D3): the readings of the forms of its word it writes
  without their marks — `recu` the past participle of *recevoir*, `arrete` those of `arrête` and
  `arrêté` together, `etais` the imperfect of *être* —, so the card says what the form is, as it does
  for the accented one. 279 readings for 145 rows (the 11 adverbs have none); change 43b's rows gain
  theirs too (`age`, `ages`, `ca`: 3).
- **What moves, measured on a prototype that reproduces 43b's tables byte for byte** (D4):
  - `tables/fr/`: the 156 spellings read as their words; the 155 that were ranked leave the ranks
    (12 in the top 5,000: `meme` 1,018, `tres` 1,697 …), 155 lemmas enter at the cut's end
    (59,843–59,997), every rank between moves up by 1 to 155; `forms.tsv` 124,101 → 124,298,
    `grammar.tsv` 125,177 → 125,576, `lexical.tsv` 26,484 → 26,517 (33 entering lemmas);
    **`level.tsv` byte for byte** (no spelling was a levelled word); the tag pool byte for byte;
  - fr-en: 34 glosses gained (entering lemmas: `désarçonner`, `enceint` …), none lost, `mwe.tsv` byte
    for byte; coverage 93.6 / 86.9 / 76.3 → 93.8 / 87.2 / 76.7 %; the pack 2,537,588 → 2,540,636 B;
  - fr-es: re-reduced on the new `tables/fr/`: one gloss out — `tres` « Tres », the French
    Wiktionary's Cuban guitar, until now shown for « tres » —, 11 in, every other byte for byte;
    coverage 83.0 / 70.7 / 56.7 → 83.1 / 70.8 / 57.0 %; the pack 1,971,114 → 1,973,142 B;
  - UD French-PUD 99.13 / 96.41 / 99.90 → 99.13 / 96.40 / 100.00 % (« Etant » now *être*;
    « Evidemment », which PUD lemmatises unaccented, now *évidemment*); GSD's test section 98.89 /
    95.86 / 99.72 → 98.91 / 95.94 / 99.72 % (« Eglise », « Equipe »).
- **In running text** (D5): on the corpora at hand (UD, Tatoeba, Wikipedia, Gutenberg, change 39's
  pages; 6.8 M words) the rows occur 159 times; 128 are the accented word (60 of 63 in Tatoeba's
  user-written sentences), 12 names (« Noel Murray », « Shimon Peres »), 18 other languages' words in
  French text (Catalan « tres », Walloon « tot », Italian « Serie A »), 1 an English title; 98 are
  capitals French writes without accents (« l'Eglise », « Etant donné »), which a row reads by name.
- **The goldens** (D6): `fr-en.golden` and `fr-es.golden` move 4 of their 221 probes each — the pack
  line, « apres » and « genial » on the `informel` page read as *après* and *génial*, the two
  vocabulary universes 26,484 → 26,517 —; five phrase probes are added (« c'etait deja tres bien »,
  « une facon de parler francais », « j'ai recu ta reponse », « L'Ecole est fermée. », « regarde
  cette video ») and two grammar probes (`recu`, `arrete`). The word-card snapshots gain those seven
  cards (27 and 26 lines) and move nothing else.
- **Not here, recorded** (D7): hyphenated words (`peut-etre`, `c'est-a-dire`), spellings with a wrong
  accent (`trés`, `déja`, `aprés`: 14 at Zipf ≥ 3), the English homographs and the threshold's lower
  band — each an open question for the owner.
- **en-fr, es-fr, es-en and en-es cannot move**: the rows and the readings live in
  `reduce-fr-en.py`, which only fr-en's rule digest names; no shared or edition module, no core code
  and no other pair's tables change; their goldens and `cross_native`, `parity`,
  `card_gloss_language` and `deck` passed unmoved over the prototype.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The web's French spellings without their marks read as the words a
  reviewed list names* (the owner's rows beside 43b's, each a spelling the dictionary gives no meaning
  and does not give as its accented word's spelling, a form of its word alone carrying its word's
  readings; the never-list; levels unmoved; nothing else moves).

No requirement is modified. The new one extends change 43b's *A French spelling without its marks
reads as the word a reviewed table names* (open), adds readings change 45's *French's word grammar*
does not list (open), and moves what changes 43, 46, 48, 48b and 49 write from the forms — all open,
some held by change 52 too — so it says so rather than MODIFY them; they are in `archiveAfter`, with
change 41 (whose cascade looks every token up lowercased) and change 44 (whose expressions a spelling
meets).

## Impact

- **Products.** Cymbra Lingua's data and its tests only:
  - `scripts/lingua-data` — *changed*: `reduce-fr-en.py` (the rows, the boundary check, the
    readings), `test_reduce_fr_en.py`, `test_reduce_editions.py`, `tables/fr/` (`forms.tsv`,
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
- **Release.** Silent: no package lists a French pair. It ships with or after change 43b (whose
  table it extends) and, like 43b, before change 52 lists French.
- **Compatibility.** No reader holds a French pack, so no status, card or level names a spelling of
  the list; nothing to migrate.
- **Effort, against no programme estimate (outside the 57)**: 1–1.75 ideal days (design, *Effort*),
  plus the owner's reading of the list.
