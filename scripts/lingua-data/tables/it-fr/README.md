# The it→fr dictionary tables

The reduced tables of Cymbra Lingua's Italian→French data pack, committed so that every build makes
the same pack with no download, and so that a change to the dictionary is a pull request whose diff
shows it (add-lingua-italian-forms-tables, change 9 of `docs/lingua/italian-programme.md`). No
extension package carries the pack yet: `packs.json` does not list it-fr.

They are in two folders (split-lingua-pack-tables-by-language). it-fr is Italian's reference pair
(`../it/studied.json`, the programme's I5): its reduction writes Italian's own tables in `../it/`,
which every pair studying Italian reads as committed (it-en, it-es), and this folder's `pin.json`
records the sources of both. Change 9 writes Italian's forms and ranks, change 13 its readings
(`../it/grammar.tsv`, add-lingua-italian-grammar-tables, *The readings* below) and their pinned tag
pool (`../it/tags.tsv`), change 14 its estimated levels (`../it/level.tsv`, add-lingua-italian-levels,
*The levels* below), and change 16 (add-lingua-pack-it-fr) its French glosses, sense runs and
expressions in this folder and Italian's dictionary words in `../it/lexical.tsv`, which the levels go
to (*The glosses* below); change 17 (refine-lingua-it-fr-glosses) refines the glosses and some of
Italian's forms (*Refined: change 17* below). The Italian baseline (`crates/lingua-wasm/tests/italian_baseline.rs`) runs
over the pack these tables build since change 16 re-blessed it once — its hand-written fixture
(`scripts/lingua-data/testdata/it-fr/`) before, a pack of forms and ranks alone blinding its gloss,
expression and level probes —; the readings are pinned by `crates/lingua-pack/tests/it_fr_grammar.rs`
over the committed tables too.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | lemma → its French gloss, up to eight senses grouped by part of speech | kaikki.org: the French Wiktionary's Italian section, then the Italian Wiktionary's French translations, then the French Wiktionary's translation tables read backwards (CC BY-SA 4.0 + GFDL; change 16) |
| `senses.tsv` | lemma → its gloss's sense runs, a part of speech and how many senses | the same |
| `mwe.tsv` | expression → its French gloss, keyed at build as Italian is read (change 12) | the same: the section's headwords with a space, the tables', the words Italian's pre-pass splits |
| `NOTICE` | the attribution stack, embedded in the pack, and the sentence saying the levels are estimated | — |
| `manifest.json` | Italian glossed in French, Italian's analyser version (`ITALIAN_ANALYZER_VERSION`, read from lingua-core), `pack_version` (the snapshot), `levels_estimated` (change 14) | — |
| `pin.json` | the raw sources these tables and `../it/` came from, and the pack, base and layer they build | — |

In `../it/`, Italian's tables, written by it-fr's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org, the English Wiktionary's Italian section (CC BY-SA 4.0 + GFDL), with UD Italian-MarkIT's counts to choose between lemmas (CC BY 4.0) and Morph-it!'s forms the Wiktionary names nowhere (CC BY-SA 2.0) |
| `freq.tsv` | lemma → frequency rank, the 60,000 commonest | wordfreq 3.1.1 (CC BY-SA 4.0); a hyphenated word by MarkIT's own frequency too |
| `grammar.tsv` | form → its readings, as Universal Dependencies tags (`form<TAB>lemma<TAB>tag<TAB>other\|-`) | kaikki.org, the same section (add-lingua-italian-grammar-tables, change 13) |
| `level.tsv` | lemma → its estimated CEFR level, 8,302 lemmas of English's level sizes, Italian's dictionary words alone since change 16 | none: derived from `freq.tsv`, the same section and `lexical.tsv` (add-lingua-italian-levels, change 14) |
| `lexical.tsv` | Italian's dictionary words: it-fr's glossed lemmas less those it glosses by a proper noun's senses alone, plus the acronym rows `NATO` and `USA` | it-fr's glosses and runs (change 16); `reduce_italian_acronyms.ACRONYMS` (keep-lingua-italian-acronyms-and-names D6) |
| `tags.tsv` | Italian's pinned tag pool: the 83 tags its readings carry, in byte order; written by no reducer | committed by hand from the reduction's readings (change 13), replacing change 9's placeholder `NOUN` |
| `studied.json` | the pair whose reduction writes `../it/`: it-fr | committed by hand |

## The sources

- **kaikki.org, the English Wiktionary's Italian section** (`kaikki-Italian.jsonl`, 624,122 entries,
  665,207,473 B, sha256 `ae7de484ba53e3a6a472b9672a69d7e7fdc96cf55b68c5399a5b338adb387f0b`), derived
  from the English edition's dump (`pack_sources.py EDITIONS`, `DUMPS["it-fr"]`): the dump
  regenerated on 2026-10-21 11:23 (Last-Modified), read on 2026-10-23, decompressed sha256
  `60a6073c…` (26,611,939,151 B; 3,082,192,147 B gzipped), not kept; the section kept as the zstd
  asset `kaikki-Italian.jsonl.zst` (33,120,545 B) of it-fr's own release
  `lingua-pack-sources-it-fr-2026.10.23`, published by `lingua-pack-update` run 39974561742. Its
  verb tables list every form with its stress marked (`dàre`, `darèbbe`), its reflexive verbs' rows
  with their pronoun (`mi làvo`), and a form's own entry points at its lemma.
- **UD Italian-MarkIT**, its training and development sections at
  `e210079df0aea331ec1dcaa90648d815ce94c3ce` (`pack_sources.py PINNED["it-fr"]`, 1,266,080 and
  665,430 B), read for how often a form stands for each lemma and how often a hyphenated lemma
  occurs. Its test section is never read: the measurement holds it out.
- **Morph-it!** 0.4.8 (Marco Baroni and Eros Zanchetta), the archive its site serves kept unmodified
  as the one-off release `lingua-pack-sources-morph-it-0.4.8` (2,617,548 B, sha256
  `b137343dc095e038ebfaef7c743185e85ec35f5322a1b8629b600e6e898c7024`), read for its lexicon
  `current_version/morph-it_048.txt` under its CC BY-SA 2.0 option, the sha256 checked at every
  fetch. Kept by the owner on 2026-10-23 (task 5.1).
- **kaikki.org, the French Wiktionary's Italian section** (`kaikki-fr-Italien.jsonl`, 769,483,152 B,
  sha256 `eb79761b83400a816a0e70552204e6b3556ef561489cf003f0252862bc6542b5`) **and its French entries'
  Italian translations** (`kaikki-fr-traductions-it.jsonl`, 7,469,549 B, sha256 `a39a2ad5…`), derived
  from the French edition's dump regenerated on 2026-10-23 23:53 (Last-Modified), read on 2026-10-24,
  decompressed sha256 `7ce93a41…` (6,865,138,291 B; 736,596,796 B gzipped), not kept; kept as the zstd
  assets `kaikki-fr-Italien.jsonl.zst` (29,272,128 B) and `kaikki-fr-traductions-it.jsonl.zst`
  (1,018,694 B) of the release `lingua-pack-sources-it-fr-2026.10.24`, published by
  `lingua-pack-update` run 40130354122 (mode `update`, `keep: en`: the English section above stays the
  2026.10.23 release's). The glosses and expressions, and their inverted fallback (change 16).
- **kaikki.org, the Italian Wiktionary's French translations** (`kaikki-it-traductions-fr.jsonl`,
  1,273,076 B, sha256 `424c527e…`), derived from the Italian edition's dump regenerated on 2026-10-21
  00:21, decompressed sha256 `09f22a16…` (566,646,259 B; 43,849,715 B gzipped), kept as
  `kaikki-it-traductions-fr.jsonl.zst` (156,241 B) in the same release: the glosses' direct fallback.
- **wordfreq `it` 3.1.1**, pinned by version and by `../../requirements-reduce.txt`'s hashes; wordfreq
  `fr`, the same package, orders the French tables read backwards.

## The rules (`reduce-it-fr.py`)

- **What a form is (D3).** An Italian word, lowercased and in NFC, the typographic apostrophe read
  as `'`. A form comes from the inflections a lemma's entry lists and from the form-of links of its
  own entry — an apocope's own link among them (`un` → *uno*, `buon` → *buono*) —, never from
  kaikki's bookkeeping or a verb's auxiliary row, an inflection tagged doubtful (alternative,
  obsolete, archaic, rare, dated, uncommon, misspelt, nonstandard, proscribed, abbreviated), a
  gender or number marker, nor a row naming another word: a diminutive, augmentative, pejorative,
  derogatory or endearing form (`casa` → `casétta`), an adjective's adverb (`vero` → `veramente`), a
  noun's relational adjective (`mare` → `marittimo`), a name's demonym, a plural noun's `plural-of`
  row — nor any row of a name's entry. A derived word is read from its own entry, its « diminutive
  of casa » a meaning: `casetta` is a lemma of its own. A form of a form reaches its word along one
  part of speech (`creduta` → `creduto` → *credere*).
- **The tables' stress marks.** Every mark but on the last letter is dropped (`dàre` → `dare`); the
  last letter's is the written accent and stays (`darà`, `dà`) unless the entry carrying the cell's
  form-of link is the bare spelling (`hà` → `ha`, `dò` → `do`). A loan's marks stay (`würstel`).
- **Reflexive verbs.** A row written with its pronoun gives its bare form: under the plain verb
  where the section has one (`lavo` → *lavare*; `lavarsi` is no lemma), under the reflexive
  otherwise (`pento` → *pentirsi*), whose rows carrying the pronoun give their bare base (`pentire`,
  `pentendo`, `pentite`), the bases change 7's enclitic rule needs. PUD lemmatises a reflexive
  without its pronoun (`riunirsi` → *riunire*), so a reflexive-only lemma counts against the content
  gate.
- **Clitics (D4).** No verb form carrying clitics is a form or a rank (`dirmelo`, `vattene`,
  `eccolo`, `farlo`, `pentirmi`, `recatevi`, `stammi`): change 7's enclitic rule reads them. A verb
  whose infinitive carries another clitic (`andarsene`, `esserci`, `farcela`) is no lemma, its rows
  no form. A noun spelled like a compound stands (`fallo`), and a compound that is another word's
  inflection stays its form (`fatti`).
- **Italian's tokenisation (D4, I7).** Every word change 6's pre-pass writes is a form — the
  reduction fails otherwise —; each elided piece and truncation is a form of its word's lemma (`l'`
  → *lo*, `un'` → *uno*, `quest'` → *questo*, `po'` → *poco*, `va'` → *andare*); the `di` family
  (`del`, `dello`, `della`, `dei`, `degli`, `delle`) are lemmas of their own, `dell'` → *dello*; the
  other articulated prepositions are neither forms nor ranks, `dai` excepted (a form of *dare*, the
  verb the pre-pass reads `dai` and `da'` as by their context, no rank); no plain word beginning
  with a piece is a form (`c'è`, `dell'acqua`). Since refine-lingua-italian-reading (an
  Italian-specialist review, its D2, D3 and D7), the pre-pass reads eighteen more elided pieces
  (`trent'` → *trenta*, `qual'` → *quale*, `tant'`, `gliel'`, `prim'` → *primo*, `ultim'` →
  *ultimo* …), `coll'` (`con` + `lo`, no form) and `degl'` (→ *degli*), and the books' truncations
  `ca'` → *casa*, `de'` → `dei`, `que'` → *quello*; its truncated articulated prepositions `a'`,
  `co'`, `da'`, `ne'` and `su'` are neither forms nor ranks, as `all'` is — the section's own `da'`
  entry is read no more —; and `mò` (of `mo'`, by the word after it) is a word it writes. So
  `trent'anni`, `qual'è` or `gliel'ho` are no forms, the pre-pass reading their pieces.
- **One lemma per form (D5, M8).** A form whose own lemma entries are names alone reads as the other
  word (`bambini`, a surname → *bambino*); `roma`, which no other word reads, stays the city; every
  other form is left to `OVERRIDES` — the owner's `stato` and `stati` → *essere* (2026-10-23; « lo
  Stato » reads as *essere*, the accepted cost), and `bianca` → *bianco*; since change 17, `usa` →
  *usare* and the owner's `dati` → *dati*, `lega` → *lega*, `prova` → *prova* (2026-10-25), no `dato`
  row —, then MarkIT's counts, the form's own entry (since change 17, by a meaning alone where a
  verb's table gives the form as a person: *Refined: change 17*), the lemma's frequency, the alphabet.
- **The apocopes** written without an apostrophe read as their word alone by a reviewed table
  (`APOCOPES`): `un` → *uno*, `buon` → *buono*, `nessun`, `alcun`, `ciascun`, `qualcun`, `bel` →
  *bello*, `san` → *santo*, `gran` → *grande*, `quel` → *quello*, `tal` → *tale*, `qual` → *quale*,
  and `fin` → *fino* (« fin qui », refine-lingua-italian-reading D7); since change 17, `signor` →
  *signore* (« Variante de signore » in the French section) and the euphonic `od` → *o*.
- **Acronyms in capitals (keep-lingua-italian-acronyms-and-names).** A reviewed table,
  `reduce_italian_acronyms.ACRONYMS`, the owner's decision row by row (2026-10-23: `NATO` and `USA`
  only; `FAI` and `AGI` too rare in the corpora, `TARI` already read as the tax): each written
  into `forms.tsv` as a form in capitals that is its own lemma — the only forms holding a capital —
  and into `lexical.tsv`, with no rank and no level; `nato` stays *nascere*'s and `usa` reads as
  *usare* (*uso*'s until change 17's override row).
  Italian's analysis reads a word standing alone in capitals as written, so « della NATO » is the
  alliance and « LA NATO HA VOTATO » the participle. The reduction fails on a row the section's own
  entry does not back, or whose lower case the tables read as no other word (`check_lowercase`): `ONU`
  reads as its own entry; `RAI` passes the measurement's rule (11 written alone in capitals, every
  one the broadcaster) but is held back by that check, `rai` being a lemma of its own spelling (rank
  1,214), and the owner kept `NATO` and `USA` alone. The rows' gloss is change 16's (`acronym_gloss`:
  the expansions, else the synonyms — `USA` through `America` and `Stati Uniti` —, else none: `NATO`).
- **Morph-it! (D3).** A form it lists under a ranked lemma, which the Wiktionary names nowhere,
  joins the table — never one carrying clitics, never moving a reading or a rank.
- **The ranks (D6).** wordfreq's order, dense, the 60,000 first, over its words that are not only
  inflected forms; a word whose own form reads as another gives its rank to the next. wordfreq's
  stems of the elided pieces (`l`, `d`, `gl`, `quest`, `dell`, `trent`, `tant`, `coll`, …) and of
  the truncations (`po`, `mo`, `be`) are no words, nor its tokens beginning with a piece, but the
  apocopes (read as their words) and `cent`, the euro cent — and the truncations' stems that are
  lemmas of their own with a rank (`de`, `ca`, `que`, and the prepositions' `a`, `co`, `ne`, `su`,
  `da`), which wordfreq counts with the truncations' own uses; a hyphenated word is ranked when
  MarkIT attests it, at the lower of wordfreq's estimate and MarkIT's frequency.

## The glosses (`gloss.tsv`, `senses.tsv`, `mwe.tsv`): change 16

Written by the same reduction as `../it/`, through the native side every pair shares
(`reduce_common`) with the French Wiktionary's rules (`reduce_edition_fr.FR`), in es-fr's order
(add-lingua-pack-it-fr D1, D2): **the French Wiktionary's Italian section**, its definitions first;
then **the Italian Wiktionary's French translations**, in the table's order; then **the French
Wiktionary's translation tables read backwards**, commonest French word first — at most three words
per part of speech from a table, every gloss French written by a person. The section is read as it-fr
reads it first (`section_as_read`, `../../SOURCES.md`, *IT → FR: Italian glossed in French*): letters
out; an acronym's sense, and a proper noun's that only says surname, first name, hamlet or commune,
kept off a word with a lowercase entry (D5: `TE` off `te`, `Ora` off `ora`; `luna` keeps « Lune,
satellite de la Terre »); a gloss filed as a link read as a gloss when no entry holding a meaning heads
its target (D4: `del` « Du. », `coloro` « Celles et ceux », `c'è` « Il y a », `te` « Toi : pronom
disjoint… »); and kaikki's leaks read out — examples and addresses dropped, colour codes taken out, a
head line merged into a sense read for its meaning, a letter's bare name dropped beside a common word,
a repeated sense shown once (the review of the implementation).

The update of **2026.10.24** (`lingua-pack-update` run 40130354122, mode `update`, `keep: en`, the
release `lingua-pack-sources-it-fr-2026.10.24`) wrote these tables, byte for byte what the
implementation predicted from local copies of the sources.

**Coverage**, the share of the commonest lemmas of `../it/freq.tsv` with a gloss
(`gloss_coverage.py --pair it-fr`), against the floor the owner settled before this measurement
(2026-10-24, D7: the study's figures less two points; never lowered after it):

| | 5,000 | 10,000 | 20,000 |
|---|---|---|---|
| it-fr, committed | **89.2 %** | **81.8 %** | **73.3 %** |
| Floor (`FLOORS["it-fr"]`) | 87.2 % | 79.8 % | 71.3 % |
| From the definitions alone | 87.6 % | 79.5 % | 70.5 % |

**The glossed lemmas by source**: 32,691 rows — 31,006 from the section's definitions (94.8 %), 279
from the Italian Wiktionary's translations, 1,405 from the French tables read backwards, and the
acronym row `USA` (« Amérique (continent); États-Unis », through its synonyms; `NATO` has none). Of the
8,183 glossed among the 10,000 commonest, 2.8 % come from a table (46 direct, 187 read backwards).
**The section as read**: 3,884 name notes, 245 communes' notes and 83 acronym senses set aside; 33
links to a capitalised word and 36 to a lowercase one read as glosses, 11 left forms beside another
gloss; 35 letter senses, 7 letter entries and 5 letters' bare names out; 9 examples filed as senses
and 16 head lines with no meaning dropped, 9 head lines read for their meaning, 226 colour codes taken
out; 87 glosses repeating a sense shown once.

**Expressions** (`mwe.tsv`, D6): 18,296 — 10,704 of the section's headwords with a space, 7,517 of the
tables', 22 + 53 words the pre-pass splits (`all'improvviso`, `c'è`, `d'accordo`, `senz'altro`). Keyed
at build by change 12's rules: 14,776 keyed, 3,520 left out (a word the pack does not hold, or past
seven tokens; 61 longer than seven); 4,677 named headwords, 140 keys several headwords reach; 157 keys
end on `di`, and over change 11a's corpus (1,011,235 blocks) 3,000 of 296,509 matches are the `di`
family read once more as `di`, on 91 expressions.

**Dictionary words** (`../it/lexical.tsv`): 24,216 — the glossed lemmas less the 8,476 glossed by a
proper noun's senses alone (Italian's names count; English's is 3,046), plus `NATO` and `USA`. Of the
1,425 commonest lemmas, 102 are no dictionary word: 49 unglossed (`facebook`, `ok`, `web`, `entro`,
`supporto`) and 53 names (`italia`, `roma`, `milano`).

**The pack** these tables build: 2,645,342 B, sha256 `0e6795d938bc0ca7307ff461e3175d1768f8edcf3b6ccae7efc4a2b3688793c5`
(50.5 % of the 5 MiB budget); its base 1,551,534 B (`f8c78228…`), Italian's, its layer 1,094,048 B
(`0927f745…`).

**A sample of 100 glosses** of the 20,000 commonest, drawn by the reduction (`SAMPLE_SEED`, D8) and
marked by source — `section` the French Wiktionary's Italian section, `it-wikt` the Italian
Wiktionary's French translations, `fr tables` the French tables read backwards; a gloss past 100
characters cut here:

| Rank | Lemma | Source | Gloss |
|---:|---|---|---|
| 47 | `degli` | section | Article contracté souvent traduit par des. Note d’usage : il s’emploie alternativement à dei devan… |
| 212 | `padre` | section | Père |
| 312 | `sociale` | section | Social, relatif à une société, une communauté, une collectivité; Social, qui se préoccupe du fonct… |
| 561 | `discorso` | section | Discours |
| 616 | `conoscere` | section | Connaître; Connaître, avoir compétence pour juger |
| 729 | `battaglia` | section | Bataille, combat général ordonné entre deux armées |
| 812 | `otto` | section | Huit |
| 956 | `specialmente` | section | Spécialement, tout particulièrement |
| 967 | `caro` | section | Cher, coûteux; Carvi — Carum carvi ^(Linné - 1753) |
| 1027 | `veloce` | section | Rapide; Rapidement, Vite |
| 1045 | `sentenza` | section | Sentence, maxime |
| 1260 | `senato` | section | Sénat |
| 1300 | `cambiamento` | section | Changement, transformation |
| 1305 | `estremamente` | section | Extrêmement, d’une manière extrême, au dernier degré, au dernier point, à la dernière limite; Extr… |
| 1374 | `industriale` | section | Industriel, qui appartient à l’industrie; Industriel, qù l’industrie est développée; Industriel, q… |
| 1474 | `uccidere` | section | Rectifier, tuer, abattre |
| 1481 | `bambina` | section | Enfant |
| 2020 | `dottore` | section | Docteur, titulaire d’un doctorat ou médecin |
| 2311 | `austria` | section | Autriche |
| 2461 | `idiota` | section | Idiot, idiote |
| 2813 | `collegato` | section | Connecter, joindre ou attacher ensemble deux objets (machines, personne et macine ou serveur dista… |
| 2858 | `televisivo` | section | Télévisé |
| 3093 | `galera` | section | Galère; Prison |
| 3099 | `invasione` | section | Invasion, irruption faite dans le dessein de piller un pays, ou de s’en emparer; Invasion, action… |
| 3132 | `cerchio` | section | Cercle, dans un plan euclidien, lieu de tous les points équidistants d’un point donné appelé le ce… |
| 3304 | `fumetto` | section | Bande dessinée, œuvre utilisant des images fixes non purement photographiques, généralement des de… |
| 3408 | `contattare` | section | Contacter, entrer en contact, en relation avec |
| 3467 | `chirurgia` | section | Chirurgie |
| 4348 | `pago` | section | Satisfait, content; Paiement, paie; Pagus, subdivision territoriale médiévale; Pag, île croate en… |
| 4437 | `siracusa` | section | Syracuse, commune et ville de la région de Sicile en Italie; Syracuse, province autour de cette vi… |
| 4630 | `ferrovia` | section | Chemin de fer, mode de transport guidé dans lequel des convois, les trains, de véhicules, les wago… |
| 4893 | `commettere` | section | Commettre, construire, fabriquer, faire, opérer, poser |
| 4950 | `toscano` | section | Toscan |
| 5264 | `attrezzi` | section | Outil; Agrès, appareil |
| 5323 | `pci` | section | Abréviation de Partito Comunista Italiano, nom d’un parti politique italien; Abréviation de Partit… |
| 5758 | `nicole` | section | Nicole |
| 5807 | `evidenziare` | section | Mettre en évidence |
| 5822 | `interfaccia` | section | Interface |
| 5947 | `tigre` | section | Tigre |
| 6319 | `ministeriale` | section | Ministériel, qui concerne un ministère (ou des ministères) |
| 6392 | `ferroviario` | section | Ferroviaire, relatif aux chemins de fer |
| 6585 | `controparte` | fr tables | Adversaire, partie adverse |
| 6706 | `compressione` | section | Compression |
| 7033 | `este` | section | Este, commune d’Italie de la province de Padoue dans la région de Vénétie |
| 7060 | `mosaico` | section | Mosaïque, assemblage de morceaux de faïence représentant un motif artistique; Mosaïque, symptômes… |
| 7675 | `camorra` | section | Camorra; Confusion, désordre |
| 7774 | `strappare` | section | Arracher, détacher avec effort, ôter de force |
| 7934 | `biodiversità` | section | Biodiversité, diversité des organismes vivants, qui s’apprécie en considérant la diversité des esp… |
| 8145 | `riunito` | section | Ensemble composé d’un fauteuil spécialisé et de divers outils utilisés pour les soins dentaires ou… |
| 8493 | `incomprensibile` | section | Incompréhensible |
| 9004 | `delinquere` | section | Commettre un délit |
| 9355 | `spargere` | section | Répandre, verser |
| 9694 | `fisiologia` | section | Physiologie |
| 9803 | `chiarimento` | section | Éclaircissement, action de rendre moins serré, moins épais; Éclaircissement, explication d’une cho… |
| 9881 | `paranoia` | section | Paranoïa |
| 9966 | `elba` | section | Elbe, île italienne; Elbe, fleuve |
| 10071 | `celentano` | section | Nom de famille italien |
| 10077 | `collisione` | section | Collision, choc de deux corps |
| 10085 | `cosmico` | section | Cosmique, relatif à l’univers |
| 10298 | `nanna` | section | Dodo, lit dans le langage enfantin; Sommeil en parlant des enfants |
| 10545 | `caucaso` | section | Caucase, chaîne de montagnes |
| 10591 | `kung` | section | Nom de famille |
| 10616 | `nuoro` | section | Nuoro, ville de la région de Sardaigne en Italie; Nuoro, province autour de cette ville |
| 10831 | `astronauta` | section | Astronaute |
| 11288 | `transessuale` | section | Transsexuel |
| 11373 | `innamorare` | section | Séduire, rendre amoureux |
| 11423 | `primaverile` | section | Printanier, qui se rapporte au printemps |
| 11847 | `conscio` | section | Conscient, qui a une connaissance claire de soi, de la réalité des choses, de se qu’il éprouve; Co… |
| 11960 | `telo` | section | Toile, tissu d'armure simple, de fils de lin, de chanvre, de coton, etc; Flèche, trait qu’on lance… |
| 12105 | `spacco` | section | Fente |
| 12691 | `aloe` | fr tables | Aloès |
| 13136 | `gelateria` | section | Crèmerie, endroit où on vend des glaces; Glacier, établissement qui vend des glaces |
| 13617 | `tempistica` | fr tables | Timing |
| 13671 | `cons` | section | Nom de famille |
| 13747 | `ossidazione` | section | Oxydation, corrosion par l’oxygène de l’air ou celui dissous dans l’eau; Oxydation, réaction chimi… |
| 13928 | `misteriosamente` | section | Mystérieusement, avec mystère |
| 14306 | `longitudine` | section | Longitude |
| 14838 | `demente` | fr tables | Fou; Aliéné |
| 14916 | `notificato` | section | Notifier |
| 15486 | `gradazione` | section | Gradation |
| 15578 | `sopracciglio` | section | Sourcil |
| 15582 | `stelvio` | section | Stelvio, commune d’Italie de la province de Bolzano dans la région du Trentin-Haut-Adige |
| 15990 | `redistribuzione` | section | Redistribution |
| 16398 | `ostruzione` | section | Obstruction |
| 16660 | `rouen` | section | Rouen (ville de Normandie) |
| 16802 | `indovino` | section | Devin |
| 17061 | `incenso` | section | Encens |
| 18276 | `remunerazione` | section | Rémunération |
| 18291 | `setaccio` | section | Tamis, sorte de crible, de sas qui sert à passer des matières pulvérulentes, les liquides troubles… |
| 18350 | `ammonizione` | section | Admonestation, avertissement |
| 18595 | `archivista` | section | Archiviste, personne qui est préposée à la garde des archives |
| 18860 | `baxter` | section | Nom de famille |
| 18930 | `frittura` | section | Friture |
| 19010 | `panoramico` | section | Panoramique |
| 19104 | `zeppa` | section | Cale; Semelle compensée |
| 19226 | `landini` | section | Nom de famille |
| 19403 | `civic` | section | Nom de famille |
| 19438 | `elusione` | section | Fait d'éluder |
| 19527 | `olivieri` | section | Nom de famille |
| 19829 | `situa` | section | Situer |

**Known defects**: change 16's list went to change 17, which fixed or decided each item (*Refined:
change 17*, below).

## Refined: change 17 (refine-lingua-it-fr-glosses)

Rules of it-fr's own reducer (`reduce-it-fr.py`), each measured alone against change 16's committed
rows — a class counted in ranked lemmas by band (1–1,000 / 1,001–5,000 / 5,001–20,000 / beyond), a rule
kept where every row it changes among the 5,000 commonest reads right or no worse and those of
5,001–20,000 right in a majority, the criteria written before the figures —, with fr-en's treebank
rule imported as fr-es imports it: `reduce_common.py`, `reduce_edition_fr.py` and
`reduce_french_treebank.py` are not edited, and no other pair moves. The section as change 16 reads it
is refined (`refine_section`), a headword's entries — every case of it — read together:

| Rule | What it does | Rows (bands) | For example |
|---|---|---|---|
| D2, MarkIT's order | a function word's row, or one the section opens on a proper noun, opens on the part of speech UD Italian-MarkIT's training and development sections read it as at least 10 times and twice as often as the section's first, never a proper noun (`reduce_french_treebank.commonest_first`); a reviewed table, `FIRST_RUN`, orders what MarkIT cannot (the owner, 2026-10-25): `come` held at « Comme », `gente`, `fino`, `pure`, `loro` | 14 (14 / 0 / 0 / 0), 13 first senses | `fra` « Entre, durant; Frère », `spesso` « Beaucoup, fréquemment, souvent; Gros, épais », `proprio`, `molto`, `solo`, `uno`, `più` (an adverb's run); `gente` « Gens… », `fino` « Jusque; Fin », `loro` « Ils, elles, eux; … » |
| D3, a name its common entry repeats | a proper noun's sense a lowercase entry of a common part of speech gives word for word is the common entry's | 12 (0 / 3 / 5 / 4), 2 | `bibbia`, `rinascimento`, `corano` read as nouns, so dictionary words with a level |
| D4, no name under a function word | a proper noun's entry, no acronym, left out of a word whose lowercase entries are all a function word's or an adverb's | 5 (5 / 0 / 0 / 0) | `io` (Io, Inachus's daughter), `qui` « Ici » (Riri), `qua`, `oltre`, `anzi` |
| D5, pointers read | an untagged « Pluriel de X », an alternative form not marked obsolete, archaic or dated, a superlative's or a plural's quoted meaning | 28 (0 / 1 / 7 / 20), 60 lemmas gained (0 / 8 / 13 / 39) | `lettere`, `terzi` « Tiers », `paro` « Paire », `pochissimo` « Très peu », `ceneri` « Cendre », `can` « Khan, prince mongol »; `fora`, `maio` not followed |
| D6, a pointer lending beside a name or an old sense | a noun's or an adjective's pointer lends its word's senses when the word's other senses are a capitalised entry's or outdated, the lent entries first | 3 (0 / 1 / 2 / 0) | `gemelli` « Jumeau; Gémeaux…; Bouton de manchette », `spezie` « Épice…; Espèce », `arme`; never a verb form's (`regolo`) |
| D8, the `di` family and the articles' notes | the `di` family's links read as glosses, « Article contracté souvent traduit par X » read « X » where the word has no other glossed sense, a usage note out of a determiner's or a preposition's sense | 10 (6 / 1 / 2 / 1), 1 gained | `della` « De la », `degli`, `delle` « Des », `del` « Du », `cogli` « Avec les »; `sebbene` keeps its note |
| D9, communes | a proper noun's sense naming its place a commune right after its name is a name's note, set aside as change 16's are — but a chef-lieu's or a capital's, a provincial capital's (« Province autour de cette ville ») and the reviewed list's (`COMMUNES_KEPT`: `piacenza`, `cesena`, the owner, 2026-10-25) | 14 (2 / 4 / 5 / 3), `trani` and `melano` left with no gloss | `vittoria` « Victoire », `ro` « Rhô », `don`, `lana`; `bordeaux`, `colonia`, `matera` keep their city |
| D10, the example, the corrections, split words | an Italian example joined to its French by a dash goes; `CORRECTIONS` (below); a word split after its elided piece holding `j`, `k`, `w`, `x`, `y` or a second apostrophe is no expression | 1 example row (`io`'s three senses); 3 split words | `c'hwerty`, `l'wha`, `d'n'b` |

**The corrections** (`CORRECTIONS`, the owner's reading, 2026-10-25), each keyed by the section's text
as the pinned file writes it — an update finding one firing on nothing names it on the report's line,
for it to be removed — and reported to the French Wiktionary: `che` gains the relative pronoun « Qui,
que » and the conjunction « Que », read before its own entries (« Qui, que; Quoi; Que; Quel »);
`volere` loses an example filed as three senses; `dirotto` gains the adjective « À verse (dans a
dirotto) » (its participle lent `dirompere`'s « Écanguer »); `dati`, a lemma of its own by the owner's
row, reads « Données », the plural.

**Italian's forms** (`../it/`, D11), judged on the dictionary's entries and MarkIT's training and
development counts: a verb's person whose own entry holds no meaning — the verb's inflection written
without its tag, an ellipsis, another part of speech's pointer marked obsolete, archaic, literary,
poetic, dated or colloquial — reads as its verb (`sto` → *stare*, `vado` → *andare*, `sapete` →
*sapere*; never a function word's homograph, a contraction, an acronym's headword or another word's
inflection: `contra`, `cogli`, `tari`, `dee`, `mei` keep theirs); no form of a form through a plural of
a plural (`viti` leaves the tables, `crochi` → *croco*); no form from a feminine noun's masculine row
its form's own entry does not name (`signorini` → *signorino*; `gattaro`, `lavandaio`, `pecoro`,
`puttano` leave); no row of a table the readings set aside outranks the form's own entry naming another
ranked word (`avi`, `ava` → *avo*, `pozzi` → *pozzo*, `fuor` → *fuori*, `volse` → *volgere*), and no
heading kaikki reads as a cell gives a form (« archaic », « infinitive », « subjunctive » leave;
`imperative` reads as *imperativo*, its own entry). `signor` and `od` join the apocopes; `usa` → *usare*
and the owner's `dati`, `lega`, `prova` join the overrides. **The readings** (D12): `fate` reads no
subjunctive (`READINGS_SET_ASIDE`, the reduction failing on a row it does not find).

**The figures**, reduced locally from the pinned sources as the update reduces them (Python 3.12 and
`requirements-reduce.txt`; `main`'s rules first gave the committed tables byte for byte) at
`pack_version` `2026.10.24+d85215a`:
- `forms.tsv` 129,610 → 129,629 lines: 58 forms change lemma, 10 leave, 29 join; `freq.tsv` moves by
  renumbering alone — 29 lemmas leave (`sto`, `vado`, `signor`, `sapete`, `speriamo`, `torno`, `guardo`,
  `incontra`, `od`, `segui` … `sieno`) and `dati` (259), `prova` (302), `lega` (709) and 26 lemmas at the
  cut join, the 60,000th (`modino`) at Zipf 2.06; `grammar.tsv` moves in 62 forms, its own and `other`
  marks where a form changes lemma, and `fate`'s subjunctive;
- 32,741 glosses (31,088 from the section, 278 from the Italian Wiktionary, 1,374 read backwards, the
  `USA` row); coverage **89.4 / 82.0 / 73.4 %** against the floor 87.2 / 79.8 / 71.3; against change 16's
  rows, 92 change (27 / 11 / 22 / 32), 59 in their first sense, 71 lemmas gain a gloss (`dati`, `prova`,
  `lega` among them) and 21 lose one (`trani`, `melano`, and the 19 that leave the ranks);
- 18,293 expressions; 24,269 dictionary words (+69, −16); 8,302 levels, 19 joining (`dati`, `prova`,
  `lega` A1; `lettere`, `terzi` A2; `bibbia`, `gemelli`, `pochissimo` B1; `rinascimento`, `corano` B2 …)
  and 19 leaving (`sto`, `vado` A1, `sapete`, `speriamo` A2 … and nine at C2's end), 25 changing band;
- the pack 2,644,651 B (sha256 `26c51cc8…`), its base 1,551,202 B (`a2af1872…`), its layer 1,093,689 B
  (`f9befdeb…`);
- UD Italian-PUD **99.27 / 91.37 / 99.90** (19,155 words, 8,909 content words, 985 auxiliaries; the gates
  hold), read after the rules: +5 content words — `signor` +1 (*signore*), `lega` +1, `prova` +2, and its
  plural `prove` +2 (no form before, *prova*'s again), `dati` −1 (PUD lemmatises its ten nouns *dato*,
  which stay misses, and its one participle *dare*) — and 2 more words resolved (`prove`); MarkIT's test
  section 99.43 / 94.65 / 98.87, unchanged.

**Left, with their reasons** (D13): 494 lemmas of the levelled span the levels pass over because no
source glosses them — the 25 commonest
Italian words among them, to contribute upstream: `ex`, `post`, `ok`, `web`, `beh`, `entro`, `oh`, `pro`,
`club`, `supporto`, `super`, `prof`, `super-io`, `leader`, `live`, `anti`, `forum`, `max`, `mm`, `frega`,
`set`, `city`, `mar`, `rio`, `email` —; names spelled as a common word's form (`Rossi` *rosso*,
`Bianchi` *bianco*, `Conti` *conto*, `Bari` *barare*, as French's `Lot`), `Esposito` and `Ferrari` after
« Dott. » and « Prof. », the press's « la Nato », « gli Usa », « tra USA e NATO » and « dai retta » —
Italian's analysis's, in the programme's backlog —; the `di` retry's partitives and its 30 literal
phrases, which stay offered; species named in common rows; a verb form's borrowed senses, read as its
verb's (`assicurato` « Assurer, certifier, garantir »: the card names the verb, the owner's choice, D7);
surnames glossed « Nom de famille » alone (`lexa`, `santangelo`), names, so no dictionary words; and
`dati`'s own reading, a masculine *singular* noun — the English section heads « data (plural of dato) »
without a number —, which no rule of the readings reads otherwise.

## Measured

`scripts/lingua-data/measure/it-ud.sh` builds the pack from these tables and runs the real analyser
(`lingua-pack-measure`, Italian at `1.1.0`) over UD Italian-PUD at
`71d932aa096368331ba2fd9bc906ec8dd853da8e` — gated — and over MarkIT's test section at `e210079d`,
reported. The reduction reads neither. Punctuation, numbers, symbols, foreign words and proper nouns
are left out. **Italian's gates** are 98.5 % resolved, **90.5 %** of content words and 97 % of
auxiliaries: the content gate is the owner's (2026-10-23), Spanish's 93.5 % being out of reach for
Italian — MarkIT's counts are about sixteen times thinner than GSD's, UD reads a participle as its
verb where the dictionary keeps the noun or the adjective, and none of the variants measured reached
it (add-lingua-italian-forms-tables, its design's *Measured at implementation*).

| | Words | Resolved (98.5) | Content words (90.5) | Auxiliaries (97) |
|---|---|---|---|---|
| UD Italian-PUD (gate) | 19,155 | **99.26 %** | **91.38 %** of 8,909 | **99.90 %** of 985 |
| MarkIT test (reported) | 8,985 | 99.43 % | 94.68 % of 4,113 | 98.87 % of 530 |

The pack these tables build, sha256
`2e03059fd3dcfcf6e80ce25b8e4f0f4f55dbec1b5114d6caa515e17d4f4de1c3` (1,287,014 B), measured on
2026-10-23. Reduced again on another machine from the release's asset and the pinned sources, with
the pinned interpreter and wordfreq, the tables come out byte for byte. At implementation, on
kaikki.org's per-language extract of 2026-10-21 (623,702 entries, against this file's 624,122), the
same rules read PUD 99.26 / 91.39 / 99.90 and MarkIT's test section 99.43 / 94.65 / 98.87.

**On the tables of 2026.10.24** (change 16's update; pack `0e6795d9…`, 2,645,342 B), measured on
2026-10-24 with Italian at `1.4.0`: PUD **99.26 / 91.31 / 99.90** (19,155 words, 8,909 content words,
985 auxiliaries; the gates hold), MarkIT's test section 99.43 / 94.65 / 98.87. These are the figures
the tables of 2026.10.23 give on the same day too: change 16's glosses, dictionary words and levels
move no lemma; the content figure moved from 91.38 with the analyser's later changes, not
here.

What misses, read on these tables by the lookup (the cascade's first step, which reproduces the
binary's figures): one auxiliary of 985 (`voglia`, PUD's *volere*); 768 of PUD's 8,909 content words
— 610 are one lemma per form and the treebank's conventions (`prima` read as *prima* where PUD
writes *primo* ×18; the noun *stato*, « lo Stato », read as *essere* by the owner's override ×18; a
participle the treebank reads as its verb where the section's adjective or noun entry keeps it,
`fatto` ×12, `utilizzato`; `dati`, « i dati », read as *dare* ×10; `successo` read as *succedere*
×8), 77 adverbs in *-mente* the treebank lemmatises as their adjective (`completamente` →
*completo*), which the tables keep as words of their own, 31 feminine nouns it lemmatises as the
masculine (`ragazza` → *ragazzo*), and 50 words the tables do not list, most of them the truncated
infinitives UD splits off a verb and its clitics (`verificar`, `riunir`). Without the two
convention-only classes, 92.50 % (8,141 of 8,801).

**What decides the lemmas (D5).** Of the 15,489 forms reaching several lemmas: 3 by an override,
1,268 by MarkIT's counts, 6,045 by the form's own entry, 8,134 by the lemma's frequency, 39
alphabetically. The names rule reads 1,710 names alone as another word.

**The cut (D7).** 60,000 lemmas, the last (`aiu`) at Zipf 2.06; 128,429 forms (French: 124,040),
`forms.tsv` 2,390,960 B, `freq.tsv` 862,388 B; the pack 1,287,014 B (24.5 % of the 5 MiB budget),
its base 1,285,594 B, its layer 1,645 B. 98.21 % of PUD's measured words have a ranked gold lemma
(96.86 % of its content words); MarkIT test 97.83 % (97.93 %).

**Morph-it!** adds 809 forms under 638 ranked lemmas (127,620 forms without it) — truncated
infinitives (`aumentar`, `scavar`), truncated third persons plural (`danzan`), past historic
variants (`battè`), superlatives, `trent'` —, moving no reading and no rank; without it, these rules
read PUD 99.22 / 91.30 / 99.90 and MarkIT's test section 99.35 / 94.51 / 98.87 (+0.04 / +0.08 / 0
and +0.08 / +0.17 / 0 with it). The owner keeps it (2026-10-23).

## What one lemma per form costs (M8)

A lemma is keyed by its own form, so a dictionary noun whose form reads as a verb's leaves the pack:
« la porta » shows *portare*'s card. Among wordfreq's 1,000, 2,000 and 5,000 commonest words, **30,
44 and 66** dictionary nouns read as a verb and are no ranked lemma (Spanish 30 / 48 / 95, French 30
/ 55 / 135), measured on these tables — 32 / 47 / 69 before change 17's rows for `dati`, `prova` and
`lega`. Among the 2,000 (rank, noun, verb): 21 sono essere, 50 era essere, 51 stato essere (the
owner's override), 84 fa fare, 105 sei essere, 156 detto dire, 181 visto vedere, 186 so sapere, 256
dato dare, 267 credo credere, 335 successo succedere, 342 scritto scrivere, 352 porta portare, 354
state essere, 359 letto leggere, 383 possa potere, 499 fatta fare, 516 messo mettere, 528 ero essere,
559 pare parere, 568 trovato trovare, 675 cerca cercare, 732 nota notare, 758 portato portare, 853
iniziato iniziare, 882 vale valere, 924 chiama chiamare, 935 resta restare, 960 scoperto scoprire,
983 passa passare, 1007 parlato parlare, 1214 permesso permettere, 1261 creato creare, 1287
conosciuto conoscere, 1456 stia stare, 1567 caccia cacciare, 1604 conta contare, 1621 reso rendere,
1701 scritta scrivere, 1704 tratto trarre, 1793 formato formare, 1855 punta puntare, 1903 venuto
venire, 1942 aggiunto aggiungere. An override row keeps one noun and reads every verb form it shares
as the noun; the owner may name rows, each with its reason, at any reduction. Change 17's, the
owner's (2026-10-25): `dati` → *dati*, the plural noun as a lemma of its own (« i dati »), `lega` →
*lega* (`leghe` its plural again) and `prova` → *prova* (`prove` too); `dato` stays *dare*'s (MarkIT's
counts, 9 *dare* against 3 *dato*): « mi ha dato » is the very common participle, which the noun
« donnée » would misread in conversation.

## The levels (`../it/level.tsv`)

**Estimated, not a CEFR list's** (add-lingua-italian-levels, change 14, the programme's I8). No
Italian CEFR word list can be shipped: the *Profilo della lingua italiana* is not free, and no openly
licensed Italian list was found. Italian's levels are estimated from frequency, as Spanish's and
French's are, and the manifest says `levels_estimated`: the extension labels them estimated and the
ladder borrows English's typical vocabularies, saying so — no code of their own; Italian is shown by
those paths once change 20 widens the extension's studied languages. No list is read, measured against
or committed; a licence request for one stays the owner's.

**The derivation** (D1). The lemmas of `../it/freq.tsv`, in rank order then lemma, take English's
level sizes — 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2: 8,302 lemmas — skipping those a
CEFR list would leave out, a lemma left out giving its slot to the next. The sizes are constants of
`reduce-it-fr.py` (`ENGLISH_BANDS`), equal to fr-en's by a test and never read from English's tables,
so an English update cannot move Italian's levels unannounced.

**Which lemmas take one** (D2) is read from the English Wiktionary's Italian section — the source of
the forms —, and since change 16 among Italian's dictionary words alone (D4; add-lingua-pack-it-fr
D5). Within the levelled span (ranks 1–11,651, change 17's tables), 3,349 ranked lemmas take no
level:

| Rule | Left out | For example |
|---|---|---|
| 1. the section gives it no sense that is not a form of another word… | 1,995 | `facebook`, `of`, `eh`, `ecc`, `km`, `http`: none has an entry in the section |
| …or only a name's, written there with a capital | 707 | `italia`, `milano`, `europa`, `giovanni`, `google` |
| 2. a single character the section gives no word's sense — a letter's, its name read by its gloss, a symbol's, a name's, an abbreviation's, another word's spelling | 16 | `b`, `r` (« alternative spelling of er »), `x` (« abbreviation of per »), `é`, `u` (« the name of the Latin script letter U/u »), `k`, `à` (« obsolete spelling of ha »); `a` (the preposition), `e` and `o` (the conjunctions) keep their place |
| 3. every sense it is given, a name's aside, only spells another word, read by French's prefixes, not kaikki's `alt_of` pointer — so a contraction or an apocope stays a word | 9 | `the` (« misspelling of tè »), `nè`, `can`, `are`, `contenente`, `zombie`, `buddha`, `ro`, `homo` |
| 4. its own form reads as another lemma in `forms.tsv` | 0 | none: the ranks keep no such lemma (change 9's D6) and the builder refuses one; the rule stays as a guard |
| 5. Italian's dictionary words do not list it | 622 | since change 16: a name it-fr glosses by a proper noun alone (`roma`, `napoli`, `maria`, `francesco`, `gesù`, `luigi`; 128) or a word the sources do not gloss (`ok`, `web`, `club`, `entro`, `supporto`; 494); `bibbia`, `rinascimento` and `corano` are words since change 17 |

Every word the pre-pass writes has a level through its lemma: the `di` family (`del`, `della`, …),
the articles (*il*, *lo*, *uno*), the elided pieces' and the truncations' words and the split
prepositions' pieces are A1 — `ciascuno`, `dovere` (`deve`), `povero` and `venti` A2, `alcuno` B2. `al` and `nel` are no
words of the tables; `è` and `i` no lemmas (*essere*, *il*); `perchè` reads as *perché* (A1). M8's cost
carries over: `porta` is no lemma, so the door has no level and « la porta » reads as *portare* (A1).
The acronym rows have no rank, so no level.

| Level | Lemmas | Ranks | First words |
|---|---|---|---|
| A1 | 1,020 | 1–1,102 | di, e, che, il, a, in, non, per |
| A2 | 1,158 | 1,103–2,469 | occidentale, partecipare, principalmente, principio, quartiere, ridere, risolvere, sostegno |
| B1 | 2,015 | 2,470–5,108 | organo, palco, parcheggio, pilota, pistola, profondità, quindici, rapida |
| B2 | 2,347 | 5,111–8,658 | cocaina, considerevole, convocazione, cuffie, didattica, ebreo, economicamente, energetico |
| C1 | 886 | 8,659–10,151 | riconciliazione, salato, sbarramento, sbloccare, scadente, scandaloso, semplificare, sensore |
| C2 | 876 | 10,153–11,651 | prefisso, preposto, privatizzazione, rallentamento, rampa, ranch, rapire, restrizione |

The table's figures are change 17's tables: 19 lemmas joined the levels (`dati`, `prova`, `lega`, ranked
again; `lettere`, `terzi`, `pochissimo`, `ceneri`, glossed by its pointer rules; `bibbia`, `rinascimento`,
`corano`, `gemelli`, words again) and 19 left (ten verb forms read as their verb, `sto`, `vado`,
`sapete`…, and nine at C2's end), 25 changed band. On the tables of 2026.10.24 (change 16), 578 lemmas
left the levels — 458 that it-fr does not gloss, 120 it glosses by a proper noun alone — and 578
joined, 1,162 changed band.

These are a local reduction's, made as the update makes it — Python 3.12 and
`requirements-reduce.txt`, `build.sh --reduce it-fr` from the pinned 2026.10.23 sources (the release
assets `lingua-pack-sources-it-fr-2026.10.23` and `lingua-pack-sources-morph-it-0.4.8`, MarkIT at
`e210079d`) — which first reproduced the committed tables byte for byte; `forms.tsv`, `freq.tsv`,
`grammar.tsv`, `lexical.tsv`, `tags.tsv` and `studied.json` come out as committed.

**Measured with open data only** (the proposal's measurement, re-run on change 9's ranks). One script
over the three languages: a levelled lemma's translation is the first word of its English glosses —
fr-en's and es-en's `gloss.tsv` for French and Spanish, the section's own glosses for Italian (each
sense that is no form of another word and no name's, in page order) — that English's committed CEFR
table levels, lowercased, text in brackets left out; « skipping » leaves out a leading *to*, *a*,
*an* or *the*. PUD: UD Italian-, French- and Spanish-PUD's test sections (the same 1,000 sentences;
gold lemmas; names, numbers, symbols and punctuation left out), each language's committed `level.tsv`.

| | Italian | French | Spanish |
|---|---|---|---|
| A levelled lemma's translation and its English level: exact / within one | 22.7 % / 54.1 % of 7,075 (85.2 % covered) | 23.1 % / 54.1 % of 6,909 (83.2 %) | 24.4 % / 55.7 % of 6,790 (81.8 %) |
| The same, skipping a leading article or *to* | 24.6 % / 57.9 % | 25.5 % / 60.3 % | 26.7 % / 61.1 % |
| Mean English level of the translations, A1 → C2 | 1.76, 2.18, 2.38, 2.37, 2.52, 2.49 | 1.67, 2.14, 2.29, 2.44, 2.32, 2.37 | 1.74, 2.19, 2.38, 2.55, 2.48, 2.42 |
| A1 words whose translation is English A1 / A1–A2 | 54.2 % / 76.3 % | 58.9 % / 80.0 % | 55.0 % / 76.5 % |
| UD PUD's words at A1, A2, B1, B2, C1, C2, none | 75.8, 8.0, 6.0, 3.5, 0.8, 0.5, 5.4 % of 19,180 | 77.7, 7.8, 5.7, 2.7, 0.6, 0.5, 5.0 % of 20,411 | 76.5, 8.1, 6.0, 2.6, 0.8, 0.4, 5.6 % of 19,285 |
| PUD's words presumed known when declaring A2, B1, B2, C1, C2 | 75.8, 83.8, 89.7, 93.2, 94.1 % | 77.7, 85.5, 91.2, 93.8, 94.4 % | 76.5, 84.6, 90.6, 93.2, 94.0 % |

Italian's estimate reads like French's and Spanish's: its exact and within-one figures are within two
points of theirs, its A1 like Spanish's, and a declared level presumes known within two points of what
it does in French and Spanish (Spanish's PUD shares are within a tenth of change 46's; French's moved
with its later tables). Two figures lag: skipping articles, Italian's within-one is 57.9 % against
60.3 % and 61.1 % (the proposal's prototype measured the same gap, 59.4 % against 62.6 % and 62.8 %),
missing by 0.4 point the floor written before the figures were read (the lower of French's and
Spanish's less two points); and the plain proxy's mean English level does not rise from B1 to B2
(2.38, 2.37), missing the second criterion written then (rising from A1 to B2) — it rises when skipping
(2.52, 2.57); in all three languages the proxy cannot tell B2, C1 and C2 apart, a rare word's
translation being often a common English word. The script does not reproduce the proposal's figures
(its method was not recorded; of 72 variants tried on French and Spanish, the closest is up to 2.3
points off), so its figures compare with each other only.

**Against the proposal's figures**, measured on a prototype (wordfreq's list, each word ranked by its
own string): A1 ended at rank 1,094 and C2 spanned 9,446–10,768; here 1,080 and 9,377–10,690. 2,388
lemmas are left out instead of 2,466: 272 fewer names — exactly the 272 name-only words among wordfreq's
14,000 commonest that change 9's names rule (its D5) reads as another lemma's form and does not rank
(`sua` → *suo*, `grandi` → *grande*, `bambini` → *bambino*, `rossi` → *rosso*); 205 more unknown words,
none of them in the section; 9 spellings instead of 18, change 10 reading most as their word (`perchè`
→ *perché*); 16 letters instead of 18, wordfreq's elision stems (`l`, `d`) being no ranks.

**Known weaknesses**, as French's and Spanish's: until change 16 narrowed the levels to dictionary
words (D4), a name kept a level through a sense of another part of speech, from a lowercase or a
capitalised entry. Since the tables of 2026.10.24, `roma`, `marco`, `napoli`, `francesco`, `maria`,
`gesù`, `luigi`, `adam`, `assunta`, `palestina`, `bellini`, `reich` and the initialisms `mm`, `cd`,
`ps`, `ac`, `tar`, `cup` take none; `svizzera` (A2), `alzheimer` (C1), `pc` (A2) and `pg` (B2) keep one,
glossed as nouns by it-fr. Before it, from a lowercase entry: `roma` (« a type of rice »), `marco` (the mark),
`napoli` (« Neapolitan ») and `francesco` (« French ») A1, `luigi` (the louis d'or), `svizzera` (a
hamburger), `brasile` (brazilwood) and `matteo` (« alternative form of matto ») A2. From a capitalised
entry — 59 levelled lemmas take their only senses that are no name's from one, words written in
capitals a list would hold among them (`tv`, `ok`, `dna`, `sms`) —: `maria` (« cannabis », from
`Maria`) and `gesù` A1, `adam`, `assunta` and `tommy` B1, `palestina`, `cambogia` and `bellini` B2,
`reich` and `alzheimer` C2, and initialisms, `mm`, `pc` and `cd` A2, `ps` and `ac` B1, `tar`, `pg` and
`cup` B2. A lemma is ranked by its own string, so a
word said mostly in other forms ranks low — `dovere` and `volere` A2, `alcuno` and `capello` B2; numbers
written as words are levelled by frequency (`undici` B1, `quattordici` B2, `diciannove` C1), and `sei`
reads as *essere* (M8), so six has none; rule 3 leaves out a word whose only sense of its own spells
another, though readers meet it as a word of its own: `contenente` (« containing »), given as
*contenere*'s participle (B1) and, of its own, only as « archaic form of continente ».

The levels section is one byte per lemma of the pack's pool, in Italian's base with the flag: the base
grows from 1,491,875 B to 1,551,912 B (+60,037 B), the layer from 1,653 B to 1,918 B (+265 B, the
NOTICE's sentence), the pack from 1,493,303 B to 1,553,605 B (29.6 % of the 5 MiB budget).

## The readings (`../it/grammar.tsv`)

Italian's word grammar (add-lingua-italian-grammar-tables, change 13): each form's readings, in the
vocabulary the word card reads (add-lingua-word-grammar), written by the same pass over the Italian
section as the forms, once they are chosen — the readings choose no form and no rank. Only the forms
`forms.tsv` holds carry readings, and only under the lemmas `freq.tsv` ranks. French's rules
(`../fr-en/README.md`, *The readings*), for Italian.

**The rules** (`reduce-it-fr.py`, after the forms):
- **Verbs.** kaikki's tags as UD's: the indicative's present, imperfect and future; the passato
  remoto (`historic past`) as `Tense=Past`; the present and imperfect subjunctive; the conditional and
  the imperative with **no tense** (UD Italian writes `Tense=Pres`), the imperative in its second
  person singular and first and second plural; the infinitive; the gerund (`VerbForm=Ger`); the
  present participle as `VerbForm=Part|Tense=Pres` with its number, no gender (`parlante`, and
  `parlanti` through the participle's own entry); the past participle with its gender and number, the
  table's bare row the masculine singular, the agreed forms through the participle's own entry
  (`creduta` → `creduto` → *credere*). Every verb is `VERB`. A table form reads as the forms read it:
  its stress marks dropped but the last letter's (`dièdi` → `diedi`). A sense merging persons or moods
  reads as each. **No reading** from the negative imperative (its second person singular is the
  infinitive, « non parlare », its plurals the affirmative's) nor the formal one (`Lei`, `Loro`: the
  present subjunctive's third persons, whose reading carries the form), a defective verb's `-`, a
  clitic cluster (`me ne vàdo`, `la hò`), a dialectal, hypercorrect, regional or register-marked row
  (`fò`, `peròro`), nor from **the Classical Roman Dialect's conjugation table**, which kaikki merges
  into an entry beside the Italian ones with no tag to say so (*avere*'s third table: `àmo`, `émo`,
  `ébbe` a first person, `avéssi` a passato remoto): when an entry has several tables, one whose
  present indicative's first person plural ends nowhere in Italian's `-iamo` while another's does is
  set aside — 26 tables of 25 verbs: the dialect's (the Classical Roman Dialect's, as *avere*'s page
  heads it) of *avere*, *essere*, *fare*, *dare*, *potere*, *volere*, *dovere*, *credere* and 16
  others, *essere*'s untagged usage notes, and *ire*'s first table, an alternative form's, which reads
  nothing anyway. The rows tagged `Traditional`, poetic, literary or Latinate read as their cell.
- **Reflexive verbs.** A row written with its pronoun reads without it, under the lemma the forms
  file it under (`mi làvo` → `lavo`, *lavare*'s; `mi pènto` → `pento`, *pentirsi*'s); a row carrying
  the pronoun reads as its bare base under a reflexive-only lemma (`pentìrsi` → `pentire`, `pèntiti`
  → `penti`) and gives nothing under a plain verb — never as its clitic-bearing string (`làvati` is no
  `lavati`, *lavare*'s participle). A verb whose infinitive carries another clitic (`andarsene`) reads
  nothing.
- **Nouns, adjectives, determiners (articles among them), pronouns, numerals.** A noun's gender from
  `it-noun`'s first argument (a reading per gender for `mfbysense`, `mf`, `m,f` and the qualified ones,
  `m-p` and `f-p` plural only), else its senses, on its own form and its plural; both numbers on a noun
  the dictionary gives one form for (`#`: `città`, `re`, `crisi`), the singular alone on an uncountable
  one (`-`: `gas`); a plural of another gender reads its own (`uova`, feminine plural of *uovo*); no
  reading from a feminine noun's masculine row, singular or plural (`attrice` → `attore`, `signorina`
  → `signorini`), nor from a derived word's row (`casétta`, `veramente`). An adjective's own form is
  masculine singular when it has a feminine of its own (`rosso`), singular when one form serves both
  (`grande`), its comparative `Degree=Cmp` (`minore` names *piccolo*), its superlative `Degree=Sup` —
  the masculine singular a head names (`bellissimo`, `abilissimo`, `minimo`), whose feminine and
  plurals its own entry gives. A determiner, pronoun or numeral reads its gender and number from its
  rows, a pronoun's row only with a gender, a plural-headed determiner's or pronoun's table none
  (`gli`, `le`); its own dictionary form names nothing on its own card. **The article table** kaikki
  writes identically under *il*, *lo*, *la*, *gli* and *le* reads in full under *il*, the lemma the
  forms file the articles under, and under any other article only for the forms its own head names:
  `i`, `la`, `gli`, `le` are *il*'s, `gli` names *lo* too (its plural), `il` names no *lo*.
- **Where a reading comes from.** A lemma's table first, a form's own entry for what no table lists —
  of the same part of speech: `grandi`, which *grande*'s noun table lists, keeps its adjective entry's
  `ADJ|Number=Plur`. A form of a form along one part of speech reads as the word the form is a form of,
  with its own agreement — never through a nominal plural: `colli`, the plural of `colle` (« hill »),
  is no form of *colla*, whose plural `colle` also is. A reading's part of speech is one its lemma's
  entries hold: a participle filed under a noun's spelling reads no verb form of it and names its verb
  (`parlata`, the noun, names *parlare*). **Change 10's spellings** read as their word: a dictionary
  spelling carries its word's own readings where nothing gives it its own (`perchè`, `caffé`, `fà`)
  and keeps them otherwise (`dà`, the section's « alternative form of dare », stays *dare*'s present);
  a spelling's table reads only in a part of speech the word's own table gives nothing of (`moglie`,
  which *moglia*'s table lists as its plural, stays the singular); a row of the web's list or an
  owner's exception carries those of its word's forms written with its letters, and none of its own
  (`puo` `può`'s, `perche` `perché`'s, never *perca*'s plural). **Never** from a row or sense change 9
  leaves out of the forms (doubtful), a form's own sense marked regional or of a register, a
  capitalised headword (`NATO`, `Roma`), an entry whose every sense is an alternative form of another
  word (an archaic spelling with a table of its own; the apocopes `bel`, `buon`, `gran`, `quel`), or a
  link an override row of change 9 sets aside.
- **`other` marks (M8).** A form's readings of its own lemma are its own; a reading of another ranked
  lemma is marked `other`, and the card names that word without counting it (`porti` names *porto*,
  `legge` *leggere*, `minore` *piccolo*), only toward an entry of the dictionary that is not only
  regional — and, at review, never on the dictionary's word alone where it marks the link:
  - **from poetry and letters**: a row or a form's own sense tagged poetic, Latinism or literary reads
    under the form's own lemma and names no other word, directly or through a form of a form (`sono`
    names no *suonare*, `fiero` no *essere*, `possente` no *potere*, `vinci` no *vinco*, `paro` no
    *pari*; `veggo`, `vegga` stay *vedere*'s own, `asperrima` *aspro*'s);
  - **from a doubtful spelling of a cell**: where an entry of the form is a doubtful spelling (an
    alternative, obsolete, archaic, misspelt… form) of a cell of a lemma's table, in that entry's part
    of speech, the form names that lemma with none of that cell's readings (`anno`, « misspelling of
    hanno », names no *avere*: kaikki drops the « disused » qualifier of *avere*'s `ànno`; `à`, `ò`,
    `dei`, `dee`); another cell's reading stands (`puzza`, a noun spelling *puzzo*, names *puzzare*'s
    present), and so does every reading of the form's own lemma (`diè`);
  - **toward an archaic or literary entry**: another word is named only in a part of speech an entry
    holds it as with a meaning not marked archaic, poetic, Latin or literary (`dire` names no *diro*,
    `idea` no *ideo*, `cane` no *cano*, `conte` no *conto* « known », `serve` no adjective *servo*,
    `armi` no *arme*); a reflexive's meaning names the plain verb it is filed under (`arreso`, of
    *arrendersi*, names *arrendere*). Its own readings are untouched. The marks it takes off that a
    reader might have wanted, the words being literary only in the dictionary: `cela` and `celato`
    (*celare*), `perito` and its forms (*perire*), `colei` (*colui*), `costei` (*costui*), `ella`, `esse`,
    `essi` (*egli*), `spoglie` (*spoglia*), `madonne` (*madonna*).

**What they hold**, reduced locally from the pinned sources as the update does (`build.sh --reduce
it-fr`, Python 3.12 and `requirements-reduce.txt`, the release's asset; main's reduction reproduced
first, every committed byte, and the first update's tables reproduced byte for byte by run
40033640362): **132,849 readings in 83 tags**; 94,786 of the 129,599 forms carry a reading of their own
lemma, 34,813 none (adverbs, prepositions, conjunctions, interjections, names, a determiner's or
pronoun's dictionary form, the apocopes and truncated infinitives); **13,366 `other` marks on 8,562
forms**. By part of speech: 66,229 verb readings, 40,707 noun, 25,737 adjective, 93 determiner, 75
pronoun, 8 numeral. Own readings per form:

| Readings | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| Forms | 77,424 | 11,527 | 4,374 | 1,422 | 39 |

The five-reading forms are invariable nouns of both genders that are adjectives too (`leader`,
`madrelingua`, `maori`) and `mi` (the note and the Greek letter). 2,883 ranked lemmas have a verb
reading on their forms; 19,110 of the 19,138 ranked lemmas whose own form reads as a noun carry a
gender. 9,499 readings come through a form of a form alone, 1,355 from change 10's spellings. The
pack these tables build is 1,493,303 B (28.5 % of the 5 MiB budget), its base 1,491,875 B, its layer
1,653 B.

**The review's rules, each one's rows** (from the first update's 133,066 readings): no form of a form
through a plural −65 (`testi` no longer *testa*'s; `signori` no *signora*'s; `viti`, `crochi`, which
change 9 files under *vita* and *croce*, keep no reading of them), a spelling's table only where the
word's gives nothing −16, poetry and letters naming nothing else −11, a doubtful spelling of a cell −6,
archaic and literary entries named on no card −118, a feminine noun's masculine plural −1
(`signorini`), the superlative's masculine 0 (324 readings re-tagged; the pool's
`ADJ|Degree=Sup|Number=Sing` became `ADJ|Degree=Sup|Gender=Masc|Number=Sing`): **−217, +0**.

**The moods on one form (M21, D7).** The readings are stored unmerged: `parli` carries four —
the present indicative's second person singular and the present subjunctive's first, second and third
—, `parla` two (the present indicative's third person singular, the imperative's second singular),
`parliamo` three. **2,582 forms read the present indicative and the present subjunctive of one person
and number**; 1,047 forms of 1,043 verbs carry exactly `parli`'s four (20 of the 1,000 commonest
lemmas, 141 of the 5,000), 1,659 `parla`'s two (30, 180), 1,326 `parliamo`'s three (62, 403). The two
presents carry `Tense=Pres` and the imperative none: whether the card merges them is change 18's call.

**The tag pool** (`../it/tags.tsv`): the 83 tags the readings carry, in byte order, written by a
person from the reduction's readings (`cut -f3 grammar.tsv | LC_ALL=C sort -u`; rewritten once at
review, before any pack shipped, when the superlative's masculine replaced a genderless tag) and kept
by every reduction; a tag a later reduction adds is appended after them, and the checks name it until
it is (`crates/lingua-pack/tests/committed_tables.rs`). It replaces change 9's placeholder `NOUN`, whose
index no shipped pack held. it-fr's base lays its pool out as the pin and nothing else: no reading
carries a tag outside it, and no sense of a pair enters a base.

**Measured** (`measure/it-ud.sh`, after its gates, `measure/it_readings.py`; reported, never gating):
over the words whose form the tables map to the treebank's lemma, the share carrying a reading of
their own **in the part of speech measured** (a pronoun the tables read only as a noun — `si`, `ci`,
`mi`, `ti`, the notes and letters — is not read as a pronoun), and the share of those whose features
are among those readings, the conditional's and the imperative's tense left aside. UD Italian-PUD
writes its verbs without `VerbForm` (the form by `XPOS`), the imperfetto `Aspect=Imp|Tense=Past` and
the imperfect subjunctive `Tense=Past`; the script reads them as MarkIT writes them. The forms' gates
do not move (PUD 99.26 / 91.31 / 99.90, MarkIT test 99.43 / 94.65 / 98.87, as on main).

| | Words | Read | Agree |
|---|---|---|---|
| PUD, finite verbs | 1,701 | 99.76 % | 98.70 % |
| PUD, participles | 510 | 97.84 % | 99.80 % |
| PUD, infinitives | 414 | 88.41 % | 100 % |
| PUD, gerunds | 91 | 100 % | 100 % |
| PUD, nouns | 4,159 | 97.57 % | 99.61 % |
| PUD, adjectives | 1,418 | 90.13 % | 99.45 % |
| PUD, determiners | 3,257 | 86.58 % | 99.96 % |
| PUD, pronouns | 827 | 17.41 % | 100 % |
| MarkIT test, finite verbs | 940 | 99.68 % | 99.47 % |
| MarkIT test, participles | 136 | 99.26 % | 99.26 % |
| MarkIT test, infinitives | 299 | 84.62 % | 99.60 % |
| MarkIT test, gerunds | 53 | 100 % | 100 % |
| MarkIT test, nouns | 1,792 | 97.77 % | 99.03 % |
| MarkIT test, adjectives | 611 | 89.85 % | 100 % |
| MarkIT test, determiners | 1,391 | 75.56 % | 99.81 % |
| MarkIT test, pronouns | 726 | 4.41 % | 96.88 % |

The infinitives read nothing when truncated (`aver` ×13, `far` ×9, `esser` ×5 among PUD's 48): the
section gives them as apocopes, alternative forms, which read nothing. A pronoun's dictionary form
reads nothing, as French's: the pronouns read are the agreed forms a pronoun's or a possessive's table
lists (`sua`, `suoi`, `quella`, `questa`; PUD writes the possessives as pronouns), while `che`, `cui`,
`loro`, `suo`, `questo` read nothing, and the clitics `si`, `ci`, `mi`, `ti` — 209 of PUD's 827 — read
only as the nouns their dictionary forms also are, so are not counted as read. What PUD's finite verbs disagree
with is mostly its own annotation (`cadde`, `accettarono` written `Tense=Pres`, `vidi` a third person,
`devi` an imperative).

**Each rule switched off alone** (the reducer's source patched one rule at a time, its base run giving
this `grammar.tsv` byte for byte):

| Rule switched off | Readings | Forms with one | `other` | Four-reading forms | What it moves |
|---|---|---|---|---|---|
| — (this reduction) | 132,849 | 94,786 | 13,366 | 1,422 | — |
| a reading's part of speech its lemma's (D5) | 137,435 | 94,795 | 14,848 | 1,451 | PUD participles read 97.84 → 98.82 %, adjectives 90.13 → 90.62 % |
| a form of a form (D5) | 123,350 | 90,132 | 10,968 | 1,218 | PUD participles read 97.84 → 65.49 %, agree 99.80 → 86.53 %; MarkIT's read 99.26 → 60.29 % |
| no form of a form through a plural (D5, review) | 132,914 | 94,788 | 13,429 | 1,422 | — |
| both numbers on an invariable noun (D4) | 130,350 | 94,786 | 13,366 | 1,262 | PUD nouns agree 99.61 → 97.81 % |
| change 10's spellings (D5) | 131,494 | 93,798 | 13,306 | 1,405 | — |
| a spelling's table only where the word's gives nothing (D5, review) | 132,865 | 94,786 | 13,366 | 1,424 | — |
| merged senses read as each (D3) | 132,824 | 94,782 | 13,366 | 1,418 | — |
| a reflexive's rows as change 9 reads them (D3) | 132,773 | 94,764 | 13,315 | 1,422 | — |
| a participle sense without `past` the past one (D3) | 132,811 | 94,774 | 13,352 | 1,422 | PUD participles read 97.84 → 97.25 % |
| articles as determiners (D4) | 132,841 | 94,781 | 13,364 | 1,422 | PUD determiners read 86.58 → 3.32 % |
| a table first per part of speech, not per pair (D5) | 132,753 | 94,776 | 13,346 | 1,422 | PUD determiners read 86.58 → 81.70 %, adjectives 90.13 → 89.56 % |
| the superlative's masculine (D4, review) | 132,849 | 94,786 | 13,366 | 1,422 | — (324 readings lose their gender) |

| Exclusion switched off | Readings | Forms with one | `other` | What it moves |
|---|---|---|---|---|
| the Roman dialect's tables read (D3) | 133,115 | 94,873 | 13,402 | — (`amo` would name *avere*, `ebbe` read a first person) |
| the negative and formal imperatives read (D3) | 135,750 | 94,787 | 13,366 | four-reading forms 1,422 → 1,436 |
| a regional or register row or sense read (D3, D5) | 132,867 | 94,802 | 13,368 | — |
| a derived word's row read (D4) | 133,317 | 94,786 | 13,834 | — |
| a feminine noun's masculine row read (D4) | 133,004 | 94,791 | 13,516 | — |
| a feminine noun's masculine plural row read (D4, review) | 132,850 | 94,787 | 13,366 | — |
| capitalised headwords read (D5) | 132,999 | 94,870 | 13,369 | PUD nouns read 97.57 → 97.69 % |
| alternative-only entries read (D5) | 133,440 | 95,142 | 13,407 | — |
| `other` toward a link-only or regional word (D6) | 133,034 | 94,786 | 13,551 | — |
| `other` from poetry and letters (D6, review) | 132,860 | 94,786 | 13,377 | — |
| `other` with a doubtful spelling's cell (D6, review) | 132,855 | 94,786 | 13,372 | — |
| `other` toward an archaic or literary entry (D6, review) | 132,967 | 94,786 | 13,484 | — |
| a composed reading over a direct one (D5) | 132,849 | 94,786 | 13,366 | — |
| a pronoun's row without a gender read (D4) | 132,854 | 94,787 | 13,369 | PUD pronouns read 17.41 → 18.62 % |
| a plural-headed table read (D4) | 132,850 | 94,786 | 13,367 | — |
| the article table in full under every article (D4) | 132,853 | 94,786 | 13,370 | — (`il` would name *lo*) |
| a head row repeating a table cell read (D4) | 132,853 | 94,786 | 13,367 | — (`i` would read a genderless plural) |
| a link an override row sets aside (D5) | 132,849 | 94,786 | 13,366 | — |

No rule was changed on these treebanks' figures: the rules were written, and the departures from the
design decided (`openspec/changes/add-lingua-italian-grammar-tables/design.md`, *As implemented*), on
the dictionary's own entries before they were measured; the review's, on its tags.

**M8's cost** is change 9's (30 / 44 / 66 since change 17's rows): the dictionary nouns read as a verb among wordfreq's
1,000 / 2,000 / 5,000, none of whose readings is kept — « la porta » opens *portare*'s card, which
names *porgere* (`porta`, the feminine of its participle `porto`) and no noun.

**Known data defects** (for change 17, `refine-lingua-it-fr-glosses`, and the section's editors):
- `braccio`'s plurals unexpanded (24 heads with a qualified plural argument): `braccia` reads from its
  own entry; `lenzuola`, `fondamenta`, `grida` likewise; `mura` reads as a lemma of its own;
- `bello`'s rows tagged `error-unrecognized-form`, and the shortened adjectives (`bel`, `buon`, `gran`,
  `san`, `quel`) and truncated infinitives (`aver`, `far`, `esser`) with no reading;
- `uova` → « uovo m » in its own entry's head; `gli` → « the article i »; `adulare`'s formal
  imperative `adùla`; `ottimo`'s canonical row and `migliore`'s « of buono » row;
- the Roman dialect's tables, set aside here, still give change 9 its `avi`, `ava` and `ate` →
  *avere* (and `ate`'s own entry names *avere*);
- change 9's chaining of a form of a form through a plural files `viti` under *vita* and `crochi` under
  *croce* (and `signorini` under *signorina*): the readings no longer follow it, so each reads none of
  its own lemma;
- an untagged old cell: `béi` makes `bei` name *bere*;
- the section's copy errors: `scalpitanti` « plural of diffondente » (read under *diffondere*, as
  change 9 files it); `imperative`, a cell of *essere*'s usage notes, filed under *essere*;
- the function words whose own lemma the dictionary also holds as a noun read the noun: `e`, `a`, `o`
  (the letters), `mi`, `ti`, `si`, `ci` (notes and letters), as Spanish's `a`, `de`, `mi` and French's
  `si`; `lui` and `lei` name each other.
The `-` cells are the source's notation for a missing cell, not a defect.

## Licences

The English Wiktionary's content (via kaikki.org): CC BY-SA 4.0 and GFDL. wordfreq's data: CC BY-SA
4.0. UD Italian-MarkIT: CC BY 4.0. Morph-it!: CC BY-SA 2.0 (dual with the LGPL), read under CC BY-SA
2.0. The NOTICE names each licence's address. UD Italian-PUD (CC BY-SA 3.0) is measured against and
never shipped. The tables are published under CC BY-SA 4.0 with the pack.
