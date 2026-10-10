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

The owner answered this proposal's questions on 2026-10-10 (design, *Settled by the owner*): `age` →
*âge* and `forcement` → *forcément* join `ca` here; « CA » in capitals is to stay the acronym, through
an analyser change of its own, and this change reaches readers only with it or after it; the
dictionary's own unmarked spellings and the web's unglossed ones are changes of their own.

## What Changes

- **A reviewed table of spellings in fr-en's reducer** (design D1): `UNMARKED_SPELLINGS`, form →
  (word, reason), four rows — `ca` → *ça*, `age` and `ages` → *âge*, `forcement` → *forcément* —, read
  as change 43 reads its elided pieces (`ELISIONS`): the form is a form of the named word alone; its
  own entries in the section — `CA`'s four initialisms, `ca` « abbreviation of circa », `age` « beam
  (central bar of a plough); shaft », `forcement` « fixing number, cooking the books » — make it no
  lemma and give it no inflection. An override row cannot say it: `OVERRIDES` chooses among a form's
  candidates, and no entry links these spellings to their word; change 43's spelling rule (D7) reads
  what the dictionary says, and it says nothing of them. `cote`, `tache`, `pale`, `foret` and `aine`,
  words a reader can mean, are never rows.
- **What the spellings' own words become: lost** (D2). `ca`, `age` and `forcement` are no ranked
  lemma, dictionary word, levelled word or glossed word any more; « board of directors », circa, the
  plough's beam and the fixing number leave the pack, and `forcements`, the noun's plural, which no
  word now reaches, is no form.
- **« CA » in capitals: the acronym, in a change of its own** (D2). The pack holds lower-case forms and
  French's cascade looks every token up lowercased (change 41), so no table can keep `CA` apart: until
  an analyser change reads a word written in capitals first in a small reviewed acronym table (the
  option the design recommends, with the others and their cost), « CA » reads as `ça`. This change
  reaches readers only together with that change or after it, unless the owner decides otherwise
  (task 6.4); the probe « Le CA a voté le budget. » records it.
- **What moves, measured on a prototype that reproduces the committed tables byte for byte** (D4):
  - `tables/fr/forms.tsv` 124,096 → 124,101 forms (`ca`, `age`, `ages`, `forcement` read as their
    word; `forcements` out; `cussac`, `céphalonie` and `côtelé`'s four forms in at the cut's end);
    `freq.tsv` `ca` (144), `age` (2,461) and `forcement` (9,588) out, 59,851 ranks up by one to three;
    `lexical.tsv` 26,486 → 26,484 words; `level.tsv` `ca` (A1), `age` (B1) and `forcement` (C2) out,
    eight words up a band (`croissance` A2 → A1 …) and three into C2; `grammar.tsv` 125,177 rows, the
    readings of `age`, `ages`, `forcement` and `forcements` out and `côtelé`'s in; the tag pool byte for
    byte;
  - fr-en: three glosses out (« board of directors », the plough's beam, the fixing number), three in
    (`cussac` and `céphalonie`, names, and `côtelé` « ribbed »); `mwe.tsv` byte for byte; coverage
    93.6 / 86.9 / 76.3 % unchanged (15,260 → 15,259 of the top 20,000); the pin's rules and pack move;
  - fr-es: re-reduced on the new `tables/fr/`, every gloss it had byte for byte and `côtelé` « Pana »
    gained; coverage 83.0 / 70.7 / 56.7 % (7,065 → 7,067 of the top 10,000), measured on change
    49b's implementation; its pin's studied record and pack move;
  - UD French-PUD 99.13 / 96.41 / 99.90 % and GSD's test section 98.89 / 95.86 / 99.72 %, unchanged.
- **The goldens** (D5): `fr-en.golden` and `fr-es.golden` move 9 of their 213 probes each — the pack
  line, « ca » on the `informel` page read as `ça`, the vocabulary universe 26,486 → 26,484 and the
  reader's estimate 2,184 → 2,185, and the deck the levels seed —; four phrase probes are added —
  « comme ca », « c'est ca », « Le CA a voté le budget. » and « à mon age » —, showing `ça`'s
  expressions met, `age` read as *âge* « age » (« Edad » in fr-es), and the acronym read as `ça` until
  its change. The word-card snapshots gain those four cards and move nothing else.
- **Recorded, not done here** (D6): the dictionary's own 186 accent-less spellings (`etre` « obsolete
  spelling of être », `etat`; 51 ranked), read as their accented word when it is the commoner, and a
  reviewed list by name of the web's unglossed accent-less spellings (`francais`, `tres`, `deja`,
  `meme`; 301 of Zipf ≥ 3) — two changes of their own, settled by the owner; the real words (`ou`,
  `a`, `la`, `des`, `du`, `sur`) never. Found while measuring: fr-en answers « ça va » with `ça ira`
  (one key for both); the owner will be asked separately.
- **en-fr, es-fr, es-en and en-es cannot move**: the rule lives in `reduce-fr-en.py`, which only
  fr-en's rule digest names; no shared or edition module, no table of theirs, no core code changes;
  their goldens passed unmoved over the prototype.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A French spelling without its marks reads as the word a reviewed
  table names* (`ca` → *ça*, `age`/`ages` → *âge*, `forcement` → *forcément*; their own entries giving
  no lemma, rank, level or gloss; read alike whatever their case until a rule of French's analysis
  reads a word in capitals apart; nothing else moves).

No requirement is modified. The new one narrows how change 43's *French's forms and frequencies*
reads four forms, as its elided pieces' table does, and it moves what changes 46 (*French's estimated
levels*), 48 and 48b (fr-en's glosses, *A pack's dictionary words do not depend on its glosses*) and
49 (fr-es on French's committed tables) write from the forms — all open, some held by change 52 too,
so it says so rather than MODIFY them. Those changes, change 41 (whose cascade reads every token
lowercased) and change 44 (whose keys a spelling meets) are in `archiveAfter`. The acronym change
will ADD its rule beside this one; the requirement leaves it room.

## Impact

- **Products.** Cymbra Lingua's data and its tests only:
  - `scripts/lingua-data` — *changed*: `reduce-fr-en.py` (the table, read in `Lexicon.read` and
    `_inflections`), `test_reduce_fr_en.py`, `test_reduce_editions.py`, `tables/fr/` (`forms.tsv`,
    `freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`), `tables/fr-en/` (`gloss.tsv`, `senses.tsv`,
    `manifest.json`, `pin.json`, `README.md`), `tables/fr-es/` (`gloss.tsv`, `senses.tsv`,
    `manifest.json`, `pin.json`, `README.md`), `SOURCES.md`; *consumed*: `reduce_common.py`,
    `reduce_edition_en.py`, `reduce-fr-es.py`, unchanged.
  - `crates/lingua-pack` — `tests/committed_tables.rs`: the four spellings read as their words in
    both French packs.
  - `crates/lingua-wasm` — four phrase probes in the French scenario; `fr-en.golden` and
    `fr-es.golden` re-blessed.
  - `apps/lingua-extension` — `word-card-fr-en.txt` and `word-card-fr-es.txt` gain four cards, their
    specs' count 34 → 38; no code.

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host app
  and the agent plugin are untouched; French's analyser version does not move.
- **Release.** Silent: no package lists a French pair. It reaches readers only with or after the
  acronym change (task 6.4) and, like 48b and 49b, before change 52 lists French.
- **Compatibility.** No reader holds a French pack, so no status, card or level names `ca`, `age` or
  `forcement`; nothing to migrate.
- **Effort, against no programme estimate (outside the 57)**: 0.75–1.25 ideal days (design,
  *Effort*).
