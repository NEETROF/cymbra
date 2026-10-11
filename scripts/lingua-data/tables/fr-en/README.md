# The fr→en dictionary tables

The reduced tables of Cymbra Lingua's French→English data pack, committed so that every build makes
the same pack with no download, and so that a change to the dictionary is a pull request whose diff
shows it (add-lingua-french-forms-tables, change 43 of `docs/lingua/language-matrix-programme.md`).
No extension package carries the pack: `packs.json` does not list it, and no reader holds a French
pack before `enable-lingua-french` (change 52).

They are in two folders (split-lingua-pack-tables-by-language). fr-en is French's reference pair
(`../fr/studied.json`): its reduction writes French's own tables in `../fr/`, which every pair
studying French reads as committed (fr-es, change 49), and this folder's `pin.json` records the
sources of both. fr-en reduces French's forms, ranks, readings and estimated levels, and its own
native side — the English glosses of French words and expressions (add-lingua-pack-fr-en, change 48,
*The glosses* below) — from the same section; its glossed lemmas, less those it glosses by a proper
noun's senses alone, are French's dictionary words (refine-lingua-fr-en-glosses). Its expressions
are completed by the French Wiktionary's English translations where the section glosses none
(add-lingua-fr-en-translated-expressions, *The French Wiktionary's expressions* below).

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → English gloss: up to eight whole senses, grouped by part of speech (*The glosses*) | kaikki.org, the English Wiktionary's French section (CC BY-SA 4.0 + GFDL), read by the English edition's rules |
| `senses.tsv` | lemma → its gloss's runs: each part of speech and how many senses it holds | the same entries |
| `mwe.tsv` | expression → English gloss: the section's headwords with a space, and the single words French's tokenisation splits (`d'abord`); then the French Wiktionary's French expressions the section does not gloss (`il existe`, `s'asseoir`; *The French Wiktionary's expressions*) | the same section; the English translations the French Wiktionary's French entries list (kaikki.org, `frwiktionary`, CC BY-SA 4.0 + GFDL) |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | the pack's metadata: French glossed in English, French's analyser version (`FRENCH_ANALYZER_VERSION`, read from lingua-core), `levels_estimated` (the levels are estimated, not a CEFR list's), and `pack_version` (the snapshot, and the rules that reduced it) | — |
| `pin.json` | the raw sources these tables and `../fr/` came from, and the pack they build | — |

In `../fr/`, French's tables, written by fr-en's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma; French's acronym rows in capitals, each its own lemma (`CA` → `CA`, *The rules*) | kaikki.org, the English Wiktionary's French section (CC BY-SA 4.0 + GFDL), with UD French-GSD's counts to choose between lemmas (CC BY-SA 4.0) |
| `freq.tsv` | lemma → frequency rank | wordfreq 3.1.1 (CC BY-SA 4.0); a hyphenated word by GSD's own frequency too |
| `grammar.tsv` | form → its readings, as Universal Dependencies tags (`form<TAB>lemma<TAB>tag<TAB>other\|-`) | kaikki.org, the same section (add-lingua-french-grammar-tables, change 45) |
| `level.tsv` | lemma → estimated CEFR level (*The levels* below) | derived from `freq.tsv`, the English Wiktionary's French section and French's dictionary words (no source of its own) |
| `lexical.tsv` | French's dictionary words: the lemmas fr-en glosses less those it glosses by a proper noun's senses alone, and French's acronym rows (26,555, `CA` among them; *The glosses*) | derived from `gloss.tsv` and `senses.tsv` by `reduce-fr-en.py` (`dictionary_words`), filed by `build.sh` (`pack_sources.py split`) |
| `tags.tsv` | French's pinned tag pool: the 79 tags its readings carry, in byte order; written by no reducer | committed by hand, once, from the first reduction's readings |
| `studied.json` | the pair whose reduction writes `../fr/`: fr-en | committed by hand |

## The sources

- **kaikki.org, the English Wiktionary's French section** (`kaikki-French.jsonl`, 403,269 entries,
  510,058,226 B), derived from the English edition's dump (`pack_sources.py EDITIONS`, `DUMPS["fr-en"]`):
  the dump regenerated on 2026-10-03 08:24, read on 2026-10-09, decompressed sha256 `93b79aac…`.
  Its inflections are tagged, a lemma's table lists them and a form's own entry points at what it is
  a form of. A verb's table lists the masculine singular past participle (`dirigé`) and none of its
  agreed forms: those hang under the participle's own entry. Its senses, written in English for
  French words, are fr-en's glosses too: the one source of the glosses, no translation table. Kept
  byte for byte by the update of 2026-10-10 (`keep=en`), its record names the release
  `lingua-pack-sources-fr-en-2026.10.09`, which holds it.
- **kaikki.org, the French Wiktionary's English translations** (`kaikki-fr-traductions-en.jsonl`,
  129,366 French entries listing English translations, 14,573,988 B, sha256 `6f54aeda…`, 1,914,447 B
  as its zstd asset), derived from the French edition's dump (`DUMPS["fr-en"]`): the dump regenerated
  on 2026-10-02 00:10, read on 2026-10-10, decompressed sha256 `968f7df0…` (the regeneration fr-es's
  pin records), published as `lingua-pack-sources-fr-en-2026.10.10` by `lingua-pack-update` run
  38085007726 — an update keeping the English edition (add-lingua-fr-en-translated-expressions D6).
  Read for expressions alone (*The French Wiktionary's expressions*).
- **UD French-GSD**, its training and development sections at `94d5b68e185fc22a9ef292040e84f476d36d9b0e`
  (`pack_sources.py PINNED["fr-en"]`), read for how often a form stands for each lemma and how often
  a hyphenated lemma occurs (390,368 words) — and, for the glosses, how often it reads each word
  under each part of speech (refine-lingua-fr-en-glosses D5). Its test section is never read: the
  measurement holds it out.
- **wordfreq `fr` 3.1.1**, pinned by version and by `../../requirements-reduce.txt`'s hashes.

## The rules (`reduce-fr-en.py`)

- **A form** is a French word, lowercased, in NFC, its typographic apostrophe read as `'`: letters,
  words joined by hyphens or an inner apostrophe (`aujourd'hui`, `presqu'île`), and the elided
  pieces' final apostrophe. Its candidates are the inflections a lemma's entry lists — never kaikki's
  bookkeeping, a multi-word construction (`avoir dirigé`, and `ayant`, which kaikki tags as one), nor
  an inflection tagged alternative, obsolete, archaic, rare, dated, uncommon, misspelt, nonstandard,
  proscribed, abbreviated, clipped or a pronunciation spelling, nor a gender or number marker a head
  left among the forms (`m` under *Paris*, which read « M. » as Paris; 7 rows in the section) — and
  the first word of each form-of target of its own entry (`bel` → *beau*).
- **A form of a form, along one part of speech.** A candidate is followed along the part of speech
  of the entry linking the form to it: along it, a word reads as the first choice among itself —
  only when one of its lemma entries is of that part of speech — and the words its entries of that
  part of speech link it to, never a word whose entries are all of other parts of speech; a
  candidate it reads as another word stands for that word. `dirigée`, the feminine of the
  participle `dirigé`, reaches *diriger*; `citée`, the feminine of the participle `cité`, reaches
  *citer*, not the noun *cité* (the city) the participle is spelt like; `tues` reaches *tuer* and
  *taire* (through the participle `tu`), not the pronoun; `quise` reaches *quérir* through `quis`,
  whose verb entry's « masculine plural of qui » is no verb; `étés`, the plural of the noun `été`,
  does not follow `été` to *être* through a verb entry, and stays out — an unknown word rather than
  the verb. Without the rule the tables fail the resolution gate (97.61 % on PUD). The first
  implementation followed a candidate's overall first choice, which stopped at the noun: 157 forms
  only verb entries link read as a noun, a name or a pronoun (`citée` → *cité*, `tues` → *tu*,
  `marchée` → *marché*); one remains (`shaka` → *shaker*, whose verb has no entry in the section).
- **French's tokenisation (M21)**, as add-lingua-french-tokenisation's pre-pass reads it:
  - every word the pre-pass writes is a form: the words the elided pieces stand for, `à`, `le`,
    `les` and the inversion's pronouns (32 words; the rarest, `quoique`, is rank 3,307);
  - the fourteen elided pieces are forms of the word the pre-pass reads them as outside its special
    cases, by a reviewed table (`ELISIONS`: `l'` → *le*, `s'` → *se*, `qu'` → *que*, `jusqu'` →
    *jusque*, …): the treebanks write them as words, and the dictionary cannot decide them;
  - no plain word beginning with a piece is a form (`c'est`, `d'abord`, `l'on`, `jusqu'à`): the
    pre-pass splits it whatever the pack lists. A hyphenated run is read whole first, so it may be one
    (`c'est-à-dire`, rank 459);
  - the dictionary's hyphenated nouns, adjectives, adverbs, pronouns and prepositions ending in a
    pronoun are listed whole, or the inversion rule would split them: `rendez-vous` (rank 1,580),
    and `qu'en-dira-t-on`, `malgré-nous` and `non-moi`, which GSD never meets, at the cut's last
    ranks. A reduction that leaves one out fails. The verbs, phrases and interjections made of a verb
    and its pronouns (`est-il`, `a-t-il`, `allez-y`, `excusez-moi`) stay out: the split reads them;
  - `au` and `aux` are neither forms nor ranks: the pre-pass always splits them;
  - `du` and `des` are words of their own (ranks 10 and 6), `des` against GSD's counts, which read
    it as *un* 1,730 times out of 1,736;
  - a verb form joined to its pronouns by hyphens (`souviens-toi`, `allons-nous-en`, `sois-t'en`) is
    no form: 459 of them, which the inversion rule reads as the verb and its pronouns.
- **A spelling variant reads as the word it spells**: an entry whose every sense is an ASCII
  spelling of a ligature (`coeur` → *cœur*) or a post-1990 spelling (`connait` → *connaître*,
  `évènement`, `chaine`), with its own inflections (`coeurs` → *cœur*). A word of its own as well
  (`clef`) keeps its entry.
- **A spelling without its marks reads as the word a reviewed table names**
  (read-lingua-french-ca-as-ca-cedilla, the owner's decision of 2026-10-10): `UNMARKED_SPELLINGS`,
  beside the elided pieces' table and read as it is, each row with its reason — `ca` → *ça*, `age` and
  `ages` → *âge*, `forcement` → *forcément*. The spelling is a form of its word alone: its own entries
  — `CA`'s four initialisms, `ca` « abbreviation of circa », `age` « beam (central bar of a plough);
  shaft », `forcement` « fixing number, cooking the books » — make it no lemma and list no inflection,
  and an entry pointing at it (« plural of CA ») links no word to it; those words leave the tables (no
  rank, level, dictionary word or gloss), and `forcements`, the fixing number's plural, is no form. No
  entry links the spellings to their word, so neither an override (which chooses among a form's
  candidates) nor the spelling rule above (which reads what an entry says it spells) could. `cote`
  (« rating »), `tache` (« stain »), `pale` (« blade »), `foret` (« drill bit ») and `aine`
  (« groin ») are words a reader can mean and never rows; `ou`, `a`, `la`, `des`, `du` and `sur` are
  words of their own. French's analysis looks every token up lowercased, so « CA » in capitals reads as
  *ça* too — unless it stands alone in capitals, when the analysis reads it as the acronym row below
  (*What still reads wrong*). The dictionary's own unmarked spellings (`etre`, `etat`) are read by the
  rule after the acronyms'; the web's (`tete`, `francais`, `tres`) by the reviewed list below.
- **An acronym in capitals is a row of a reviewed table** (keep-lingua-french-acronyms-in-capitals, the
  owner's decision of 2026-10-10, option A of read-lingua-french-ca-as-ca-cedilla D2): `ACRONYMS` in
  `reduce_french_acronyms.py`, a rule module fr-es's reducer imports too, each row the acronym as
  written, its expansions as the section's own entry names them, its part of speech and its reason —
  one row, `CA`: `conseil d'administration`, `chiffre d'affaires`, `courant alternatif`, `comptable
  agréé`, a noun. The section's entry headed exactly `CA` is read for the table alone, and must point
  at every expansion the row names, or the reduction stops naming the row. The row is written as a form
  that is its own lemma, capitals kept — `CA` → `CA`, the one form holding a capital, which only
  French's analysis reaches, looking a word standing alone in capitals up as written (« Le CA a voté le
  budget. »; a headline or a shout in capitals, and « Ca », read *ça*) — and as a dictionary word
  (`lexical.tsv`), with no rank, level or reading: wordfreq lowercases, and its `ca` is mostly *ça*.
  `ca` → *ça* is unchanged. The builder accepts a capital nowhere else.
- **A spelling the dictionary gives only as a marked word reads as that word when the marked word is
  the commoner** (read-lingua-french-dictionary-unmarked-spellings, the owner's decision of
  2026-10-10): the spelling rule's third kind, beside the ASCII spelling of a ligature and the
  post-1990 spelling. A pass of its own before the forms (`dictionary_unmarked_spellings`) names each
  word every sense of every entry of which is an `alt_of` of one word that, written without its
  accents, cedillas, diaereses and circumflexes, is it — `etre` « obsolete spelling of être », `Etat`
  « alternative spelling of État », `siecle` « obsolete form of siècle », `repondre` « obsolete
  spelling of répondre » — and keeps it when wordfreq (3.1.1, the pin's) rates the marked word
  strictly commoner than the spelling and than each of the spelling's forms (the inflections its
  entries list, less the ones `Lexicon` skips, and the entries a form-of sense of which points at
  it). The spelling is then read as a ligature's ASCII spelling is: a form of its word, its
  inflections following it along their part of speech (`repond`, `reponds`, `repondu` → *répondre*;
  `siecles` → *siècle*; `etait` → *était* → *être*), its readings filed under the marked word, its own
  entries giving no lemma — no rank, level, dictionary word or gloss —, whatever its case (« Etat »,
  « ETAT »: the analysis looks every token up lowercased). Of the section's **141 accent-less
  spellings** (beside the 45 ASCII spellings of a ligature the rule above reads), **81 are read**: 33
  ranked before (`etat` 489, `etats` 867, `etre` 1,014, `etats-unis` 1,354, `etait` 1,995, `benoit`,
  `derniere`, `arreter`, `repondre`, `siecle`…), 21 a form already or now (`critere`, `babord`,
  `pylone`, `melancolie`), 27 no form either way (`chomer`, whose forms `chome` and `choma` do read
  as *chômer*; `geologique`, `taboule`). **56 keep their own reading**, the spelling as common as its
  word or commoner — 18 of them ranked: `diesel` (Zipf 4.05 against *diésel*'s 2.16), `ego`,
  `egypte`, `etienne`, `eve`, `ethiopie`, `impresario` (2.47 / 2.45) — and 29 of them a pair wordfreq
  attests in neither spelling (`abigeat`, `ypreau`); **`teter`** « alternative form of téter » is kept
  by the forms' half, its form `tete` (3.83) being how the web writes *tête*, commoner than *téter*
  (2.52); « de main de maitre », « emirats arabes unis » and « ltee. » are no word form. The
  reduction says so on its stderr: `spellings without their marks read as their word=81, kept
  apart=60 (56 …, 1 …, 3 no word form)`. A word with an entry or a sense of its own beside the
  spelling is not read — `sure`, `sures`, `surs` (the plural of *sur* « sour »), `entrainant` (the
  participle of the post-1990 `entrainer`), `chouia`, `colon`, `mercedes`, `to`; `cordeliere`,
  `grande-ile` —, nor is a row of `UNMARKED_SPELLINGS`, which its table reads first; no core rule
  reads a word by its marks (change 41's D2), and French's analyser version does not move. One
  correctly spelt form changes word: `rebellions`, the imperfect of *se rebeller* (« nous nous
  rebellions ») and the noun's plural without its accent, reads as *rébellion*, the commoner word
  (M8; the owner kept it on 2026-10-10).
- **The web's spellings without their marks are rows of a reviewed list**
  (read-lingua-french-web-unmarked-spellings, the owner's decision of 2026-10-10): `WEB_SPELLINGS` in
  `reduce-fr-en.py`, **464 spellings** the section names nowhere that the web writes for a French word
  — `meme`, `tres`, `apres`, `francais`, `deja`, `recu`, `leger`, `tete` —, merged into
  `UNMARKED_SPELLINGS` with one shared reason, and the owner's **four exceptions** as rows of their own
  with their reasons: `sure`, `surs`, `sures` → *sûr* and `entrainant` → *entraîner*, words the section
  gives a sense of their own (*sur* « sour »; the participle of the post-1990 `entrainer`) that the
  web writes for the commoner word. 468 rows with 43b's four. Each row was chosen by a person, never
  by a rule on marks: a spelling wordfreq attests at Zipf 2.5 or more, that the section names nowhere
  (neither a meaning of its own nor its accented word's spelling), whose accented word is a
  dictionary word at least twice as common, and that English does not write more often (`video`,
  `hotel`, `these`, `role`, `mere`, `premiere` are left out, 239 in all); a spelling two words share
  reads as the commoner (22: `ecoute`, `pate`, `peche`); `dela` and `nes` stay out; names and other
  languages' words spelt like a row are accepted (`noel`, `tres`, `tresor`). Hyphenated words
  (`peut-etre`) and wrong accents (`trés`, `déja`) are a reviewed list of their own each (below). Each
  row's word is the lemma the tables give its accented form (`etais` → *être*, `recu` → *recevoir*,
  `epargne` → *épargner*). A row is read as 43b's are — a form of its word alone, its own entries and
  any entry's inflection making it no other word's form, seeded whether or not an entry names it —,
  and an entry pointing at a row's spelling follows the row: `patisseries` « plural of patisserie »
  reads as *pâtisserie*, `entrainante` and `entrainantes` as *entraîner* (43b's rows still drop the
  link). **The boundary is checked at each reduction**: the reduction fails, naming them, on any
  spelling of `WEB_SPELLINGS` the section gives a meaning of its own (a headword with a sense that is
  neither a form nor a spelling of another word, a form of such a word, or an inflection such a
  word's entry lists) or gives only as its accented word's spelling (the dictionary rule's kind), so a
  row never shadows a word (`cote`) or a spelling the dictionary gives (`etait`), and a dictionary
  update naming a row stops for a person. `tete` and `tetes`, listed only by `teter` « alternative
  form of téter », pass, and the dictionary rule's forms guard still counts them among `teter`'s
  forms, so `teter` stays apart. Each spelling carries the readings of every form of its word it
  writes without their marks (*The readings*). `ou`, `a`, `la`, `sur`, `du`, `des`, `cote`, `tache`,
  `pale`, `foret` and `aine` are never rows.
- **The web's hyphenated words without their marks and its wrong accents are two more reviewed
  lists** (read-lingua-french-hyphenated-and-misaccented-spellings, the owner's decision of
  2026-10-10): `WEB_HYPHENATED`, **19 runs** — `peut-etre`, `la-bas`, `grand-mere`, `lui-meme`,
  `grand-pere`, `apres-midi`, `moi-meme`, `petit-dejeuner`, `celui-la`, `c'est-a-dire`, `celle-la`,
  `au-dela`, `eux-memes`, `ceux-la`, `soi-meme`, `la-dessus`, `apres-demain`, `belle-mere`,
  `celles-la` —, and `WEB_MISACCENTED`, **65 words** written with marks not their own — `déja`, `trés`,
  `aprés`, `étre`, `oú`, `mére`, `eté`, `théatre`, `agé`, `méme`, `pére`, `arreté` … —, each merged into
  `UNMARKED_SPELLINGS` with its list's reason and read as `WEB_SPELLINGS` is, under the same boundary
  (which also refuses a row the section gives only as another word's spelling with the same letters:
  `problême` « alternative form of problème », `réglement`). 552 rows in all. A hyphenated row is the
  whole run, the accented compound less its marks: the pack lists it, and the pre-pass reads it as one
  token, as it reads `peut-être` — before an elision splits it (`c'est-a-dire`) or its last pieces read
  as an inversion (`celle-la`); a row for a piece could not do it. wordfreq cannot count a hyphenated
  spelling (it answers its rarer piece: `peut-etre` 4.75 is `etre`'s), so a row is a form of a
  dictionary word whose accented compound, counted in 6.36 M words of French corpora and shifted by the
  web's measured share of unaccented spellings (−1.59, the median over 43e's rows), stands at Zipf 2.5
  or more — 18 of 227 —, and `celles-la` with its paradigm. A misaccented row is a word wordfreq rates
  at Zipf 2.5 or more whose letters are a form's of the tables and whose marks are not, that the
  section names nowhere, and that no other language writes more often; it reads as the form whose marks
  it keeps in the most places, then the commoner: `agé` → *âgé* (not *âge*), `elève` → *élève*
  (*élever*'s), `elevé` → *élevé*, `régle` → *règle*. Never rows: `á`, `lá`, `ã`, `cà`, `accés`,
  `procés`, `succés` (Portuguese's, Italian's, Catalan's words), `çe` and `abbès`; nor the names and
  non-words a wrong accent points at (`jérome`, `andrés`, `malgrès`), nor `pêle`, `elysées`, as common
  as their word. `oú`'s `ú` is no letter of French's forms: no inflection names it, the row alone
  seeds it, and the analysis, which reads any letter, looks it up.
- **One lemma per form (M8).** A form that is a proper name and a commoner word's keeps the word
  (`cette` → *ce*; `paris` stays the city), then the first rule that decides wins:
  1. `OVERRIDES` in `reduce-fr-en.py`, each row with its reason — no homograph; three rows correct
     the source's copy errors: `fatiguée` (« feminine singular of parlé ») → *fatiguer*, `bridée`
     (a form-of target « female slant ») → *bridé*, `quis` (« masculine plural of qui » in a verb
     entry) → *quérir*. A row holds wherever a chain passes too (`fatiguées` follows `fatiguée`);
  2. GSD's counts of the form under each lemma (`porte` → *porter*, 39 against 23);
  3. the form's own entry;
  4. the commoner lemma;
  5. the alphabet.
- **The ranks**: wordfreq's order over its words that are not only inflected forms, the 60,000 first
  kept. Not ranked:
  - wordfreq's elision stems, which carry the pieces' frequency (`l`, `d`, `qu`, `jusqu`, `ç`, …);
  - a word whose own form reads as another word once the forms of forms are followed: a pack finds a
    lemma by its own form — the builder keys its rank, and later its gloss and level, by looking the
    lemma up as a form —, so such a word would lend its rank to the other. Every form of the noun
    *tenue* reads as *tenir*; `donnée` reads as *donner*, and the noun *donnée*, which `données`
    alone still reached, leaves the pack with it, as M8's nouns do (`données` then reads as *donner*).
    27 words give their rank to the next this way (`tenue`, `allée`, `destinée`, `levée`, `donnée`,
    `venue`, `revenue`, `saisie`, `tranchée`, `bordée`, `retombée`, `subordonnée`, …);
  - a hyphenated word GSD's training sections do not attest. wordfreq splits at the hyphen and
    estimates a compound from its parts, so `est-il` would be French's 17th word. A compound GSD
    attests is ranked at the lower of wordfreq's estimate and GSD's own frequency, after wordfreq's
    words of the same frequency, compounds alphabetically: `lui-même` 276, `celui-ci` 335,
    `peut-être` 938, `au-delà` 1,007, `week-end` 1,666, `après-midi` 1,912, `en-cas` 10,597.
- **The forms**: those of a kept lemma that wordfreq attests, each kept lemma's own form, the
  elided pieces and the spellings `UNMARKED_SPELLINGS` names.

## What is in them

On the 2026-10-09 tables: **60,000 lemmas** and **124,847 forms** (`forms.tsv` 2,270,676 B,
`freq.tsv` 844,825 B). 3,585 forms keep more than one ranked candidate: GSD's counts decide 1,381,
the form's own entry 1,312, frequency or the alphabet 892. 7,219 forms take their lemma through a
form of a form. 473 hyphenated words are ranked — 470 by GSD's evidence and the 3 nouns ending in a
pronoun it never meets — 103 of them among the 5,000 first ranks, and 142 words with an inner
apostrophe (`aujourd'hui` 136, `quelqu'un` 203).

**French's unmarked spellings** (read-lingua-french-ca-as-ca-cedilla) moved them from 124,096 forms:
`ca`, `age`, `ages` and `forcement` read as *ça*, *âge* and *forcément*, `forcements` is no form,
and `cussac`, `céphalonie` and `côtelé` (with `côtelée`, `côtelées`, `côtelés`) entered with their
ranks. `ca` (144), `age` (2,461) and `forcement` (9,588) left the ranks: 59,851 lemmas after them
moved up by one, two or three, and the three entered at 59,995–59,997, before the three nouns ending
in a pronoun. `ça` keeps rank 26 (wordfreq ranks a lemma by its own frequency, not its forms'); `âge`
is 406 and `forcément` 1,286. The ranks this file quotes from *Measured* on were measured before
it, and may be one to three higher than these tables'. Every ranked lemma's own form reads as itself
(`crates/lingua-pack/tests/committed_tables.rs` checks every pair's pack holds each rank on its own
lemma).

**French's acronym table** (keep-lingua-french-acronyms-in-capitals) adds one form, `CA` → `CA`, the
first in byte order, and no rank, level or reading: `freq.tsv`, `level.tsv` and `grammar.tsv` are byte
for byte.

**The dictionary's unmarked spellings** (read-lingua-french-dictionary-unmarked-spellings) moved
them from 124,102 forms: 100 forms now read as a marked word — 69 that read as one of the 33 ranked
spellings (`etre`, `etait`, `etaient` → *être*; `etat`, `etats` → *état*; `repond`, `repondait` →
*répondre*; `gache`, `gaché` → *gâcher*), 30 that were no form (`critere`, `criteres`, `babord`,
`pylone`, `melancolie`, `chome`…) and `rebellions` (*rebeller* → *rébellion*) —, 53 forms of the 33
lemmas entering the cut joined them, and `désossé`, one of those lemmas, reads as itself, no longer
as *désosser*. The 33 spellings left the ranks: 59,476 lemmas after them moved up by one to 33, and
`dagon` … `eamon` entered at 59,965–59,997 (wordfreq's next words at the cut's frequency:
`dashboard`, `depth`, `déboîter`, `désarçonner`, `dînette`…), before the three nouns ending in a
pronoun. *être*, *état* and *siècle* keep ranks 36, 140 and 377; *répondre* is 590 (591), *arrêter*
525 (526).

**The web's unmarked spellings** (read-lingua-french-web-unmarked-spellings) moved them from 124,185
forms: the 468 spellings read as their words — 465 were ranked lemmas of their own, `sure` a form of
*sur*, `tete` and `tetes` no form (`teter` is unranked) —, `patisseries`, `entrainante` and
`entrainantes` follow their rows, and 555 forms of the lemmas entering the cut joined them. The 465
ranked spellings left the ranks — 12 of the 5,000 first (`meme` 1,015, `tres` 1,693, `apres` 2,338,
`ecole` 2,903, `eglise` 3,289), 42 of the 10,000, 168 of the 20,000, the last `sures` 37,569 —:
58,518 lemmas after them moved up by one to 465, and `ears` … `uruk` entered at 59,533–59,997
(`enceint`, `gloser`, `estudiantin`, and spellings under the list's threshold such as `eclats`,
`embete`, `emeute`, which the cut took in as it did before), before the three nouns ending in a
pronoun. The words keep their ranks: *même* 40, *très* 50, *déjà* 84, *français* 97, *école* 295.

**The web's hyphenated words and wrong accents** (read-lingua-french-hyphenated-and-misaccented-spellings)
moved them from 124,742 forms: the 84 rows read as their words — the 64 misaccented spellings
wordfreq ranks were lemmas of their own, the 19 hyphenated rows and `oú` no form —, and 86 forms of
the 64 lemmas entering the cut joined them; `épaississant`, one of those lemmas, reads as itself, no
longer as *épaissir*. The 64 ranked spellings left the ranks — `déja` 4,120 and `trés` 4,720 of the
5,000 first, 3 of the 10,000, 12 of the 20,000 —: 55,814 lemmas after them moved up by one to 64, and
`uterus` … `airelle` entered at 59,934–59,997 (`vertébré`, `étriper`, `écœurer`, `épicène`), before
the three nouns ending in a pronoun. No hyphenated row was ranked. The words keep their ranks: *très*
50, *déjà* 84, *peut-être* 938, *âgé* 2,733, *grand-mère* 3,514.

The pack these two tables built — no gloss, reading or level — was 1,241,733 B; Spanish's same two
tables build 1,308,123 B. With the levels it is 1,302,031 B (*The levels*), and with the readings
too 1,460,253 B (*The readings*). With the glosses, their runs and the expressions it was 2,527,222 B,
with fr-en's own rules and French's lexical table 2,537,386 B, with French's unmarked spellings
2,537,588 B, with French's acronym table 2,535,748 B, keeping every headword of a key several reach
(pick-lingua-expression-written-as-the-page) 2,536,702 B, with the dictionary's unmarked spellings
2,538,572 B, with the web's unmarked spellings 2,550,510 B (*The glosses*), with the French
Wiktionary's expressions 3,004,986 B (*The French Wiktionary's expressions*), and with the web's
hyphenated words and wrong accents it is **3,006,129 B**, 57 % of the builder's 5 MiB.

## Measured

`scripts/lingua-data/measure/fr-ud.sh` builds the pack from these tables and runs the real analyser
(`lingua-pack-measure`, French at `1.0.0`, change 41's cascade) over UD French-PUD at
`db260db10fe728853c549760801229ef4e7b16e1` — gated, with Spanish's thresholds — and over GSD's test
section, reported. The reduction reads neither. Punctuation, numbers, symbols, foreign words and
proper nouns are left out.

| | Words | Resolved | Content words | Auxiliaries |
|---|---|---|---|---|
| UD French-PUD (gate) | 20,232 | **99.13 %** (98.5 %) | **96.36 %** of 9,573 (93.5 %) | **100.00 %** of 1,030 (97 %) |
| GSD test (reported) | 8,049 | 98.91 % | 96.10 % of 3,791 | 99.72 % of 359 |

The web's hyphenated words and wrong accents (read-lingua-french-hyphenated-and-misaccented-spellings)
moved GSD's content words from 96.02 % (3,640 → 3,643): its reviews' « trés » (three times, « la
qualité du service est trés trés trés… bonne ») reads as *très*. PUD holds no row; its figures and
both treebanks' readings do not move (`tâtonner`, which the design expected to enter here, entered
with the web's unmarked spellings).

The web's unmarked spellings (read-lingua-french-web-unmarked-spellings) moved them from 99.13 /
96.36 / 99.90 and 98.91 / 95.94 / 99.72 %: PUD's « Etant » (an auxiliary) reads as *être*; its
« Evidemment », which PUD lemmatises as written, reads as *évidemment* now — the row is right, the
treebank keeps the unaccented lemma —, and `tâtonner`, a lemma entering the cut, resolves
« tâtonnant », so the content words hold at 9,225. GSD's « Eglise » (twice) and « Equipe » read as
*église* and *équipe* (3,637 → 3,640). No row of the list's lower band (Zipf 2.5–3) stands in either.

The dictionary's unmarked spellings (read-lingua-french-dictionary-unmarked-spellings) moved them
from 99.13 / 96.41 / 99.90 and 98.89 / 95.86 / 99.72 %: PUD writes « Etats-Unis » four times and
lemmatises it as written, which `etats-unis`, read as *états-unis* now, no longer matches (9,229 →
9,225 content words); GSD lemmatises « Etat » (twice) and « Etats » as *état*, matched now, and
`dilapidation`, a lemma entering the cut, resolves. The gates hold. The figures below predate it.

At `0.2.0`, the pack's lookup alone, the same tables read 99.13 / 96.30 / 99.90 on PUD and 98.89 /
95.70 / 99.72 on GSD's test section: change 41's cascade (an unlisted plural read as its singular)
adds 0.11 and 0.16 points of content lemmas. The figures below are the lookup's, at `0.2.0`.

The design's prototype measured 99.12 / 96.38 / 99.90 and 98.91 / 95.65 / 99.72; the same binary
reads its tables alike at `0.1.0` and `0.2.0`. Two rules of the implementation move them: a ranked
word whose own form reads as another gives its rank (PUD's content words 96.38 → 96.30, `données`
now *donner*; GSD's 95.65 → 95.67), and a gender marker is no form (GSD's resolved words 98.91 →
98.89: its `m` no longer reads as Paris). The chain along one part of speech moves them by a word:
PUD's resolved words 99.12 → 99.13 (`maitrisés`), its content words even — six participles take
PUD's verb (`traitée`, `coupée`, `revenue`), four adjectives and a noun lose PUD's adjective
(`morte` → *mourir*, where PUD writes *mort*), and `entrainé` reads as *entraîner*, the spelling it
stands for, where PUD writes *entrainer* —; GSD's content words 95.67 → 95.70. A participle's agreed
form that is an adjective's too (`morte`, `sacrée`, `ravie`, 506 forms) now reaches the verb beside
the adjective, and the one-lemma rule decides between them: GSD's counts first (`morte` 13 times
*mourir*, once *mort*), else the commoner lemma (`crues` → *croire*, `rangées` → *ranger*). 713
forms change lemma, 98 come in and 52 leave; none is among wordfreq's 1,800 commonest words.

The copy errors were looked for by the forms whose lemma begins with another letter (106): all but
three are a suppletive verb (`sont` → *être*, `va` → *aller*), a ligature's ASCII spelling (`oeuvre`
→ *œuvre*), `eux` → *ils*, `yeux` → *œil*, a pair the dictionary itself links (`marraine` →
*parrain*, `moindre` → *petit*), an Old French form (`fust` → *estre*) or an inclusive pronoun
(`ellui` → *iel*). `fatiguée` and `bridée` are overridden; `créditiste` (Zipf 1.2) reads as
*ralliement*, the first word of its target « Ralliement créditiste », which no row can mend since
*créditiste* is no ranked lemma.

The harness reads UD's words as UD writes them: an elided piece is `l'`, which `ELISIONS` resolves;
an inverted pronoun is `-il` or `-t-il`, which stays unresolved where the pre-pass hands `il` to the
lookup (18 of PUD's words, 17 of GSD's).

The cut, measured on the prototype's tables by the same lookup in Python when the design was
written (the binary reads 0.09 points less on PUD, the inverted pronouns):

| Cut | Forms | `forms.tsv` | PUD resolved | content | GSD test resolved | content |
|---|---|---|---|---|---|---|
| 20,000 lemmas | 62,013 | 1.15 MB | 98.24 % | 95.65 % | 97.70 % | 94.14 % |
| 40,000 lemmas | 97,037 | 1.78 MB | 99.05 % | 96.29 % | 98.93 % | 95.41 % |
| **60,000 lemmas** | **124,040** | **2.26 MB** | **99.21 %** | **96.40 %** | **99.12 %** | **95.67 %** |
| 80,000 lemmas | 147,773 | 2.66 MB | 99.32 % | 96.46 % | 99.23 % | 95.75 % |
| 60,000, every form | 212,718 | 4.15 MB | 99.23 % | 96.44 % | 99.12 % | 95.67 % |

## What one lemma per form costs (M8)

A lemma is keyed by its own form, so a dictionary noun whose form GSD reads as a verb's leaves the
pack: « la porte » shows *porter*'s card. Among wordfreq's 1,000, 2,000 and 5,000 commonest words,
**30, 55 and 135** dictionary nouns read as a verb and are no ranked lemma — Spanish's committed
tables, counted the same way, have 30, 48 and 95 (`cuenta` → *contar*). Among the commonest French
loses what Spanish does; further down it loses more, its past participles being nouns too. An
override row keeps one noun and reads every verb form it shares as the noun; the owner may name
rows, each with its reason, at any reduction. The 135, with their wordfreq rank and GSD's counts
(of the form under the verb / under itself; `données` is the noun *donnée*'s plural, which GSD
counts 40 times under *donnée*):

| Rank | Noun | Read as | GSD |
|---|---|---|---|
| 8 | est | être | 5,150 / 66 |
| 14 | a | avoir | 2,112 / 43 |
| 49 | fait | faire | 428 / 125 |
| 65 | été | être | 830 / 51 |
| 84 | va | aller | 78 / 0 |
| 92 | dit | dire | 99 / 0 |
| 205 | reste | rester | 88 / 37 |
| 249 | demande | demander | 37 / 36 |
| 256 | passe | passer | 53 / 7 |
| 284 | porte | porter | 39 / 23 |
| 318 | passé | passer | 38 / 12 |
| 326 | donne | donner | 62 / 4 |
| 364 | aura | avoir | 32 / 0 |
| 384 | ayant | avoir | 84 / 0 |
| 422 | étant | être | 113 / 0 |
| 500 | écrit | écrire | 66 / 1 |
| 555 | laisse | laisser | 25 / 0 |
| 559 | produit | produire | 48 / 22 |
| 620 | présente | présenter | 57 / 0 |
| 633 | offre | offrir | 23 / 15 |
| 684 | données | donner | 1 / 0 |
| 685 | dû | devoir | 37 / 0 |
| 689 | joue | jouer | 49 / 2 |
| 691 | montre | montrer | 24 / 3 |
| 718 | connu | connaître | 75 / 0 |
| 732 | annonce | annoncer | 22 / 5 |
| 747 | sort | sortir | 17 / 9 |
| 789 | fini | finir | 14 / 3 |
| 813 | reçu | recevoir | 40 / 0 |
| 827 | permis | permettre | 30 / 5 |
| 1,018 | arrêté | arrêter | 15 / 13 |
| 1,042 | rendu | rendre | 24 / 4 |
| 1,055 | change | changer | 10 / 3 |
| 1,076 | suivi | suivre | 34 / 7 |
| 1,097 | signe | signer | 25 / 14 |
| 1,147 | pose | poser | 12 / 5 |
| 1,213 | touche | toucher | 7 / 6 |
| 1,240 | tombe | tomber | 13 / 7 |
| 1,282 | vit | vivre | 23 / 0 |
| 1,411 | envoyé | envoyer | 14 / 1 |
| 1,441 | lance | lancer | 18 / 7 |
| 1,450 | partage | partager | 11 / 10 |
| 1,523 | figure | figurer | 13 / 5 |
| 1,549 | élu | élire | 43 / 2 |
| 1,585 | chargé | charger | 23 / 0 |
| 1,590 | découvert | découvrir | 14 / 1 |
| 1,671 | passant | passer | 17 / 0 |
| 1,686 | tente | tenter | 23 / 1 |
| 1,688 | vivant | vivre | 10 / 7 |
| 1,748 | oublie | oublier | 2 / 0 |
| 1,794 | vécu | vivre | 12 / 0 |
| 1,796 | élève | élever | 14 / 13 |
| 1,859 | conduit | conduire | 15 / 1 |
| 1,952 | estime | estimer | 17 / 4 |
| 1,991 | demeure | demeurer | 18 / 6 |
| 2,019 | tenue | tenir | 9 / 5 |
| 2,034 | disparu | disparaître | 17 / 4 |
| 2,084 | invité | inviter | 7 / 2 |
| 2,113 | blessé | blesser | 11 / 2 |
| 2,166 | commande | commander | 6 / 5 |
| 2,185 | monte | monter | 9 / 2 |
| 2,219 | composé | composer | 30 / 3 |
| 2,336 | réduit | réduire | 8 / 1 |
| 2,346 | vendu | vendre | 18 / 0 |
| 2,383 | remarque | remarquer | 5 / 3 |
| 2,397 | venue | venir | 6 / 3 |
| 2,403 | affiche | afficher | 7 / 5 |
| 2,409 | cache | cacher | 6 / 0 |
| 2,416 | condamné | condamner | 14 / 4 |
| 2,454 | tire | tirer | 5 / 0 |
| 2,464 | cesse | cesser | 7 / 3 |
| 2,478 | frappe | frapper | 9 / 0 |
| 2,514 | donnée | donner | 15 / 4 |
| 2,548 | tiré | tirer | 11 / 0 |
| 2,563 | commis | commettre | 5 / 2 |
| 2,601 | touché | toucher | 7 / 0 |
| 2,605 | acquis | acquérir | 12 / 1 |
| 2,613 | extrait | extraire | 6 / 2 |
| 2,645 | soumis | soumettre | 11 / 0 |
| 2,656 | allant | aller | 17 / 0 |
| 2,713 | employé | employer | 7 / 1 |
| 2,747 | venant | venir | 8 / 0 |
| 2,793 | pousse | pousser | 8 / 0 |
| 2,795 | relève | relever | 3 / 1 |
| 2,805 | établi | établir | 21 / 0 |
| 2,808 | associé | associer | 11 / 2 |
| 2,857 | traite | traiter | 8 / 5 |
| 3,028 | paye | payer | 3 / 0 |
| 3,120 | accusé | accuser | 13 / 0 |
| 3,121 | adopté | adopter | 10 / 0 |
| 3,147 | lis | lire | 1 / 0 |
| 3,149 | lâche | lâcher | 3 / 0 |
| 3,155 | participe | participer | 50 / 0 |
| 3,168 | sachant | savoir | 5 / 0 |
| 3,181 | assuré | assurer | 11 / 0 |
| 3,217 | raté | rater | 1 / 0 |
| 3,236 | applique | appliquer | 6 / 0 |
| 3,261 | frappé | frapper | 5 / 0 |
| 3,309 | abandonné | abandonner | 14 / 0 |
| 3,322 | convaincu | convaincre | 6 / 0 |
| 3,345 | paie | payer | 4 / 0 |
| 3,374 | voyant | voir | 8 / 0 |
| 3,407 | intéressé | intéresser | 4 / 1 |
| 3,458 | gagnant | gagner | 3 / 1 |
| 3,478 | opposé | opposer | 9 / 2 |
| 3,499 | remonte | remonter | 18 / 0 |
| 3,527 | destinée | destiner | 10 / 0 |
| 3,601 | contenant | contenir | 16 / 0 |
| 3,738 | bouge | bouger | 1 / 0 |
| 3,754 | enseigne | enseigner | 6 / 0 |
| 3,766 | jugé | juger | 10 / 0 |
| 3,772 | limitée | limiter | 3 / 0 |
| 3,836 | devoirs | devoir | 2 / 0 |
| 3,917 | finance | financer | 2 / 1 |
| 3,937 | marié | marier | 7 / 0 |
| 3,975 | adapté | adapter | 9 / 3 |
| 3,976 | allée | aller | 3 / 0 |
| 4,089 | poussé | pousser | 2 / 0 |
| 4,180 | restant | rester | 10 / 4 |
| 4,262 | partant | partir | 8 / 0 |
| 4,291 | tenant | tenir | 6 / 2 |
| 4,292 | tombée | tomber | 6 / 0 |
| 4,308 | attaché | attacher | 9 / 0 |
| 4,344 | joint | joindre | 4 / 0 |
| 4,387 | traverse | traverser | 16 / 0 |
| 4,488 | exposé | exposer | 5 / 2 |
| 4,561 | conserve | conserver | 12 / 2 |
| 4,636 | épargne | épargner | 2 / 0 |
| 4,669 | failli | faillir | 3 / 0 |
| 4,727 | aie | avoir | 1 / 0 |
| 4,771 | jeté | jeter | 2 / 0 |
| 4,794 | prévenu | prévenir | 4 / 2 |
| 4,869 | lancée | lancer | 11 / 0 |
| 4,952 | levée | lever | 3 / 2 |
| 4,980 | reproche | reprocher | 4 / 0 |

The rank is wordfreq's own, before the tables skip its stems and inflected forms. Most are a verb
form first in GSD by far; the closest are `demande` (37 / 36), `partage` (11 / 10), `arrêté`
(15 / 13), `élève` (14 / 13), `touche` (7 / 6) and `porte` (39 / 23).

## Function words ranked on their own

Twelve determiners and pronouns are ranked lemmas of their own though another determiner's or
pronoun's table lists them as its feminine or plural: `des` (M21's word of its own), `vous` and
`nous` (listed as `tu`'s and `je`'s plurals), `ils`, `ma`, `mes`, `elles`, `ta`, `tes`, `vôtres`,
`iels`, `celleux`; `cet` is a headword no table points from `ce`. The dictionary gives each an entry
of its own, and GSD's lemmas never name the dictionary's head — it lemmatises every possessive as
*son* (`ma` 54 times, `mon` 66) and `ils` as *lui* —, so the counts cannot decide and the form's own
entry does (D5's third rule); `ma` is also a Louisiana preposition's headword, and two `Ma` proper
nouns in GSD count for it. Spanish's tables merge them (`mis` → *mi*, `ellos` → *él*), English's
keep a pronoun's case forms apart (`them`, `their`). No rule merges them here: a paradigm rule would
read `nous` as *je* and `vous` as *tu*. The rows that would — `ma` → *mon*, `mes` → *mon*, `ta` →
*ton*, `tes` → *ton*, and, as a convention, `ils` → *il*, `elles` → *elle* — are the owner's to name
in `OVERRIDES`, each with its reason (task 5.1); `cet` → *ce* needs a candidate the dictionary does
not give, left to the analysis (change 41) or the grammar (change 45).

## The levels (`../fr/level.tsv`)

**Estimated, not a CEFR list's** (add-lingua-french-levels, the programme's M7). No French CEFR list
can be shipped: FLELex (CEFRLex) is CC BY-NC-SA 4.0, which the builder's licence guard refuses, and
no openly licensed French CEFR word list was found. French's levels are estimated from frequency, as
Spanish's are, and the manifest says `levels_estimated`: the extension labels them estimated and the
ladder borrows English's typical vocabularies, saying so — no code of their own. No FLELex data is
read, measured against or committed.

**The derivation.** The lemmas of `../fr/freq.tsv`, in rank order, take English's level sizes —
1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2: 8,302 lemmas — skipping those a CEFR list
would leave out. The sizes are constants of `reduce-fr-en.py` (`ENGLISH_BANDS`), equal to es-fr's by
a test and never read from English's tables, so an English update cannot move French's levels
unannounced. Where the truth is known, giving English's 8,302 CEFR lemmas these sizes in their own
rank order agrees with the lists for 39.8 % of them and within one level for 82.6 %; each estimated
level's mean true level rises from 1.67 at A1 to 5.03 at C2, so French keeps six levels.

**Which lemmas take one** is read from the English Wiktionary's French section — the source of the
forms, never a pair's glosses — and, since refine-lingua-fr-en-glosses (D3), from French's
dictionary words: a lemma they do not list takes no level, so that a card seeded from a level always
carries a gloss. The table moves with fr-en's glosses only when they add or remove a dictionary word.
Within the levelled span (ranks 1–10,763), 2,461 ranked lemmas take no level:

| Rule | Left out | For example |
|---|---|---|
| 1. the section gives it no sense that is not a form of another word… | 1,347 | `the`, `etc`, `in`, `km`, `http` |
| …or only a name's | 1,026 | `france`, `paris`, `québec`, `facebook` |
| 2. a single character the section gives no word's sense — a letter's name, a symbol, an abbreviation | 16 | `p`, `i`, `h`, `e`, `b`; `à` (a preposition), `y` (a pronoun), `x` (a stool, X-rated) and `ô` (a vocative) keep their place |
| 3. every sense it is given, a name's aside, only spells another word — an obsolete, archaic, rare, dated or alternative spelling, a letter-case form, a misspelling, a pronunciation spelling | 26 | `parceque`, `orient`, `okay`, `hazard`, `isle`, `zombie`, `lys` |
| 4. its own form reads as another lemma in `forms.tsv` | 0 | none: the ranks keep no such lemma; the rule stays as a guard, since the builder keys a level by looking the lemma up as a form (`donnée`, read as *donner*, would give *donner* its level) |
| 5. French's dictionary words do not list it — fr-en glosses it not at all, or by a name's senses alone (read last; refine-lingua-fr-en-glosses D3) | 46 | `parce` (met only in « parce que »), `quant`, `x`, `pme`, `stp`, `expliquez`; `coran`, `satan`, `bcp`, `jo` |

`du` and `des`, words of their own (M21), are A1 whatever their senses say; `au` and `aux` are no
words. Every word the pre-pass writes has a level: the elided pieces' words, `à`, `le` and the
inversion's pronouns are A1, `jusque` A2 and `quoique` B1. M8's cost carries over: `porte` is no
lemma, so the door has no level and « il porte » reads as *porter* (A1).

| Level | Lemmas | Ranks | First words |
|---|---|---|---|
| A1 | 1,020 | 1–1,081 | de, le, et, à, en, des, un, que |
| A2 | 1,158 | 1,082–2,377 | célèbre, davantage, distance, entier, exposition, extrême |
| B1 | 2,015 | 2,378–4,816 | concentration, couteau, doucement, ed, fi, file |
| B2 | 2,347 | 4,820–8,105 | ciné, configuration, cuire, célébration, discrétion, distributeur |
| C1 | 886 | 8,107–9,427 | pitoyable, plasma, postal, propreté, pâtisserie, querelle |
| C2 | 876 | 9,428–10,763 | orphelin, panda, parachute, paramètre, participatif, perruque |

**The dictionary's unmarked spellings** (read-lingua-french-dictionary-unmarked-spellings): `benoit`
(B2), the one of the 33 spellings that had a level, leaves the table with its word; `patriotique`
moves C1 → B2 and `opter` C2 → C1, and `digestif` takes C2's last place: 4 rows, 8,302 levels, as
many at each level as before. *être*, *état*, *siècle* and *répondre* keep A1, *benoît* B1, `diesel`
B1 and `ego` B2. The spans move by at most eight ranks, as the ranks after each spelling move up.
Rule 3 leaves out 26 lemmas instead of 33: `etat`, `etats`, `etre`, `etait`, `etaient`, `derniere` and
`arreter` are no ranked lemmas now; the names rule two fewer (`ecosse`, `etats-unis`).

**The web's unmarked spellings** (read-lingua-french-web-unmarked-spellings) move no row: no spelling
of the list was a levelled word (each was « unknown », rule 1), so `level.tsv` is byte for byte. Rule 1
leaves out 1,347 lemmas instead of 1,396, the spellings that stood in the levelled span gone, and the
spans start up to 49 ranks earlier.

**The web's hyphenated words and wrong accents** (read-lingua-french-hyphenated-and-misaccented-spellings)
move no row either: no misaccented spelling was levelled (each « unknown », rule 1), so `level.tsv` is
byte for byte. Rule 1 leaves out 1,344 lemmas instead of 1,347, and the spans from B1's end start up
to three ranks earlier (B2 4,818–8,102, C1 8,104–9,424, C2 9,425–10,760).

**French's unmarked spellings** (read-lingua-french-ca-as-ca-cedilla): `ca` (A1), `age` (B1) and
`forcement` (C2) leave the table with their words; eight lemmas move one band up — `croissance`
A2 → A1, `communiste` B1 → A2, `australien` and `boue` B2 → B1, `naïveté` and `parano` C1 → B2,
`morphologie` and `nantais` C2 → C1 — and `crabe`, `cuisinière` and `dar` take C2's last places:
14 rows, 8,302 levels, as many at each level as before. `ça`, `âge` and `forcément` keep A1, A1 and
A2. The spans move by at most three ranks (B2 from 4,836, C1 from 8,141; they were 4,833 and 8,140).

**Levels given to dictionary words alone** (refine-lingua-fr-en-glosses D3): against change 46's
table, 45 lemmas lose their level — 36 fr-en does not gloss (`parce`, `x`, `quant`, `to`, `mm`,
`for`, `pp`, `tom`, `com`, `inter`, `rio`, `av`, `encontre`, `fur`, `pme`, `po`, `am`, `ong`,
`instar`, `ken`, `ep`, `bo`, `app`, `cie`, `pass`, `stp`, `caf`, `expliquez`, `sp`, `rc`, `ht`,
`rsa`, `tnt`, `rip`, `tpe`, `nc`) and 9 it glosses as names alone (`pq`, `jo`, `coran`, `bcp`,
`satan`, `vo`, `cb`, `cac`, `mao`); rule 5 also names `asm`, past the old span —, 45 gain one at C2's end (ranks 10,763–10,820:
`sous-titre`, `sèche-cheveux`, `tire-bouchon`…), and 122 move one band up (5 A2 → A1, 10 B1 → A2,
23 B2 → B1, 40 C1 → B2, 44 C2 → C1): 212 rows, 8,302 levels, as many at each level as before. The
four levelled lemmas of change 48's 40 that fr-en now glosses (`french`, `burger`, `dev`, `ès`, by a
pointer that carries its meaning) keep their level. The figures below are change 46's, measured on
its table.

**Measured with open data only** (design D4), Spanish's committed estimate the control:

| | French | Spanish |
|---|---|---|
| A levelled lemma's translation — the first word of its English Wiktionary glosses that English's CEFR lists level — and that word's level: exact / within one | 28.3 % / 66.7 % of 5,949 (71.7 % covered) | 29.4 % / 65.9 % of 6,140 (74.0 %) |
| Mean English level of the translations, A1 → C2 | 1.80, 2.40, 2.75, 3.06, 3.12, 3.06 | 1.83, 2.37, 2.74, 3.04, 3.04, 3.08 |
| A1 words whose translation is English A1 / A1–A2 | 51.6 % / 76.5 % | 49.6 % / 73.8 % |
| UD PUD's words at A1, A2, B1, B2, C1, C2, none (gold lemmas; names, numbers, symbols, punctuation left out) | 78.4, 7.8, 5.7, 2.7, 0.6, 0.6, 4.3 % | 76.6, 8.1, 6.0, 2.6, 0.8, 0.4, 5.5 % |
| PUD's words presumed known when declaring A2, B1, B2, C1, C2 | 78.4, 86.2, 91.9, 94.6, 95.2 % | 76.6, 84.7, 90.7, 93.3, 94.1 % |

English's lists on the same PUD sentences: 65.8, 12.2, 9.0, 3.9, 0.5, 0.2, 8.3 %, a B1 reader
presumed to know 78.0 % of the words. The estimate presumes more below each level than English's
lists, in French as in Spanish. The translation proxy rises from A1 to B2 and cannot tell B2, C1 and
C2 apart in either language: a rare word's translation is often a common English word.

**Against the design's figures**, measured on change 43's prototype ranks: every proxy holds to the
tenth, and the spans end a few ranks earlier — A2 at 2,374 (2,376), B1 at 4,809 (4,814), B2 at
8,081 (8,086), C1 at 9,412 (9,418), C2 at 10,762 (10,768). Seven lemmas the prototype ranked within
the span are no ranks of these tables: `venue`, `donnée`, `saisie`, `tranchée` and `revenue`, whose
own form reads as a verb, and `ç` and `jusqu`, wordfreq's elision stems. So 2,460 lemmas are left out
instead of 2,466 — rule 4 finds none of its four, the letters lose `ç`, the unknown words `jusqu` —,
the translation proxy covers 5,949 lemmas instead of 5,950, and four levels differ from the
prototype's table: `revenue` (B2) is no lemma, so `confidentialité` moves up to B2, `cheveu` to C1,
and `sous-préfecture` takes C2's last place. On the prototype's ranks the reduction gives the
design's table byte for byte.

**Departures from Spanish's outcome.** Rules 2 and 3 are French's: Spanish's rule (a French gloss
that is not only a proper noun's) levels 20 single characters, 15 of them letters rather than words
(`b`, `d`, `h`, `k` A1, `w` A2, `z` B1), which « Renforcer un niveau » can seed. Without them French's
A1 would hold 11 letters and, before read-lingua-french-dictionary-unmarked-spellings read them as
their word, `etat`, `etats`, `etre`. Most of the 26 spellings spell a word with a level of its own
(`hazard` → *hasard*, `parceque` → *parce que*); some spell none the table levels (`orient`,
`méditerranée`, `zombie`, `ndlr`, `latino-américain`) and lose a level with nothing in its place.
Spanish's table does not move; aligning it, or narrowing rule 3 to a spelling of a commoner word, is
the owner's question (task 5.2).

**Known weaknesses**, as for Spanish: numbers written as words are levelled by frequency (`onze`,
`treize`, `soixante`, `dix-sept` B1, `vingt-quatre` B2, `dix-neuf`, `soixante-dix` C2, `trente-deux`
none), where English's lists put them at A1; a word ranked by a name's frequency keeps its rank
(`jean` A1, `twitter` A2); a noun said mostly in the plural ranks by its singular (`cheveu` C1, `œil`
A2). On change 46's table C2 ended inside a block of 326 ranked lemmas sharing Zipf 3.41, most of
them compounds GSD meets once: 209 of C2's 876 lemmas were in it, 125 of them compounds, and its 27
lemmas past the edge had no level. C2 now ends at rank 10,760 (10,820 before read-lingua-french-dictionary-unmarked-spellings, 10,812 before read-lingua-french-web-unmarked-spellings, 10,763 before read-lingua-french-hyphenated-and-misaccented-spellings). A C2 word is presumed known by no declared level, so only the C2 ladder and C2 seeding see
it.

**If a licence is granted** (design D8): the owner asks UCLouvain's CENTAL for FLELex, with ELELex,
for deriving a lemma → level table, committing it in this public repository and shipping it in the
packages for commercial use. Then `reduce-fr-en.py` reads FLELex at a pinned version — each lemma the
first level at which the list attests it, the lowest on collisions, joined to French's lemmas through
`forms.tsv` —, the manifest drops `levels_estimated`, the NOTICE credits the list under the granted
terms, the licence guard admits the grant as a category of its own, and fr-en and fr-es are reduced
again in one pull request. This estimate is measured against FLELex then, and the figures kept here.

The levels section is one byte per lemma of the pack's pool: the pack grows from 1,241,733 B to
1,302,031 B — 60,035 B the section and the manifest's flag, 263 B the NOTICE's sentence.

## The readings (`../fr/grammar.tsv`)

French's word grammar (add-lingua-french-grammar-tables, change 45): each form's readings, in the
vocabulary the word card reads (add-lingua-word-grammar), written by the same pass over the French
section as the forms, once they are chosen — the readings choose no form and no rank. Only the forms
`forms.tsv` holds carry readings, and only under the lemmas `freq.tsv` ranks: a card opens on a form
the analysis resolved through the table.

**The rules** (`reduce-fr-en.py`, after the forms):
- **Verbs.** kaikki's tags as UD's: the indicative's present, imperfect and future; the passé simple
  (`historic past`) as `Tense=Past`; the present and imperfect subjunctive; the conditional and the
  imperative with **no tense**, as Spanish's tables write them (UD French writes `Tense=Pres`); the
  infinitive; the present participle as `VerbForm=Part|Tense=Pres`, as UD French writes it; the past
  participle with its gender and number — a verb's table lists it bare, the masculine singular, and
  the agreed forms hang under the participle's own entry (`dirigé`: `dirigée`, `dirigés`,
  `dirigées`). Every verb is `VERB`. A sense merging persons or moods (`parle`: « first/third-person
  singular present indicative/subjunctive ») reads as each of them; a compound tense
  (`avoir + past participle`) gives none. A pronominal verb's rows read without their pronoun
  (`s'évanouit` → `évanouit`, `nous évanouissions` → `évanouissions`, `évanouis-toi` →
  `évanouis`), as French's pre-pass leaves the bare form: 12,023 rows.
- **Nouns, adjectives, determiners (articles among them), pronouns, numerals.** A noun's gender from
  `fr-noun`'s head, else its senses, on its own form and on its plural, a reading per gender for a
  noun of both (`enfant`); both numbers on a noun the dictionary gives one form for (`temps`, `bras`,
  `vis`: 549 readings); no reading from a feminine noun's masculine row (`déesse` → `dieu`). An
  adjective's own form is masculine singular when it has a feminine of its own (`grand`), singular
  when one form serves both (`rapide`), plural when every sense is (`plusieurs`); `bel` is a
  masculine singular. A determiner, pronoun or numeral reads its gender and number (`la` is *le*'s
  as the article and as the pronoun), a pronoun's row only with a gender (`ils` is no plural of
  *il*), and a plural-headed determiner's or pronoun's table gives none (`tes` lists `ton`). A
  determiner's, pronoun's or numeral's dictionary form names nothing on its own card.
- **Where a reading comes from.** A lemma's table first, a form's own entry for the pairs no table
  lists. A form of a form along one part of speech reads as the word the form is a form of reads,
  with its own agreement (`dirigée` → `dirigé` → *diriger*'s feminine past participle; `faites`
  adds *faire*'s feminine plural participle), through the form's own lemma too, unless the form
  already reads as that word in that part of speech and verb form (`les` is no feminine through
  `la`). A spelling variant reads as the word it spells (`coeurs` → *cœur*), and so does a spelling
  the dictionary gives only as a marked word (`repond` → *répondre*, `siecles` → *siècle*;
  read-lingua-french-dictionary-unmarked-spellings). A row of `UNMARKED_SPELLINGS` carries the readings
  of every form of its word it writes without their marks (Unicode's combining marks dropped; `œ` is
  no mark), the form re-keyed, its mark kept (read-lingua-french-web-unmarked-spellings D3): `recu`
  those of `reçu` (*recevoir*'s past participle), `arrete` those of `arrête` and `arrêté` together,
  `etais` *être*'s imperfect and, as `étais` does, the plural of *étai* (`other`), `ecoute` the noun
  *écoute* and *écouter*'s present (`other`), `sure` *sûr*'s feminine singular and *sur*'s as `other`,
  `age` *âge*'s. A row written with marks (read-lingua-french-hyphenated-and-misaccented-spellings D4)
  compares the letters with every mark dropped and, when some of those forms keep one of its marks in
  place (a marked letter at the same place, whichever its mark), takes only those keeping the most:
  `arreté` *arrêter*'s past participle alone (`arrêté`), not the present `arrête`, `agé` the adjective
  `âgé`, `régle` *règle*'s noun and *régler*'s present (`other`), as `règle` carries, `espére` both
  `espère`'s present and `espéré`'s participle; a hyphenated row its form's (`grand-mere` the feminine
  singular noun, `celle-la` the pronoun's feminine singular).
- **A reading's part of speech is one its lemma's entries hold**, when the dictionary holds the lemma
  as a noun, verb, adjective, determiner, pronoun or numeral: a participle filed under a noun's or
  an adjective's spelling reads no verb form of it and names its verb (`cités`, the plural of the
  noun *cité*, names *citer*; `privée` *priver*), and `venait` names no *came*, an English gloss its
  entry lists that the section holds as a noun.
- **Never a reading** from a row or sense change 43 leaves out of the forms (alternative, obsolete,
  archaic, rare, dated, misspelt, nonstandard, proscribed, abbreviated, a multi-word construction…),
  from a form's own sense marked as a region's or a register's (`été`, Louisiana's past participle of
  *aller*), from a capitalised headword (`CE`, `LE`: 3,592 entries), from an entry whose every sense
  is an alternative form or a neologism (`estre`, archaic spelling of *être*, whose table would make
  `est` its form; `lea`: 2,548 entries), from a letter's name toward its plural (`elle`, the letter
  L), or through a link an override row of change 43 sets aside as a copy error: `fatiguée`'s verb
  entry reads « feminine singular of parlé », and no reading of it names *parler* (an overridden
  form's own entry linking it to another word than the row's, which no lemma's table lists).
- **`other` marks (M8).** A form's readings of its own lemma are its own; a reading of another
  ranked lemma is marked `other`, and the card names that word without counting it (`fils` names
  *fil*, `couvent` *couver*, `vis` *vivre* and *voir*), only toward an entry of the dictionary that
  is not only regional: `irait` names no *would*, `va` no *vader* (Louisiana's, Switzerland's),
  `entrainait` (*entraîner*'s) no *entrainer*, a spelling the section holds no entry of.

**What they hold**, on the 2026-10-09 tables: **126,389 readings of 89,359 forms**, in 79 tags —
117,102 under the form's own lemma, which 89,214 forms carry, and **9,287 `other` marks** on 4,909
forms. Of the 125,177 readings before read-lingua-french-dictionary-unmarked-spellings, 116,561 came
from a lemma's table, 484 from a form's own entry, 8,132 from a form of a form. By part of speech:
71,150 verb readings, 35,404 noun, 19,751 adjective, 40 determiner, 30 pronoun, 14 numeral. Own
readings per form:

| Readings | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| Forms | 70,195 | 15,092 | 1,491 | 102 | 2,162 | 172 |

**The dictionary's unmarked spellings** (read-lingua-french-dictionary-unmarked-spellings) moved them
from 125,177: out, the 28 readings filed under the spellings' own words, `rebellions`' as *rebeller*'s
and `désossé`'s as *désosser*'s; in, the moved forms' readings under their marked words (`siecles`
under *siècle*, fourteen of *répondre*'s, nine of *gâcher*'s) and the entering lemmas'. `tags.tsv` is
byte for byte: every new reading's tag is in the pinned pool.

**The web's unmarked spellings** (read-lingua-french-web-unmarked-spellings) moved them from 125,303:
in, **870 readings the spellings carry** — 867 for 442 of the list's 468 rows (the 26 others are
adverbs and prepositions, `tres`, `deja`, `apres`, `plutot`, `evidemment`, and `alleluia`, whose words
carry none either) and 3 for 43b's rows (`age`, `ages`, `ca`; `forcement` none) — and 224 of the
lemmas entering the cut; out, 8: `sure`'s own reading under *sur* (now `other`, its own *sûr*'s), the
`other` readings `sure`, `surs`, `sures` and `entrainant` had toward the accented word (now their
own), `entrainante`'s and `entrainantes`' under `entrainant` and `patisseries`' under `patisserie`.
`tags.tsv` is byte for byte. 35,528 forms have no reading of their own after it (35,581 before).

**The web's hyphenated words and wrong accents** (read-lingua-french-hyphenated-and-misaccented-spellings)
moved them from 126,389 to **126,554**: in, **109 readings for 64 of the 84 rows** (11 for 8
hyphenated rows; the 20 without are adverbs and pronouns whose word carries none either — `déja`,
`trés`, `aprés`, `peut-etre`, `lui-meme`, `c'est-a-dire`, `oú`) and 57 of the lemmas entering the cut;
out, 1: `épaississant`'s participle under *épaissir* (a lemma of its own now). `tags.tsv` is byte for
byte.

The 35,578 forms without a reading of their own (35,583 since French's unmarked spellings, which add
`age`, `ages` and `forcement` — each read as a word whose readings are not written for it — and the
names `cussac` and `céphalonie`, and take `forcements` out; 35,580 since the dictionary's unmarked
spellings) are, for 35,308 of them, forms of a lemma the
dictionary holds as none of the parts of speech read (adverbs, prepositions, conjunctions,
interjections, names); the rest are a determiner's, pronoun's or numeral's dictionary form (`le`,
`je`, `deux`), old spellings the dictionary marks doubtful, and 18 forms whose readings the
part-of-speech rule takes off their lemma (`croisée`, filed under the noun *croisé*, names
*croiser*; `vingt-et-unième`, an ordinal filed under the numeral *vingt*, reads nothing). The 60,000
lemmas hold 3,339 verbs, 3,285 of them with all 45 simple finite cells in their table; those tables
list 151,363 form–cell pairs, 53,080 of them (35.07 %) forms the table holds — the rest are forms
wordfreq never met (`vînmes`). 17,122 of the 17,132 ranked nouns carry a gender on their own form.

**M8's cost.** A dictionary noun whose own form the forms table reads as another word is no lemma of
the tables, so none of its readings is kept: 551 nouns, 296 of them forms of one of the 5,000
commonest lemmas (`fait`, `été`, `porte`, `demande`, `élève`). « La porte » opens *porter*'s card
with its five verb readings.

**The five-reading forms (M21).** `parle` carries five readings, each its own tag — present
indicative and present subjunctive, first and third persons singular, and the imperative's second
person singular —, all under *parler*: **2,141 forms of 2,110 verbs** (40 of the 1,000 commonest
lemmas, 350 of the 5,000), 180 of PUD's 3,282 verb words and 92 of GSD test's 1,180. The two
presents carry `Tense=Pres` and the imperative none, so the card, which already merges the persons of
one tense and number, is left three groups, and the moods' merge — the word card's, change 51 —
joins the two presents and leaves the imperative apart. No flag marks them: a five-reading form is
recognised from its readings. 6,145 forms read the indicative and the subjunctive of one tense,
person and number (the 2,141, 2,728 third persons plural, 1,080 second persons singular, 162 like
`finissent`, 33 like `finissions`, one other): whether change 51 merges those too is its call.

**The tag pool** (`../fr/tags.tsv`): the 79 tags the readings carry, in byte order, written once by a
person (`cut -f3 grammar.tsv | LC_ALL=C sort -u`) and kept by every reduction (`pack_sources.py
KEPT_INPUTS`); a tag a later reduction adds is appended after them, so that a pinned tag never
changes index. Every pack studying French lays its pool out as the pin, then any reading tag it
lacks, then the tags only its senses carry: fr-en's and fr-es's readings are stored alike whatever
their glosses' senses carry. The pin moves no byte today — fr-en built with the empty pin had the
same sha256. `crates/lingua-pack/tests/committed_tables.rs` holds it to the readings, naming a tag
missing or left over.

**Size.** `grammar.tsv` is 7,936,427 B (752,507 B gzipped); with `forms.tsv` and `freq.tsv`, French's
three tables hold 11,038,422 B (1,636,173 B gzipped, file by file). The readings add 158,222 B to the
pack, which is 1,460,253 B with the levels (1,399,955 B on the forms and ranks alone).

**Measured** on the held-out treebanks (`measure/fr-ud.sh` → `measure/fr_readings.py`, reported,
never gating): over the words whose form the tables map to the treebank's lemma, the share that carry
a reading of their own and the share of those whose treebank part of speech and features are among
them — an auxiliary read as a verb, the conditional's and the imperative's tense left aside, a
participle without a tense read either way.

| | Words | Read | Agree |
|---|---|---|---|
| PUD, finite verbs | 1,742 | 99.94 % | 99.60 % |
| PUD, participles | 973 | 100 % | 99.38 % |
| PUD, infinitives | 466 | 100 % | 100 % |
| PUD, nouns | 4,478 | 99.17 % | 98.85 % |
| PUD, adjectives | 1,473 | 98.85 % | 96.43 % |
| PUD, determiners | 3,587 | 61.36 % | 85.42 % |
| PUD, pronouns | 522 | 16.67 % | 27.59 % |
| GSD test, finite verbs | 637 | 100 % | 99.53 % |
| GSD test, participles | 302 | 100 % | 99.34 % |
| GSD test, infinitives | 190 | 99.47 % | 100 % |
| GSD test, nouns | 1,776 | 98.48 % | 98.68 % |
| GSD test, adjectives | 560 | 98.93 % | 95.67 % |
| GSD test, determiners | 1,350 | 57.85 % | 78.10 % |
| GSD test, pronouns | 292 | 20.89 % | 19.67 % |

A determiner's or pronoun's dictionary form names nothing, hence their « read » column; what they
read is mostly a dictionary noun's spelling — `un` (the noun « un »), `son` (« sound »), `nous` —,
whose readings are the noun's, hence their « agree » column. Among verbs, what disagrees is mostly
the treebank's own annotation (`prit` and `crée` as participles, `attirerait` as an imperfect).

The nouns' rows moved with the dictionary's unmarked spellings (PUD 4,482 words, 99.09 % read;
GSD's test 1,772, 98.65 %): PUD's four « Etats-Unis » no longer take its lemma, GSD's « Etat » and
« Etats » do.

**Each rule, switched off alone** (on the tables before read-lingua-french-dictionary-unmarked-spellings;
PUD's figures move as shown):

| Rule switched off | Readings | Forms with one | `other` | Five-reading forms | On PUD |
|---|---|---|---|---|---|
| — (the rules) | 125,177 | 88,518 | 9,088 | 2,141 | — |
| a reading's part of speech its lemma's | 126,495 | 88,536 | 9,051 | 2,141 | adjectives agree 96.43 → 96.57 % |
| a form of a form | 117,038 | 81,649 | 8,124 | 2,141 | participles read 100 → 75.44 %, agree 99.38 → 98.09 % |
| both numbers on an invariable noun | 124,628 | 88,518 | 9,088 | 2,141 | nouns agree 98.85 → 97.79 % |
| spelling variants | 124,662 | 88,095 | 9,118 | 2,128 | nouns read 99.09 → 99.02 % |
| merged senses read as each | 125,097 | 88,503 | 9,088 | 2,130 | — |
| a pronominal verb without its pronoun | 125,127 | 88,484 | 9,079 | 2,141 | infinitives read 100 → 99.79 %; 37 fewer full paradigms |
| a participle sense without `past` | 125,162 | 88,507 | 9,087 | 2,141 | — |
| articles as determiners | 125,171 | 88,518 | 9,085 | 2,141 | determiners agree 85.42 → 9.59 % |

| Exclusion switched off | Readings | Forms with one | `other` | On PUD |
|---|---|---|---|---|
| capitalised headwords read | 126,684 | 88,766 | 9,123 | nouns agree 98.85 → 99.26 %, adjectives 96.43 → 96.23 %, determiners 85.42 → 83.93 % |
| alternative-only and neologism entries read | 125,874 | 88,958 | 9,189 | nouns agree 98.85 → 98.81 % |
| `other` toward a link-only or a regional word | 125,348 | 88,518 | 9,259 | — |
| a feminine noun's masculine row read | 125,323 | 88,524 | 9,228 | — |
| a composed reading over a direct one | 125,192 | 88,518 | 9,091 | — |
| a letter's plural read | 125,190 | 88,518 | 9,089 | — |
| a pronoun's form without a gender read | 125,185 | 88,520 | 9,093 | pronouns read 16.67 → 23.37 %, agree 27.59 → 48.36 % |
| a plural-headed table read | 125,179 | 88,518 | 9,090 | — |
| a regional sense of a form's entry read | 125,180 | 88,518 | 9,091 | — |
| a link an override row sets aside | 125,178 | 88,518 | 9,089 | — (`fatiguée` names *parler*) |

**Known data defects**, left as the source writes them: `supe` reads as the verb *super* (« to
sip »), one of the 40 five-reading forms among the 1,000 commonest lemmas; `meilleure` reads as the
noun *meilleur* only, the adjective being a form entry of *bon*; `quelques-unes`, « feminine of
quelques-uns » with no number, reads as a feminine singular of *quelqu'un*.

## The glosses (`gloss.tsv`, `senses.tsv`, `mwe.tsv`)

French glossed in English (add-lingua-pack-fr-en, change 48), from the **English Wiktionary's French
section alone** — the file the forms come from, at the same pinned snapshot: fr-en was reduced again
from change 43's pin, nothing fetched that it does not record, `../fr/` byte for byte but its
dictionary words.

**The rules** (`reduce-fr-en.py`, after the forms, ranks, readings and levels; no shared module
edited):
1. **The section, cut** (`native_fields`): each entry's word, part of speech and senses (glosses,
   tags, pointers); a typographic apostrophe in a headword read as `'`, as the forms are
   (`nombre d’oxydation`), its case kept.
2. **The English edition's pre-passes**, in es-en's order: a letter's entry and a sense naming a
   letter left out (`elle`, the letter L), and the words written under a single capital letter
   (`X` « X-frame stool », `C` « abbreviation of cavalier »); the senses read as meanings and in one
   English typography (`english.read_as_meanings`, refine-lingua-es-en-glosses); a word's
   etymologies merged as the edition's setting says (off).
3. **French's expressions** (`expression_senses`, `split_words`): in an expression, a sense that only
   names another spelling (« post-1990 spelling of »), an inverted form (« subject-inverted form
   of ») or the pieces it is written with (« que + elle ») is no gloss, a meaning written after it
   stays (« y a-t-il » « is there? are there? »); `à la` is left out by name (`LEFT_OUT`): its one
   sense, « in the style of », needs the word after it, and the section writes those uses as entries
   of their own (`à la carte`, `à la maison`). The single words French's tokenisation splits — an
   apostrophe or a hyphen, a French token, no form of `../fr/forms.tsv` and no name (`d'abord`,
   `c'est`, `allez-y`) — are offered as expressions, read with the shared rules' sense rules; the
   builder keys those that read as two tokens or more (add-lingua-french-expression-keys).
4. **The shared rules** (`reduce_common.native_tables`) over the 60,000 ranked lemmas: eight whole
   senses grouped by part of speech, a form-only lemma lending its base's senses, an acronym's entry
   sparing the common word; the expressions' three senses. Every glossed lemma is a ranked lemma its
   own form reads as (`crates/lingua-pack/tests/committed_tables.rs`): the builder files a gloss at
   the lemma's own place, and no gloss is lent to another word — the design's prototype, on change
   43's tables before its fix, glossed *venir* « coming, arrival », `venue`'s.

5. **fr-en's own rules** (refine-lingua-fr-en-glosses), in `reduce-fr-en.py` and, for the treebank's
   part of speech, `reduce_french_treebank.py` — a rule module of its own, in fr-en's digest, that
   fr-es is to read too (refine-lingua-fr-es-glosses D8) — the English edition's and the shared rules
   read es-en's rows and every pair's too, and are not edited, so only fr-en re-pins (D1):
   - `read_as_french`, a pre-pass after `read_as_meanings` and before the etymology merging:
     **a pointer that carries its meaning** is read as that meaning (D4) — the section's `extra`
     for its target, else its quoted text, else its text past a colon or a semicolon —, for a word
     only when it names a degree of comparison, a synonym, a plural or a contraction (« des »
     « some; of the, from the, some », « mieux » « better; best; … », « ouais » « yeah, yep… »),
     never a female equivalent (`directrice` keeps `directeur`'s « director; school principal »:
     the owner, 2026-10-10), an alternative form, a spelling, an ellipsis or a clipping; for an
     expression whatever it names (« il y a » « there is, there are; ago »); never in a name's or an
     acronym's entry, never a meaning in capitals only. **A function word's row** (`ADP`, `DET`,
     `PRON`, `CCONJ`, `SCONJ`, `PART`, `ADV`) **or a row the page opens on a name** opens on the
     part of speech UD French-GSD reads the word as at least ten times and twice as often as the
     page's first, never a proper noun's (D5: `pas` on its negation, `son` « his, her… », `leur`
     « their », `bien` « well », `quand`, `juste`, `pendant`, `outre`, `envers`; `marche`,
     `réunion`, `somme`, `restauration` no longer open on a place; with D4, `du`'s article before
     its contraction); a noun, verb or adjective ahead
     of another stays as the page writes it (`ferme` « firm », `mort` « dead », `devoir` « duty »:
     the owner). **No name under a function word** (D6: `le` « a surname from Vietnamese », `on`
     « a village in Luxembourg »). **The page's notes out** (D8: « see usage notes », « (all
     senses) », « in its various senses », « (Folk etymology: …) »), fr-en's description openers in
     lower case (« Substitutes », « Impersonal », « Followed »…), a quotation's citation cut, a
     source's sense number out.
   - Five expressions whose one sense needs a context their key does not hold are left out by name
     (`LEFT_OUT`, each with its reason counted in GSD's text: `et des`, `que de`, `sur ce`, `et
     si`, `un coup`), and six post-1990 spellings keyed apart from their traditional spelling lend
     its gloss (`traditional_spellings`: `à priori`, `à postériori`, `et cétéra`, `sur son
     trente-et-un` and two verbs on it) (D7).
   - Eleven expressions D4 would make of no meaning are left out by name too (`LEFT_OUT`, settled by
     the owner on 2026-10-10 after the implementation found them): the Louisiana spellings `à le`,
     `à les` « to the », which every « au »/« aux » would meet once the pre-pass reads it `à` +
     `le`/`les`, and `de le`, `de les` « “of the”, some », which « décidé de le faire » would meet;
     `j'suis` and `ç'a`, which every « je suis » and « ça a » would meet; `l'a` « him/her/it » and
     `n'ai` « not », one piece's meaning; `j't'à` « the 't' is epenthetic » and `poser des lapins`
     « frequentative or plural », grammatical notes; `point d'inflexion`, half a definition. `t'as`
     « you've » and `t'es` « you're », met only where they are written so, stay.
   - « etc » takes its period back after the shared rules (`with_etc_period`, D8).
   - **French's dictionary words** (`dictionary_words`, D2): the glossed lemmas less the 3,581 every
     sense run of which is a proper noun's, written to `../fr/lexical.tsv`; the levels read them
     (*The levels*).

**fr-en's own rules, each alone and together** (the design's *Measured*, re-measured with the
reducer's own code on the committed tables, its rules switched off but one; with none, the tables
before them byte for byte):

| Rule | Rows / top 10,000 | First sense / top 10,000 | Lemmas gained | Expressions changed / gained / left out |
|---|---|---|---|---|
| D4, a pointer's meaning (the eleven of its expressions left out by name) | 40 / 18 | 13 / 10 | 12 (4 of the top 10,000) | 15 / 155 / 0 |
| D5, the treebank's part of speech | 13 / 13 | 13 / 13 | — | — |
| D6, no name under a function word | 2 / 2 | 0 / 0 | — | — |
| D7, left out by name | — | — | — | 0 / 0 / 5 |
| D7, post-1990 spellings lent | — | — | — | 0 / 6 / 0 |
| D8, notes | 106 / 36 | 96 / 28 | — | 3 / 0 / 0 |
| D8, openers | 14 / 8 | 8 / 2 | — | 1 / 0 / 0 |
| D8, citations and sense numbers | 4 / 3 | 2 / 2 | — | — |
| D8, « etc. » | 117 / 42 | 59 / 10 | — | 7 / 0 / 0 |
| **Together** | **291 / 117** | **190 / 64** | **12, none lost** | **26 / 161 / 5** |

The 12 lemmas gained are words whose only senses were pointers of D4's wordings: `french`, `burger`,
`dev`, `ès` (four of change 48's levelled lemmas with no gloss, which keep their level), `ive`,
`because`, `blockchain`, `chui`, `sherry`, `axis`, `loix`, `broyeuse`. Every figure is the design's,
its expressions gained as amended when the owner left the eleven out (166 → 155 by D4, 172 → 161
together). A few read oddly and are in the owner's sample (task 6.2): « matelas » gains
« French tacos », « bercy » « drunkard ». « au marché », « Personne au village » and « Il a décidé de
le faire » meet none of the eleven (« de le faire » meets `de l'`, which `main` already glossed: a
separate change makes an elided expression piece match an elided token only, the owner decided on
2026-10-10).

**No translation table glosses a word** (D3). The French Wiktionary's English translations and the
English Wiktionary's French translations read backwards were measured on the design's prototype and
declined for words: they would add 1,988 lemmas, 1,895 of them words the section has no entry for — English
words in French text (« in », « end » « NDE, NDI, NDT »), names, initialisms, unaccented misspellings,
1,193 listing the word itself as its translation — and fr-en, French's reference pair, would make
them French's dictionary words, counted by every French reader's vocabulary estimate and kept as
words by the names rule. Read backwards, an English entry makes French's commonest bigrams
expressions (« il est » "he's", 782 times in UD French's 425,111 words): the English Wiktionary's
French translations are no longer derived. A word the section does not gloss has no gloss. The
French Wiktionary's English translations complete the expressions since
add-lingua-fr-en-translated-expressions, the owner's decision of 2026-10-10 on that design's Open
Question 1 (*The French Wiktionary's expressions*).

**What they hold**, on the 2026-10-10 tables:

| | |
|---|---|
| Glossed lemmas | **30,147**, every one from the section's own entries: 30,146 ranked lemmas and the acronym `CA`, the one lemma glossed and unranked |
| French's dictionary words (`../fr/lexical.tsv`) | **26,555**: the glossed lemmas less the 3,592 glossed by a proper noun's senses alone (367 / 916 / 1,775 of the 5,000 / 10,000 / 20,000 commonest); `CA` among them, unranked, which the vocabulary estimate does not count |
| Of the 5,000 / 10,000 / 20,000 commonest lemmas | **4,693 / 8,716 / 15,348 — 93.9 / 87.2 / 76.7 %**; floor 91.9 / 85.1 / 74.4 (`gloss_coverage.py FLOORS`, the study's 93.9 / 87.1 / 76.4 less two points, held by the reduce job; settled by the owner) |
| Expressions | **43,542**: the section's 17,635 — 15,653 headwords with a space and 1,982 words the tokenisation splits — and the French Wiktionary's 25,907 — 23,141 and 2,766 (*The French Wiktionary's expressions*) |
| `gloss.tsv`, `senses.tsv`, `mwe.tsv` | 1,619,095, 513,722 and 1,909,492 B (626,644, 125,999 and 701,450 B gzipped; the section's expressions alone 833,953 B); `../fr/lexical.tsv` 254,526 B |
| The pack | **3,004,986 B**: change 48's 2,527,222 B, fr-en's own rules +2,652 B (the glosses −96, their runs +315, the expressions +2,433), French's lexical table +7,512 B, French's unmarked spellings +202 B (the glosses +437, their runs −224, the expressions +106 — `velours côtelé` « corduroy » keyed now that `côtelé` is a lemma —, the forms, lemmas and paradigms −117), French's acronym table −1,840 B (`CA` takes lemma id 0 and every other id moves by one, which the forms map, −746 B, and the paradigms, −750 B, encode slightly tighter; the glosses −348 B, their runs −5 B; the new rows of the lemmas, ranks, levels and dictionary words +9 B), every headword of a key several reach +954 B (pick-lingua-expression-written-as-the-page: the 61 other headwords of 60 keys, their glosses +35 B and names +411 B, and their list, `expr.others`, 492 B), the dictionary's unmarked spellings +1,870 B (the forms map +1,097, the paradigms +944, the lemmas +7, the glosses −89, their runs −89), the web's unmarked spellings +11,938 B (the forms map +5,115, the paradigms +4,510, the glosses +1,226, their runs +582, the expressions +586 — keyed on the new lemmas, no headword gained or lost —, the lemmas −81); the French Wiktionary's expressions +454,476 B (`mwe.tsv`'s 25,907 rows, the notice's paragraph) |

**French's unmarked spellings** (read-lingua-french-ca-as-ca-cedilla) took three glosses out — `ca`
« board of directors » (`CA`'s initialisms, lowercased), `age` « beam (central bar of a plough);
shaft », `forcement` « fixing number, cooking the books » — and three lemmas entering the cut brought
theirs: `cussac` « a surname from Occitan » and `céphalonie` « Cephalonia… », names, and `côtelé`
« ribbed », a dictionary word. `mwe.tsv` is byte for byte: no headword holds `ca`, `age` or
`forcement`; `velours côtelé`, which had no key while `côtelé` was no lemma, enters the pack.

**French's acronym table** (keep-lingua-french-acronyms-in-capitals D4) glosses `CA` from fr-en's own
glosses of its expansions, each cut to its first sense, in the row's order — `conseil
d'administration` « board of directors », `chiffre d'affaires` « revenue, turnover, net sales »,
`courant alternatif` « alternating current » (« domestic electricity supply » cut); `comptable agréé`,
which fr-en does not gloss, passed over —: « board of directors; revenue, turnover, net sales;
alternating current », `NOUN:3`. Every other row of `gloss.tsv` and `senses.tsv`, and `mwe.tsv`, is
byte for byte; coverage is unchanged (it counts ranked lemmas).

**The dictionary's unmarked spellings** (read-lingua-french-dictionary-unmarked-spellings) took 29
glosses out — the 25 dictionary words that were spellings (`arreter`, `benoit`, `cameraman`,
`colere`, `comparaitre`, `cooperation`, `ecossais`, `egyptien`, `electro`, `etat` « state,
condition… », `etre` « to be… », `experimental`, `gacher`, `haiku`, `icone`, `matiere`, `methode`,
`pecho`, `poele`, `rebellion`, `referendum`, `repondre`, `scenario`, `siecle`, `uberisation`) and four
names (`chloe`, `ecosse`, `equateur`, `etats-unis`) — and the 12 dictionary words entering the cut
brought theirs (`dilapidation`, `doctrinaire`, `drette`, `débaptiser`, `déboîter`, `décalque`, `dég`,
`déjection`, `déplétion`, `désarçonner`, `désossé`, `dînette`). No form loses its gloss: each marked
word has its own (`etat` shows *état*'s « state, condition… », `benoit` *benoît*'s, which opens with
« a male given name… », then the same senses; `ecossais` gains « Scottish »), and 34 forms that had
none gain it (`etait`, `etaient`, `etats`, `derniere`, and 30 newly read: `critere`, `pylone`,
`melancolie`…). `désossé` reads its own gloss « boned, off the bone… » instead of *désosser*'s.
`mwe.tsv` is byte for byte: no headword is gained or lost, and « il etait une fois » meets `il était
une fois` through the new forms. Coverage holds: 4,679 / 8,688 / 15,259 → 4,681 / 8,688 / 15,254.

**The web's unmarked spellings** (read-lingua-french-web-unmarked-spellings) took three glosses out —
`surs` and `sures` « sour » and `entrainant` « post-1990 spelling of entraîner », words of their own
now read as *sûr* and *entraîner* by the owner's exceptions — and the 99 lemmas entering the cut
brought theirs, 86 of them dictionary words (`enceint` « pregnant », `entrejeu` « midfield »,
`estudiantin` « student », `gloser` « to annotate, gloss… », `génialement` « brilliantly… ») and 13
names (`gemayel`, `japhet`, `séleucie`); no other row changes. The spellings, unglossed before, now
show their word's gloss: « tres » *très* « very », « recu » *recevoir* « to receive… ». `mwe.tsv` is
byte for byte; « c'etait deja tres bien » meets `très bien` and « une facon de parler » `façon de
parler` through the new forms. Unglossed spellings leave the tops and glossed lemmas take their
place: coverage 4,681 / 8,688 / 15,254 → 4,693 / 8,716 / 15,348 (93.6 / 86.9 / 76.3 → 93.9 / 87.2 /
76.7 %).

**The web's hyphenated words and wrong accents** (read-lingua-french-hyphenated-and-misaccented-spellings)
took no gloss out; the 64 lemmas entering the cut brought 16, 14 of them dictionary words
(`accusatoire`, `airelle` « cranberry, bilberry », `vertébré` « vertebrate », `écœurer` « to nauseate;
to disgust… », `étriper`, `épaississant`) and 2 names (`achéron`, `vsl`): 30,147 → 30,163 glosses,
26,555 → 26,569 dictionary words. The spellings show their word's gloss: « peut-etre » *peut-être*
« maybe, perhaps », « grand-mere » *grand-mère* « grandmother », « trés » *très* « very ». `mwe.tsv`
is byte for byte; « c'est trés bien » meets `très bien` through the new form. Coverage 4,693 / 8,716 /
15,348 → 4,695 / 8,718 / 15,355 (93.9 / 87.2 / 76.7 → 93.9 / 87.2 / 76.8 %).

No figure is published: the site's figures list the shipped pairs alone, and fr-en ships with change
52.

**French's expressions** (D11), against the shared rules alone: of their 15,526 headwords with a
space, 18 are left out — `à la` and 17 post-1990 spellings whose one sense points at the
traditional one, which stands alone (`crème fraiche`, `s'il vous plait`, `boite à gants`, `cout
d'opportunité`, `plateforme de forage`, `être sur son trente-et-un`…) — and one gloss changes,
`y a-t-il`'s. Of 2,005 words the tokenisation splits that the section glosses, 34 are left out, every
sense of theirs a pointer: `qu'elle`, `qu'on`, `qu'un`… (« que + elle »), `s'est`, `s'en`, `jusqu'au`,
`jusqu'aux`, `qu'au`, `m'en`, `n'en`, `d'avoir`, and post-1990 spellings (`maitre-nageur`,
`casse-croute`, `bloc-note`); « jusqu'au soir » meets `jusqu'à` « until », not « jusque + au ».
`m'a` keeps « “I'm going” » and loses « me + a ». `gloss.tsv` and `senses.tsv` are byte for byte
the same without these rules: they read no lemma's entry.

**23b's rules on fr-en** (`read_as_meanings`), each alone and together, against the section without
them:

| Rule | Rows / top 10,000 | First sense | Expressions | Examples |
|---|---|---|---|---|
| D2, nested senses | 21 / 13 | 12 / 7 | 1 changed, 1 gained | « chambre » « a chamber in its various senses, including » → « a room; a hotel room; a bedroom; a house of a parliament »; « nous » « the plural personal pronoun in the first person » → « we; us, to us; … »; « scène », « champ », « ours », « concevoir », « parrain » |
| D3, shortened and case forms | 8 / 4; 2 lemmas gained (`eine`, `mam'zelle`) | 3 / 0 | 1 gained | « elle » gains « her, it, à elle = hers, its », « elles », « pus », « fac » |
| D4, a function word spelled like a place | 0 | 0 | 0 | — |
| D5, one typography | 243 / 110 | 93 / 30 | 141 | « du » « Forms the partitive article » → « forms the partitive article », « de », « en », « que », « pas » |
| **Together** | **269 / 125** | **107 / 37** | **142 changed, 2 gained** | no gloss lost |

Before them, 8 rows (6 of the top 10,000) held an ellipsis written `...` and 46 (28) straight
quotes; after them, none.

**M20 on fr-en** (the English edition's two settings, the owner's, committed at 0 and off): the
long-parenthesis bound at 40 would change 2,342 rows (902 of the top 10,000), the etymology merging
304 (168); neither loses a gloss. Either setting re-pins es-en and fr-en together.

**Against the design's figures** (the prototype, on change 43's tables before its fixes): 30,059
glossed lemmas → 30,055 (`revenue` and `subordonnée`, whose own form reads as a verb, are no ranks
of these tables; `curial` is) and 86.9 → 86.8 % at 10,000, the design's re-run on 43's first fix
included; 17,480 expressions → 17,479 (`maitre-nageuse`: what its post-1990 pointer writes after
the target, « female equivalent of maitre-nageur », is itself a pointer by the English edition's
rules, so no sense is left); the 23b and M20 figures to the row. The pack, 2,421,321 B in the design
(on changes 45's and 46's prototype tables), is 2,527,222 B: measured on the committed tables and
with change 44's builder, which keys French's expressions through the core's reading and names each
by its headword where its key differs.

**Names glossed by the section are no dictionary words** (refine-lingua-fr-en-glosses D2, the
owner's decision of 2026-10-10). 3,581 glossed lemmas (912 of the top 10,000) are glossed by a
proper noun's senses alone — every run of their `senses.tsv` row is `PROPN`: `france`, `paris`,
`québec`, `françois`, `lyon`, `durand` « a surname »; 3,586 before fr-en's own rules. They keep
their gloss — a reader who opens `Paris` at a sentence's head, or a lowercase `lyon`, still reads it
— and leave French's dictionary words: the vocabulary estimate's universe goes from 30,095 to 26,486
words, and change 41's names rule sets them aside as written: on the baseline's `noms` page `Durand`,
`Lefèvre`, `Jean-Pierre`, `Saint-Étienne`, `Rhône`, `Garonne`, `Paris`, `Renault`, `François`, `Lyon`
and `Grenoble` with `Myriel` (70 → 59 counted words), on `proust` `François` and `Charles`. A word
with a common sense beside a name's stays one (`lot`, `aube`, `nice`, `marche`). Left unglossed, the
names would take fr-en under its floor at every cut (86.4 / 77.8 / 67.5 %). English's and Spanish's
dictionary words keep their names: a change of its own (the owner, Q1). fr-en's pack therefore
carries a lexical table, and `crates/lingua-pack`'s check of a studied folder accepts a reference's
glossed lemmas or all of them but its names alone, nothing in between.

**No level without a gloss** (D3, the owner's decision): of change 48's 40 levelled lemmas fr-en did
not gloss, four are glossed now (`french`, `burger`, `dev`, `ès`) and keep their level; the 36 others
and the 9 levelled lemmas glossed as names alone lose theirs, their slots going to the next lemmas in
rank order (*The levels*). `crates/lingua-pack/tests/committed_tables.rs` checks that every levelled
lemma is a dictionary word with a gloss.

**What still reads wrong**. Change 48's list (its D7) was the input of refine-lingua-fr-en-glosses,
which fixed or left each class (its design's D9), the owner settling Q1–Q5 on 2026-10-10:

| Class | Then (rows / top 10,000) | Now |
|---|---|---|
| A borrowed gloss wrong for the word | « des » « of the; some, the feminine partitive article »; « ca » « board of directors » | `des` reads its own pointers (D4). `ca` reads as `ça` (read-lingua-french-ca-as-ca-cedilla, the change Q4 asked for), with `age` and `forcement`; « board of directors », circa, the plough's beam and the fixing number are gone |
| A pointer's own meaning left out | « il y a », `mieux`, `moins`, `ouais` | read as meanings (D4), on a closed list of a word's wordings |
| An expression whose sense needs a context its key does not hold | `et des`, `que de`, `sur ce`, `et si`, `un coup` | left out by name (D7) |
| A post-1990 spelling keyed apart from its traditional one | `à priori`, `à postériori`, `et cétéra`, `sur son trente-et-un` | lend the traditional spelling's gloss (D7); `crème brulée` already met `crème brûlée` |
| A name glossed and a dictionary word | 3,586 / 914 | no dictionary word, still glossed (D2) |
| A proper noun's run in a common word's row | first 218 / 108; after another run 484 / 260 | the 13 function words' and names-first rows D5 moves, `le` and `on` (D6); 217 rows (104 of the top 10,000: `louis`, `midi`, `belgique`, `noël`, `bordeaux`) still open on a proper noun and 487 / 263 end on one — accepted until a card reads the token's capital (23b's Q3; Q5) |
| The page's own notes | « see usage notes » 5 / 3; « (all senses) » 98 / 30; « in its various senses » 1 / 1; « (Folk etymology: …) » 2 / 2 | out (D8) |
| A description in a capital outside 23b's list | 14 / 8 | fr-en's openers in lower case (D8); the definitions the page writes in sentence case (« Military rank equivalent to corporal ») stay |
| A citation inside a sense | 2 / 2 | cut (D8) |
| A source's numbered sense in another shape | 2 / 1 | out (D8) |
| « etc » without its period | 117 / 42 | « etc. » in fr-en (D8); en-fr's, es-fr's, es-en's and en-es's « etc » is the shared fix 23b named (its D6) |
| Labels left out | « or », « monde »: an obsolete or archaic sense | 23b's Q1 |
| The part of speech a row opens on | 2,897 / 1,264 rows hold two or more | 13 move (D5); a noun, verb or adjective ahead of another stays (Q2) |
| An acronym's pointers lending to a word with an entry of its own | `ca`, `svp`, `jsp`, `cv` | left (D9): dropping them would cost `svp` and `jsp` their gloss; `ca` is no lemma since read-lingua-french-ca-as-ca-cedilla |
| « CA » in capitals (read-lingua-french-ca-as-ca-cedilla D2) | « Le CA a voté le budget. » | read as *ça* « that… » until keep-lingua-french-acronyms-in-capitals (change 43c): « CA » standing alone in capitals — its nearest words not in capitals — is now the acronym `CA` « board of directors; revenue, turnover, net sales; alternating current », a noun to learn; in a headline or a shout in capitals (« LE CA A VOTÉ », « CA SUFFIT ! ») and as « Ca » it reads *ça*. A language code or an address the tokeniser keeps (« (ca) », `.ca`) read « board of directors » before 43b and reads as *ça* now: wrong both ways |
| The acronym's other meanings (keep-lingua-french-acronyms-in-capitals D3) | of 31 « CA » standing alone in 23.5 million tokens: 3 boards, 2 turnovers, 1 chartered accountant, 4 Geneva's Conseil administratif, 6 sports clubs (« CA Brive »), 2 army corps, 2 Canadian dollars, 2 certificate authorities, 2 Latin syllables, 2 geometry segments, 3 codes, 1 activated carbon, 1 unclear | the gloss names the right meaning 5 times and a close one 4 (the owner kept the table knowing it, 2026-10-10); still a noun to learn, never « that ». « CA » beside another word in capitals (« CA HT ») reads *ça* (none in the corpora). The French Wiktionary's *corps d'armée* is no row (open question 2, settled as designed) |
| An unmarked spelling outside the table | `etre`, `etat` (the dictionary's own, 51 ranked); `francais`, `tres`, `deja` (the web's, unglossed) | read-lingua-french-ca-as-ca-cedilla D6: two changes of their own, settled by the owner. The dictionary's are read when the marked word is the commoner (read-lingua-french-dictionary-unmarked-spellings: 81 read, 33 of them ranked before — `etre`, `etat`, `siecle`; 18 ranked kept, the spelling the commoner — `diesel`, `ego`, `egypte`). The web's are read by the owner's reviewed list (read-lingua-french-web-unmarked-spellings: 464 rows down to Zipf 2.5 — `tete`, `francais`, `tres`, `deja` — and the four exceptions `sure`, `surs`, `sures`, `entrainant`); `ou`, `a`, `la`, `des`, `du`, `sur` and `cote` never. Still read wrong: a name or another language's word spelt like a row reads as the French word (« Noel Murray », « Shimon Peres », Occitan « Lou Tresor dóu Felibrige », Catalan « tres »: 53 of the 270 occurrences in the corpora the change's design measured, D5, 19 of them from the list's lower band), the sour adjective's `sure`, `surs`, `sures` read as *sûr* (3 occurrences in 41), and the web's spellings the list leaves out: under Zipf 2.5, or written more often in English (`video`, `hotel`, `mere`, `premiere`). Hyphenated words and wrong accents are two reviewed lists of their own (read-lingua-french-hyphenated-and-misaccented-spellings: 19 runs, `peut-etre`, `c'est-a-dire`, `grand-mere`; 65 words, `trés`, `déja`, `agé`). Still read wrong through them: a name or a neighbour language's word spelt like a misaccented row (4 of the 72 occurrences in the corpora its design measured, D6: « Monsieur Faché », Walloon « béta » and « oû » twice); `elle-meme`, `toi-meme`, `vous-meme`, `nous-memes`, `elles-memes`, whose accented words are no forms of the tables (GSD lemmatises `elle-même` as *lui-même*, and a compound is ranked by its GSD lemma), read as unknown compounds until a change to the ranks; hyphenated spellings under the estimated Zipf 2.5 (`vis-a-vis`, `bien-etre`, `beau-frere`); wrong accents inside a hyphenated word (`lui-mème`); and the 13 misaccented spellings the dictionary itself gives (`problême`, `précisement`, `sureté`, `réglement`), words of their own, which the dictionary rule (marks dropped) does not read and the boundary refuses as rows |
| A correctly spelt form read as a commoner word without its accent | `rebellions` (« nous nous rebellions », *se rebeller*) | reads as *rébellion*, the noun's plural without its accent and the commoner word (M8), as the owner kept it (read-lingua-french-dictionary-unmarked-spellings Q1, 2026-10-10); an override row could keep the verb |
| Two expressions under one key | `ça va` « how are you? … » and `ça ira` « it will be all right », both keyed *ça aller* | the key answers `ça ira`, so « ça va » — and « ca va » now — shows « it will be all right… »; the choice between headwords sharing a key is change 44's, asked of the owner separately |

The rules fr-en adds would read es-en's rows right too (« (all senses) » in 15 es-en rows, its
« etc » in 35): a later refinement of the English edition, which re-pins es-en and fr-en together.

**The French baseline** (`crates/lingua-wasm/tests/french_baseline.rs`) runs over the pack these
tables build from this change on: a pull request that moves `../fr/` or this folder and the French
golden with them re-blesses `fr-en.golden` and says so.

## The French Wiktionary's expressions (`mwe.tsv`)

add-lingua-fr-en-translated-expressions, the owner's decision of 2026-10-10: fr-en's expressions are
completed by the English translations the French Wiktionary's French entries list
(`kaikki-fr-traductions-en.jsonl`, *The sources*), where the English Wiktionary's French section
glosses none. They gloss **no word**: `gloss.tsv`, `senses.tsv`, `../fr/` and the coverage are the
section's alone, `CA`'s gloss too (read from the section's expressions alone).

**The rules** (`reduce-fr-en.py`, `translated_expressions`; no shared module edited):
1. **The table, for expressions** (D1): each French headword's typographic apostrophe read as `'`
   (`straight_translations`: « s’asseoir » is `s'asseoir`), read forwards as es-en reads its direct
   table (`reduce_common.read_translations`, a proper noun's entry left out), a headword with a space
   or a word the tokenisation splits (`split_word`) alone; its gloss up to three translations in the
   table's order, one sense, their case kept, « etc » with its period.
2. **The section's expression wins** (D2): on its headword (the section glosses it: « par exemple »),
   on its key (a headword whose words, each read through `../fr/forms.tsv` and an elided piece
   through `ELISIONS`, are a section expression's: « fondu enchainé » "cross fading" beside « fondu
   enchaîné », « feu de croisement » beside « feux de croisement »), and on what its rules leave out
   (`LEFT_OUT`'s names and the section's expressions whose every sense only points: « à la », « sur
   ce », « boite à gants »). A section entry that leaves no gloss otherwise — a proper noun's — lets
   the table gloss it: « au Moyen Âge » meets « moyen âge » "Middle Ages".
3. **A translation that repeats its headword is no gloss** (D3): « à deux » "à deux", « in situ »,
   « panna cotta » are no expressions; a headword keeps its other translations (« à la diable »
   "devilled").
4. **A reviewed list** (D5): 51 table expressions a person read wrong in most of the contexts they
   are met in are left out by name, in `LEFT_OUT` beside the section's 17, each with its reason — the
   table's gloss, its matches on the corpora and what they are:
   - words met as they are where the table names a set phrase or a field's sense (33): « les
     autres » "other people", « le grand » "the Great", « d'argent » "argent", « d'azur », « plus
     haut », « de haut », « travailler à », « un temps », « une étoile », « en premier », « se mêler »,
     « il se trouve », « la peau », « sortir avec », « au vent », « bien du », « première ligne »,
     « ombre de », « quatre heures », « grand homme », « la mettre », « première partie », « bon
     vieux », « c'est une chose », « du gâteau », « la barbe », « au temps », « petite bête », « à la
     ligne », « à ses yeux », « sur le champ », « avoir l'œil », « graine de »;
   - an address or an interjection met as plain words (4): « à vous » "over", « mon cœur », « et
     bien », « tu vois »;
   - a name met as itself (4): « côte d'azur » "blue band", « louis xiv », « henri iv », « de gaulle »;
   - a number met as itself (1): « à neuf » "anew" (« à neuf heures »);
   - a verb's form met as a noun's headword (5; M8): « être cher », « groupe a », « en laisse », « en
     joue », « en conserve » (« en conservant »);
   - a wrong translation (4): « se mettre d'accord » "pull together", « voie navigable »
     "thoroughfare", « se heurter » "blunder", « se serrer » "clench".

**What the rules count** on the table of 2026-10-02 (headwords with a space / split words, each
counted at the first rule it stops at, as the reduction's summary line prints them):

| Rule | With a space | Split words |
|---|---|---|
| The section glosses the headword | 7,434 | 1,151 |
| Left out by name (`LEFT_OUT`: 4 of the section's names, the 51 of the review) | 53 | 2 |
| The section's senses only point | 2 | 3 |
| A key the section reaches | 133 | 0 |
| Only repeats the headword | 783 | 265 |
| **Kept** | **23,141** | **2,766** |

210 kept headwords lost a translation that repeated them. 55 glosses run past 80 characters (the
longest, « même une horloge arrêtée donne l'heure juste deux fois par jour », 142), kept whole as
es-en's direct table's are (the owner, 2026-10-10). No row is empty or ends on an opening mark.

**Measured on the committed tables** with the design's harness (`build_pack`, the core's
`reading_expression_key` and `gloss_phrase` over 1,060,904 words of French text — UD French-GSD's
training and development sentences, Gutenberg, French Wikipedia, a Tatoeba sample and the
baseline's pages): the builder keys 17,050 of the kept headwords with a space and 117 split words
(the rest hold a word the lexicon does not, run past seven tokens, or are one hyphenated token); **no
key is reached by both a section and a table headword**, and 142 keys by two table rows, settled by
the builder's order. On the corpora, 60,340 → 68,361 expression matches, **56.9 → 64.4 per 1,000
words**, 52.2 → 55.7 % of lines meeting one (GSD 54.3 → 61.6, Gutenberg 58.9 → 67.9, Wikipedia
51.6 → 58.8, Tatoeba 73.1 → 80.4, the baseline's pages 56.9 → 59.9); 8,890 matches are the table's
(1,606 expressions; 7,653 with a space, 1,237 split words, most of them a reflexive verb before a
vowel: « s'étend », « s'installer »), and 875 of the section's (1.5 %) are displaced by a longer or an
earlier table expression — of 50 sampled in the design, 37 read better, 11 the same, 2 worse. The
design's samples, read by one judge: 85 right, 6 partly, 9 wrong in 100 matches drawn after the list;
the section's own expressions, sampled the same way, 73 right, 1 partly, 26 wrong.

**What is left**: the 1,285 expressions met fewer than five times are not read one by one (about one
match in ten of the result still reads wrong, « premières sorties de l'eau » "roll-out"); the
owner reads a sample of 100 drawn from these tables (task 6.2). The section's own expressions read
wrong as often (« de l'équipe » met as the partitive "some", « il m'a sauvé » as "“I'm going”", « en
faisant » as « en fait » "actually", « sur le champ de bataille »'s « champ » in fr-es's « sur le
champ » "En el acto"): a refinement of their own (the design's Open Questions 3 and 4), fr-en's
done by refine-lingua-fr-en-expressions (*The section's expressions read where they are met*).

**The French baseline**: `fr-en.golden` moves in its pack, notice and licences and in four phrase
probes that now meet a table expression — « Il est prêt à le faire » « prêt à » "ready, about to",
« un moteur diesel » « moteur diesel » "diesel engine, Diesel engine, Diesel motor", « Il est au
courant de tout. » « être au courant » "aware, know, in the know", longer than the section's « au
courant », and « Elle se leva. » « se lever » "get up, stand up"; three phrase probes are added to
both French goldens — « Il existe plusieurs solutions. » meets « il existe » "there exists", « Les
autres enfants jouent dehors. » none, « au Moyen Âge » « moyen âge » "Middle Ages". « une crème
brûlée » and « sur le champ de bataille », probes of refine-lingua-fr-es-expressions, still meet the
section's « crème brûlée » and « champ de bataille », not « sur le champ ».

## The section's expressions read where they are met (`mwe.tsv`)

refine-lingua-fr-en-expressions, the owner's decision of 2026-10-10: the English Wiktionary's French
section's own expressions are offered only where their gloss says what the words mean where a reader
meets them. Three rules of `reduce-fr-en.py`, over the section's expressions alone (`is_expression`):
`gloss.tsv`, `senses.tsv`, `../fr/`, French's dictionary words, levels and coverage do not move, and
no shared module is edited, so no other pair re-pins.

1. **A reviewed list** (D1). Every section expression met ten times or more on the corpora of *The
   French Wiktionary's expressions* was read in three or four of its sentences (734 on the tables
   before add-lingua-fr-en-translated-expressions; on its tables, the 13 that came to ten and the 3 that
   came to ten once the list was applied were read in all their sentences). **115 read wrong in most
   of their contexts are left out by name**, in `LEFT_OUT` with their reasons — the section's gloss,
   their matches on those tables, how they are met and one example:
   - a labelled sense met as plain words (47; 2,945 matches): every sense carries a register or a
     region the plain words do not — « pas de » "without", Quebec (607: « il n'a pas d'enfants »),
     « dire que » "to think that", « histoire de » "in order to", « j'aime » "a like", « à toute » "see
     you", « fait que », « sa mère » "(so) fucking", « de deux », « pas que », « t'a », « sais pas »,
     « espèce de », « m'as », « vu que », « de couleur », « encore que », « la faire », « du reste »,
     « au plus », « comme tout », « et quelques », « de première », « du vent », « à plus », « qui
     parle », « à la place », « la ferme », « à d'autres », « tu parles », « d'aucuns », « des
     familles », « et encore », « faire avec », « en fait de », « y passer », « à la limite »,
     « rapport à », « bande de », « faire bien », « s'en être », « se prendre », « c'est cela », « du
     diable », « quoi faire », « de un », « faire sans », « y rester »;
   - a headword met through another word's form (22; 1,830): its key reads a word through its
     dictionary form, and the page writes another — « d'autre » "else" on « d'autres » (461), « en
     être » on « en est », « de nouveau » "again" on « de nouveaux », « de fait » on « de faire »,
     « en fait » "actually" on « en faisant », « étant donné », « en tout », « n'eût été », « tenir
     de », « il ne fallait pas », « ceci étant », « la tienne », « cela étant », « la majeur »,
     « attendu que », « en tenue », « en devenir », « se passer de », « avoir lieu de », « à
     découvert », « de plus belle » on « de plus beau », « à l'anglaise »;
   - words met as they are, where the headword names a set phrase, a field's sense or a cry (44;
     1,645): « être à » "to belong to" (293), « de plus » "moreover", « nombre de » "quite a few",
     « première fois » "first time (the first instance of sexual intercourse)", « à moi » "help!",
     « d'y », « le temps de », « au pied » "to heel!", « de droit », « bon dieu », « de rien » "you're
     welcome" (« ne … de rien »), « au travail », « c'est moi », « un tel », « de service », « de
     jour », « du siècle », « de confiance », « qui sait », « tout fait », « de grâce », « avoir mal »,
     « d'esprit », « de cœur », « le chercher », « au reste », « avoir raison de », « au feu », « bien
     fait », « de force », « de tête », « pauvre de », « à suivre », « à la base » "originally", « en
     série » (« en Serie A »), « de poids » (« perdre un peu de poids »), « d'un coup », « de concert »,
     « du temps de », « à l'horizon », « à la pointe », « au courant », « le leur », « au point »;
   - a colloquial spelling met as the standard one (2; 505): « m'a » "I'm going" (455: « il m'a
     sauvé »), « t'es » "you're".

   Read and kept: « ça va » reads « it's going to » in 11 of its 18 matches, but its key is « ça
   ira »'s, which would then meet every « ça va » (« Ça va ? » "it will be all right"); « la leur »
   reads « theirs » in 5 of 10. The French Wiktionary's table does not bring a name back: 50 of its
   headwords the section glossed count as left out by name instead (7,386 / 1,149 headwords the
   section glosses, 101 / 4 left out by name), and the kept rows are the same 23,141 / 2,766.
2. **A sense that only notes a form or a usage is no meaning** (D2, `_FORM_NOTE`). A sense, or a
   piece of it between semicolons, opening on « compound of X and Y », « (the|an|a) (interrogative)
   form of X … in the first|second|third-person », « an interrogative form of », « for non-idiomatic
   uses », « used only in the » or « equivalent to “ » goes, and an expression left with none is
   none and counts as only pointing, so the table leaves it out too: « j'ai » "I've, I have", « ne
   faire que » "only, merely, just; …", and « a-t-il », « puis-je », « colin-tampon » out (148
   matches); « à moi », « m'as » and « s'en être » are on the list as well. « a form of » alone opens
   meanings and stays (« pain d'épice » "a form of French gingerbread").
3. **« de l' » reads "of the" first** (D3, `LENT_SENSES`). The section glosses it by the partitive
   alone, where it is « de la » or « du » before a vowel and both are glossed "of the" first (29 of 30
   sampled matches): it opens on « de la »'s first sense, "of the; some; the singular prevocalic
   partitive ar". « de l'eau » keeps its partitive.

**Measured** with the design's harness on 1,060,904 words (the tables of
add-lingua-fr-en-translated-expressions → these): `mwe.tsv` 43,542 → 43,424 rows (the section's
17,635 → 17,517, three glosses changed); the pack 3,004,986 → 3,002,217 B; **68,361 → 61,780
expression matches**, 64.4 → 58.2 per 1,000 words, 55.7 → 52.5 % of lines meeting one (GSD 19,903 →
18,339, Gutenberg 14,283 → 12,375, Wikipedia 22,286 → 20,783, Tatoeba 11,829 → 10,232, the
baseline's pages 60 → 51). 7,100 matches go — 6,925 of the list, 148 of the expressions left with no
sense, 27 displaced — and 519 come, a shorter or later expression a listed one hid (465 of the
section, 54 of the table: « en faisant du bruit » meets « faire du bruit » "to make a lot of noise").
The design's samples (one judge): of 300 matches before, 221 right, 13 partly, 66 wrong; of 100
drawn from its prototype's result, 95 right, 2 partly, 3 wrong.

**What is left**: the expressions met fewer than ten times are not read one by one (about one wrong
in ten, sampled); the matcher's runs across a number or a punctuation mark the tokeniser drops (« en
2007 est » meets « en être », 0.9 % of the matches) and a split word keyed by its verb's dictionary
form met in another tense (« j'ai » on « j'avais ») are the engine's, which moves fr-es too.

**The French baseline**: `fr-en.golden` moves in its pack and in three phrase probes — « de l’eau »
and « de l'huile d'olive » meet « de l' » "of the; some; …", and « j'ai recu ta reponse » « j'ai »
"I've, I have"; no other probe answers differently, fr-es's golden not at all.

## What the later changes add

| Change | Adds |
|---|---|
| 40 tokenisation | French's pre-pass, which these tables serve; its version bump re-reduces fr-en (`manifest.json` alone moves, and the pack and pin with it). The check over these tables (`crates/lingua-pack/tests/committed_tables.rs`) then reads its lists from lingua-core |
| 41 analysis | the cascade, designed and measured on these tables; its version bump re-reduces fr-en |
| 44 expression keys | the plain words left out here because they begin with a piece (`d'abord`, `c'est`, `l'on`) |
| 48 fr-en | done: the English glosses, expressions and senses (*The glosses*); `../fr/lexical.tsv` holds the glossed lemmas, and the French invariance baseline runs over these tables |
| 48b refine-lingua-fr-en-glosses | done: fr-en's own rules (*The glosses*); French's dictionary words leave out the names fr-en glosses alone, and only they take a level |
| 43b read-lingua-french-ca-as-ca-cedilla | done: French's unmarked spellings (`ca` → *ça*, `age`/`ages` → *âge*, `forcement` → *forcément*; *The rules*); « CA » in capitals waits for the acronym change (43c, `keep-lingua-french-acronyms-in-capitals`), and so does this change's release |
| 43c keep-lingua-french-acronyms-in-capitals | done: French's acronym table, `CA` alone — a form in capitals that is its own lemma, a dictionary word with no rank or level, glossed from fr-en's expressions (*The rules*, *The glosses*); French's analyser `1.3.0` |
| 43d read-lingua-french-dictionary-unmarked-spellings | done: the dictionary's own unmarked spellings read as their word when it is the commoner (`etre` → *être*, `etat` → *état*, `repond` → *répondre*; *The rules*); it ships with or after 43b |
| 48c add-lingua-fr-en-translated-expressions | done: the French Wiktionary's English translations complete the expressions (*The French Wiktionary's expressions*); the pin records two editions under two releases |
| 48d refine-lingua-fr-en-expressions | done: the section's own expressions read where they are met — a reviewed list, the form notes, « de l' » "of the" (*The section's expressions read where they are met*) |
| 49 fr-es | a reader of `../fr/` as committed, capped at the same 60,000 |

## Licences

The repository is Apache-2.0; **these files are not**, nor French's in `../fr/`. They are derived
from the sources above and carry their licences:
- `../fr/forms.tsv`: CC BY-SA 4.0 and the GFDL (kaikki), and CC BY-SA 4.0 (GSD's counts);
- `../fr/freq.tsv`: CC BY-SA 4.0 (wordfreq, and GSD's counts for the compounds);
- `../fr/grammar.tsv`: CC BY-SA 4.0 and the GFDL (kaikki);
- `../fr/level.tsv`: CC BY-SA 4.0 (derived from `freq.tsv`) and CC BY-SA 4.0 and the GFDL (kaikki,
  which says which lemmas take a level);
- `gloss.tsv`, `senses.tsv`, `mwe.tsv` and `../fr/lexical.tsv` (derived from `gloss.tsv` and
  `senses.tsv`): CC BY-SA 4.0 and the GFDL (kaikki: the English Wiktionary, and the French
  Wiktionary for `mwe.tsv`'s expressions the section does not gloss), and CC BY-SA 4.0 (GSD's counts,
  which order a function word's senses).

`NOTICE` gives the full attribution. See `../../SOURCES.md`.

## Changing them

Never by hand — except `../fr/tags.tsv` and `../fr/studied.json`, which no reducer writes.

- **Take in upstream changes**: dispatch `lingua-pack-update` with `pair=fr-en` and `mode=update`.
  It reads the English and the French editions' dumps once each (about 3 GB and 0.74 GB gzipped),
  derives the French section and the French entries' English translations from them, keeps their
  bytes as the release `lingua-pack-sources-fr-en-<snapshot>` (about 26 MB and 1.9 MB compressed),
  records GSD's two sections at their commit, reduces, and pushes the branch
  `lingua-pack/fr-en/<snapshot>`. With `keep=en` (or `fr`) it keeps that edition's recorded bytes
  and reads the other alone — how the French edition was added on 2026-10-10 without a new French
  dictionary; never a way to avoid an update (`../../SOURCES.md`, *An update can keep an edition*). Its runs are one at a time in the `fr` group, so an update of fr-en
  and of fr-es never propose `../fr/` on two branches.
- **After editing the reduction rules** — `reduce-fr-en.py`, the override, elision and `LEFT_OUT`
  tables included, `reduce_common.py`, which every pair shares, or `reduce_edition_en.py`, the
  English Wiktionary's rules, which es-en reads too (`pin.json` lists the three under
  `reducer.files`): the check lane fails until the tables are reduced again from the pinned sources.
  Run `scripts/lingua-data/build.sh --reduce fr-en <out>` (Python 3.12, `requirements-reduce.txt`),
  or `lingua-pack-update` with `mode=reduce`. A bump of `FRENCH_ANALYZER_VERSION` asks the same.
- **After a builder or dependency change** that changes the pack's bytes: update `pack.sha256` and
  `pack.size` in `pin.json` in the same pull request.
- **When a reduction's readings carry a tag `../fr/tags.tsv` does not pin, or no longer carry a
  pinned one**, `crates/lingua-pack/tests/committed_tables.rs` fails naming it: append the new tag
  after the pinned ones (a pinned tag never changes index), or remove the one left over, in the same
  pull request.

The monthly dry run of the update checks this pair as it checks the others.
