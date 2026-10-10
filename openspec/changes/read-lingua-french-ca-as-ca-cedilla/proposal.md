# read-lingua-french-ca-as-ca-cedilla — « ca » without its cedilla reads as `ça`

## Why

Row 43b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes: a follow-up of change 43 (`add-lingua-french-forms-tables`, French's forms
table) that change 48b (`refine-lingua-fr-en-glosses`) handed over.

French written on the web often drops the cedilla of « ça » (« ca va », « c'est ca », « Ca fait du
bien. »). Today French's tables read that `ca` as a word of its own: rank 144, level A1, a French
dictionary word glossed « board of directors » in fr-en — the English Wiktionary's French `CA`
(conseil d'administration, chiffre d'affaires, courant alternatif, comptable agréé), lowercased like
every form, beside `ca`, « abbreviation of circa ». A reader of « ouais bon on verra, tkt pas, ca va
aller » (change 39's own corpus) is shown « board of directors »; in fr-es the word has no gloss at
all. Change 48b measured it (its D9) and the owner settled its Q4 on 2026-10-10: **« ca » is read as
a spelling of `ça`, in a forms-table change of its own** — this one.

Measured on the corpora at hand (design D3): written in lower case or with an initial capital, every
`ca` that is a French word is `ça` — 6 of 6 in UD's French treebanks (GSD 4, PUD 2), 19 of 19 in
Tatoeba's 727,867 French sentences —; written in capitals it is the acronym — `CA` 3 of 3 in UD (the
sports clubs « CA Vitry », « CA Peñarol », « CA Montreuil »), 5 of 5 in the French Wikipedia sample
(« le CA » of Geneva's Conseil administratif) —; « ca. 1893 » (circa) once. wordfreq, which reads the
web, subtitles and social media, rates `ca` at 12 % of `ça`'s frequency (Zipf 5.60 against 6.53),
three to six times the share of French's other cedilla-less spellings (`francais`, `facon`, `garcon`,
`recu`: 2–4 %). No web forum corpus is in the measurements' reach: wordfreq is the web's evidence.

## What Changes

- **A reviewed table of spellings in fr-en's reducer** (design D1): `UNMARKED_SPELLINGS`, form →
  (word, reason), one row, `ca` → *ça*, read as change 43 reads its elided pieces (`ELISIONS`): the
  form is a form of the named word alone; its own entries in the section — `CA`'s four initialisms,
  `ca` « abbreviation of circa » — make it no lemma and give it no inflection. An override row cannot
  say it: `OVERRIDES` chooses among a form's candidates, and no entry links `ca` to `ça`; change 43's
  spelling rule (D7) reads what the dictionary says, and it says nothing of `ca`.
- **What `ca`'s own words become: lost** (D2). `ca` is no ranked lemma, no dictionary word, no
  levelled word and no glossed word any more; « board of directors » and circa leave the pack. `CA` in
  capitals reads as `ça` too: the pack holds lower-case forms and French's cascade looks every token
  up lowercased (change 41), so no table can keep the acronym for capitals — a core rule could, and is
  not proposed (open question 1).
- **What moves, measured on a prototype that reproduces the committed tables byte for byte** (D4):
  - `tables/fr/forms.tsv` `ca` → *ça* (124,096 → 124,097 forms: `cussac` enters at the cut's end);
    `freq.tsv` `ca` (144) out, 59,853 ranks up by one; `lexical.tsv` 26,486 → 26,485 words;
    `level.tsv` `ca` (A1) out and one word up a band at each boundary (`croissance` A2 → A1,
    `communiste` B1 → A2, `australien` B2 → B1, `naïveté` C1 → B2, `morphologie` C2 → C1, `crabe` gains
    C2); `grammar.tsv` and the tag pool byte for byte;
  - fr-en: `gloss.tsv` and `senses.tsv` lose `ca`'s row and gain `cussac`'s (a name, no dictionary
    word), `mwe.tsv` byte for byte; coverage 93.6 / 86.9 / 76.3 % unchanged (15,260 → 15,259 of the
    top 20,000); the pin's rules and pack move, its snapshot and sources do not;
  - fr-es: re-reduced on the new `tables/fr/`, its glosses, senses and expressions byte for byte;
    coverage 83.2 / 70.8 / 56.8 % (7,082 → 7,083 of the top 10,000); its pin's studied record and pack
    move;
  - UD French-PUD 99.13 / 96.41 / 99.90 % and GSD's test section 98.89 / 95.86 / 99.72 %, unchanged.
- **The goldens** (D5): `fr-en.golden` and `fr-es.golden` move 9 of their 213 probes each — the pack
  line, « ca » on the `informel` page read as `ça` (fr-en « that… » for « board of directors », fr-es
  « Eso, esto, aquello » where it had none), the vocabulary universe 26,486 → 26,485 and the reader's
  estimate 2,184 → 2,185, and the deck the levels seed (`croissance` is now A1: `célèbre` and
  `davantage` lead the review, `exposition` and `morphologie` join the deck) —; three phrase probes are
  added — « comme ca », « c'est ca » and « Le CA a voté le budget. » —, showing `ça`'s expressions met
  and the acronym's cost. The word-card snapshots gain those three cards and move nothing else.
- **What is not generalised** (D6): of the other spellings without their marks, the real words keep
  their own (`ou`, `a`, `la`, `des`, `du`, `sur`, `mais`, `cote`, `tache`: UD reads `ou` as *où* 8 times
  in 1,048), and the candidates — the dictionary's own unmarked spellings (`etre`, `etat`, 51 ranked),
  rare words an unmarked spelling swamps (`age` « beam of a plough », `forcement`), unmarked spellings
  no entry knows (`meme`, `tres`, `deja`, `francais`, `facon`) — are listed with their figures and left
  to the owner (open questions 2–4).
- **en-fr, es-fr, es-en and en-es cannot move**: the rule lives in `reduce-fr-en.py`, which only
  fr-en's rule digest names; no shared or edition module, no table of theirs, no core code changes;
  their goldens passed unmoved over the prototype.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A French spelling without its cedilla reads as the word a reviewed
  table names* (`ca` → *ça*, its own entries giving no lemma, no rank, no level and no gloss; capitals
  read alike; nothing else moves).

No requirement is modified. The new one narrows how change 43's *French's forms and frequencies*
reads one form, as its elided pieces' table does, and it moves what changes 46 (*French's estimated
levels*), 48 and 48b (fr-en's glosses, *A pack's dictionary words do not depend on its glosses*) and
49 (fr-es on French's committed tables) write from the forms — all open, some held by change 52 too,
so it says so rather than MODIFY them. Those changes, change 41 (whose cascade reads every token
lowercased) and change 44 (whose keys a spelling meets) are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua's data and its tests only:
  - `scripts/lingua-data` — *changed*: `reduce-fr-en.py` (the table, read in `Lexicon.read` and
    `_inflections`), `test_reduce_fr_en.py`, `tables/fr/` (`forms.tsv`, `freq.tsv`, `level.tsv`,
    `lexical.tsv`), `tables/fr-en/` (`gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json`,
    `README.md`), `tables/fr-es/` (`manifest.json`, `pin.json`, `README.md`), `SOURCES.md`;
    *consumed*: `reduce_common.py`, `reduce_edition_en.py`, `reduce-fr-es.py`, unchanged.
  - `crates/lingua-pack` — `tests/committed_tables.rs`: `ca` reads as `ça` in both French packs.
  - `crates/lingua-wasm` — three phrase probes in the French scenario; `fr-en.golden` and
    `fr-es.golden` re-blessed.
  - `apps/lingua-extension` — `word-card-fr-en.txt` and `word-card-fr-es.txt` gain three cards, their
    specs' count 34 → 37; no code.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host app
  and the agent plugin are untouched; French's analyser version does not move.
- **Release.** Silent: no package lists a French pair. It lands before change 52 lists fr-en, so no
  reader ever studies `ca` as « board of directors ».
- **Compatibility.** No reader holds a French pack, so no status, card or level names `ca`; nothing to
  migrate.
- **Effort, against no programme estimate (outside the 57)**: 0.75–1.25 ideal days (design,
  *Effort*).
