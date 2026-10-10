# read-lingua-french-hyphenated-and-misaccented-spellings — the web's hyphenated words without their marks and its wrong accents, read by a reviewed list

## Why

Row 43f of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. Change 43e (`read-lingua-french-web-unmarked-spellings`) reads the web's
French written without its marks (`tres`, `deja`, `francais`) through a reviewed list, and left two
kinds out, which the owner sent to a reviewed list of their own on 2026-10-10 (43e's Q5):

- **Hyphenated words written without their marks**: `peut-etre`, `c'est-a-dire`, `lui-meme`,
  `grand-mere`, `la-bas`. 43e's rows cannot reach them. The pre-pass reads a hyphenated run as one
  token whole only when the pack lists it; otherwise the run is a compound the pack does not know, its
  lemma the written run, no gloss, no level — « peut-etre » is `peut-etre`, its pieces read as
  *pouvoir* and `etre` —, an inversion when its last pieces are pronouns (« celle-la » reads `celle`
  and `la`, the article), or split at an elision (« c'est-a-dire » reads `ce` and an unknown
  `est-a-dire`). A row for a piece cannot help: the compound keeps its written lemma and no gloss, and
  the pieces `a` and `la` are words of their own.
- **Wrong accents**: `trés`, `déja`, `aprés`, `pére`, `agé`, `oú`. Each is ranked today as a word of
  its own (`déja` 4,125, `trés` 4,725), unglossed, levelless: a reader of « c'est trés bien, je l'ai
  déja vu » meets two unknown words.

## What Changes

- **The hyphenated candidates, from data** (design D1). wordfreq cannot count a hyphenated spelling:
  it splits it at the hyphen and answers its rarer piece (`peut-etre` 4.75 is `etre`'s 4.77;
  `c'est-a-dire` 5.86 is `a`'s order). Of the 275 hyphenated forms of `tables/fr/` written with marks,
  266 have a spelling without them that is no form yet and that the French section names nowhere; 227
  are forms of dictionary words (39 are names: `ile-de-france`, `jesus-christ`). Their frequency is estimated from the
  accented compound's count in 6.36 M words of French corpora (UD, Tatoeba, Wikipedia, Gutenberg,
  change 39's pages), shifted by the web's measured share of unaccented spellings — the median, over
  43e's rows, of wordfreq's Zipf for the spelling less the corpora's Zipf for its accented word
  (−1.59) —, and held to 43e's settled threshold, Zipf 2.5: **18 rows**, and `celles-la` with its
  paradigm (`celui-la`, `celle-la`, `ceux-la`): **19** — `peut-etre`, `la-bas`, `grand-mere`,
  `lui-meme`, `grand-pere`, `apres-midi`, `moi-meme`, `petit-dejeuner`, `celui-la`, `c'est-a-dire`,
  `celle-la`, `au-dela`, `eux-memes`, `ceux-la`, `soi-meme`, `la-dessus`, `apres-demain`,
  `belle-mere`, `celles-la`. Each row is the spelling of the whole run: the pack then lists it, and
  the pre-pass reads it whole, as it reads `peut-être` (no core code moves).
- **The misaccented candidates, from data** (D2). Every word wordfreq rates at Zipf ≥ 2.5 that carries
  a mark, whose letters are a table form's and whose marks are not: 66 the section names nowhere and
  the tests keep, `çe` (its letters are `ce`'s; a row would read `cé`) and `abbès` (« Sidi Bel Abbès »
  twice in two) out: **65 rows** — 12 at Zipf ≥ 3 (`déja`, `trés`, `aprés`, `egalité`, `étre`, `oú`,
  `electricité`, `mére`, `eté`, `théatre`, `verité`, `agé`), 53 from 2.5 (`méme`, `desolé`, `pére`,
  `arreté`, `frére`, `probléme`, `oû`). Each reads as the form whose marks it keeps in place, then the
  commoner: `agé` → *âgé*, not *âge*; `elève` → *élever*, `elevé` → *élevé*. Left out, each group
  listed in the design: a spelling another language writes more often (`á`, `lá`, `ã`, `cà`,
  `accés`, `procés`, `succés`: Portuguese, Catalan, Italian), names or no dictionary word (`jérome`,
  `andrés`, `malgrès`), as common as their word (`pêle`, `elysées`), and 13 the dictionary itself
  gives (`problême`, `précisement`, `sureté`, `réglement`).
- **The rows join 43e's table** (D3): two literals in `reduce-fr-en.py`, `WEB_HYPHENATED` (19) and
  `WEB_MISACCENTED` (65), merged into `UNMARKED_SPELLINGS` with one shared reason each, read as 43e
  reads `WEB_SPELLINGS` — a form of its word alone, no entry making it another word's form —, under
  43e's boundary check: the reduction fails on a row the section gives a meaning of its own or gives
  only as another word's spelling.
- **A spelling carries its word's readings** (D4), as 43e's do; a misaccented one those of the forms
  whose marks it keeps in place (`arreté` the past participle of *arrêter*, not the present
  `arrête`), a hyphenated one those of its form (`grand-mere` the feminine singular noun): 109
  readings for 64 of the 84 rows.
- **What moves, measured on a prototype stacked on 43e's** (D5): the 64 misaccented rows that were
  ranked leave the ranks (2 in the top 5,000: `déja`, `trés`; 12 in the top 20,000), 64 lemmas enter
  at the cut's end; `forms.tsv` +95, `grammar.tsv` +139, `lexical.tsv` +12, glosses +14 in fr-en;
  **`level.tsv` byte for byte**; fr-en coverage 93.8 / 87.2 / 76.8 % → 93.9 / 87.2 / 76.8 %; fr-es +7
  glosses, `trève` « Tregua » gone (now read through *trêve* « Tregua »), coverage unchanged at one
  decimal; the packs 2,547,436 → 2,547,127 B and 1,980,036 → 1,979,249 B; UD French-PUD content lemmas
  96.40 → 96.41 % (« tâtonnant », an entering lemma), GSD's test section 95.94 → 96.02 % (« trés »
  ×3).
- **In running text** (D6): the 84 rows occur 72 times in the corpora, 68 as the word the row names;
  4 false (a name, « Monsieur Faché »; Walloon « béta » and « oû » ×2). The hyphenated rows read no
  name or foreign word anywhere.
- **The goldens** (D7): `fr-en.golden` and `fr-es.golden` move only their pack line and their two
  vocabulary universes (26,570 → 26,582); six phrase probes and two grammar probes are added
  (« peut-etre qu'il viendra », « il l'a fait lui-meme », « c'est-a-dire demain », « ma grand-mere
  habite la-bas », « c'est trés bien, je l'ai déja vu », and « elle l'a fait elle-meme », the record
  of a spelling no row can name; `arreté`, `grand-mere`).
- **Not here, recorded** (D8): `elle-meme`, the owner's example, and `toi-meme`, `vous-meme`,
  `nous-memes`, `elles-memes` — their accented words (`elle-même` 266 times in the corpora) are no
  forms of the tables, because GSD lemmatises them as *lui-même* and *eux-mêmes* and change 43 ranks a
  compound by its GSD lemma; a row needs a ranked word, so they wait for a change to the ranks. The 13
  misaccented spellings the dictionary gives, which 43d's rule (marks dropped) does not read. Wrong
  accents inside hyphenated words (4 occurrences in the corpora).
- **en-fr, es-fr, es-en and en-es cannot move**: the rows, their check and their readings live in
  `reduce-fr-en.py`, which only fr-en's rule digest names; over the prototype every lingua-wasm test
  passed, the four other pairs' goldens, `cross_native`, `parity`, `card_gloss_language` and `deck`
  unmoved.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *French's hyphenated words without their marks and its wrong accents
  read as the words a reviewed list names* (19 hyphenated rows read whole, 65 misaccented rows, each a
  spelling the section names nowhere; the readings of the forms whose marks a spelling keeps; the
  never-list; levels unmoved; nothing else moves).

No requirement is modified. The new one extends change 43e's *The web's French spellings without
their marks read as the words a reviewed list names* (open), relies on change 40's listed-run rule
(open) and moves what changes 43, 45, 46, 48, 48b and 49 write from the forms — all open, so it says
so rather than MODIFY them; they are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua's data and its tests only:
  - `scripts/lingua-data` — *changed*: `reduce-fr-en.py` (the two literals, the check over them, the
    readings rule for a spelling with marks), `test_reduce_fr_en.py`, `test_reduce_editions.py`,
    `tables/fr/` (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `lexical.tsv`), `tables/fr-en/`
    (`gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json`, `README.md`), `tables/fr-es/`
    (`gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json`, `README.md`), `SOURCES.md`.
  - `crates/lingua-pack` — `tests/committed_tables.rs`: the rows read as their words in both French
    packs.
  - `crates/lingua-wasm` — six phrase probes and two grammar probes in the French scenario; the
    probe-count test +2; `fr-en.golden` and `fr-es.golden` re-blessed.
  - `apps/lingua-extension` — `word-card-fr-en.txt` and `word-card-fr-es.txt` gain eight cards each,
    their specs' counts +2 grammar and +6 phrase probes; no code.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host app
  and the agent plugin are untouched; French's analyser version does not move.
- **Release.** Silent: no package lists a French pair. It ships after change 43e (whose table and
  check it extends) and, like 43e, before change 52 lists French.
- **Compatibility.** No reader holds a French pack; nothing to migrate.
- **Effort, against no programme estimate (outside the 57)**: 0.75–1.25 ideal days (design,
  *Effort*), plus the owner's reading of the 84 rows.
