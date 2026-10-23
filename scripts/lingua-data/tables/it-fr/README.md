# The it→fr dictionary tables

The reduced tables of Cymbra Lingua's Italian→French data pack, committed so that every build makes
the same pack with no download, and so that a change to the dictionary is a pull request whose diff
shows it (add-lingua-italian-forms-tables, change 9 of `docs/lingua/italian-programme.md`). No
extension package carries the pack yet: `packs.json` does not list it-fr.

They are in two folders (split-lingua-pack-tables-by-language). it-fr is Italian's reference pair
(`../it/studied.json`, the programme's I5): its reduction writes Italian's own tables in `../it/`,
which every pair studying Italian reads as committed (it-en, it-es), and this folder's `pin.json`
records the sources of both. This change writes Italian's forms and ranks; its glosses, readings,
levels and dictionary words are later changes' (16, 13, 14, 11), so `gloss.tsv` and
`../it/lexical.tsv` are empty and `../it/tags.tsv` holds `NOUN` alone. The Italian baseline
(`crates/lingua-wasm/tests/italian_baseline.rs`) keeps its hand-written fixture
(`scripts/lingua-data/testdata/it-fr/`) until change 16: a pack of forms and ranks alone would blind
its gloss, expression and level probes.

In this folder:

| File | What it maps | From |
|---|---|---|
| `gloss.tsv` | nothing yet: change 16 glosses Italian in French | — |
| `NOTICE` | the attribution stack, embedded in the pack | — |
| `manifest.json` | Italian glossed in French, Italian's analyser version (`ITALIAN_ANALYZER_VERSION`, read from lingua-core), `pack_version` (the snapshot); no `levels_estimated` until change 14 | — |
| `pin.json` | the raw sources these tables and `../it/` came from, and the pack, base and layer they build | — |

In `../it/`, Italian's tables, written by it-fr's reduction:

| File | What it maps | From |
|---|---|---|
| `forms.tsv` | form → lemma | kaikki.org, the English Wiktionary's Italian section (CC BY-SA 4.0 + GFDL), with UD Italian-MarkIT's counts to choose between lemmas (CC BY 4.0) and Morph-it!'s forms the Wiktionary names nowhere (CC BY-SA 2.0) |
| `freq.tsv` | lemma → frequency rank, the 60,000 commonest | wordfreq 3.1.1 (CC BY-SA 4.0); a hyphenated word by MarkIT's own frequency too |
| `lexical.tsv` | Italian's dictionary words: none yet | change 11's acronym rows and change 16's glosses |
| `tags.tsv` | the pinned tag pool: `NOUN`, the one tag change 4 pins for a language with no readings | committed by hand; change 13 writes Italian's pool |
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
- **wordfreq `it` 3.1.1**, pinned by version and by `../../requirements-reduce.txt`'s hashes.

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
  truncation `da'`'s word, no rank); no plain word beginning with a piece is a form (`c'è`,
  `dell'acqua`).
- **One lemma per form (D5, M8).** A form whose own lemma entries are names alone reads as the other
  word (`bambini`, a surname → *bambino*); `roma`, which no other word reads, stays the city; every
  other form is left to `OVERRIDES` — the owner's `stato` and `stati` → *essere* (2026-10-23; « lo
  Stato » reads as *essere*, the accepted cost), and `bianca` → *bianco* —, then MarkIT's counts,
  the form's own entry, the lemma's frequency, the alphabet.
- **The apocopes** written without an apostrophe read as their word alone by a reviewed table
  (`APOCOPES`): `un` → *uno*, `buon` → *buono*, `nessun`, `alcun`, `ciascun`, `qualcun`, `bel` →
  *bello*, `san` → *santo*, `gran` → *grande*, `quel` → *quello*, `tal` → *tale*, `qual` → *quale*.
- **Morph-it! (D3).** A form it lists under a ranked lemma, which the Wiktionary names nowhere,
  joins the table — never one carrying clitics, never moving a reading or a rank.
- **The ranks (D6).** wordfreq's order, dense, the 60,000 first, over its words that are not only
  inflected forms; a word whose own form reads as another gives its rank to the next. wordfreq's
  stems of the elided pieces (`l`, `d`, `gl`, `quest`, `dell`, …) and of the truncations (`po`,
  `mo`, `be`) are no words, nor its tokens beginning with a piece, but the apocopes (read as their
  words) and `cent`, the euro cent; a hyphenated word is ranked when MarkIT attests it, at the lower
  of wordfreq's estimate and MarkIT's frequency.

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
« la porta » shows *portare*'s card. Among wordfreq's 1,000, 2,000 and 5,000 commonest words, **32,
47 and 69** dictionary nouns read as a verb and are no ranked lemma (Spanish 30 / 48 / 95, French 30
/ 55 / 135), measured on these tables. Among the 2,000 (rank, noun, verb): 21 sono essere, 50 era
essere, 51 stato essere (the owner's override), 84 fa fare, 105 sei essere, 156 detto dire, 181
visto vedere, 186 so sapere, 256 dato dare, 267 credo credere, 335 successo succedere, 342 scritto
scrivere, 352 porta portare, 354 state essere, 359 letto leggere, 383 possa potere, 424 dati dare,
491 prova provare, 499 fatta fare, 516 messo mettere, 528 ero essere, 559 pare parere, 568 trovato
trovare, 675 cerca cercare, 732 nota notare, 758 portato portare, 853 iniziato iniziare, 882 vale
valere, 924 chiama chiamare, 935 resta restare, 960 scoperto scoprire, 983 passa passare, 1007
parlato parlare, 1112 lega legare, 1214 permesso permettere, 1261 creato creare, 1287 conosciuto
conoscere, 1456 stia stare, 1567 caccia cacciare, 1604 conta contare, 1621 reso rendere, 1701
scritta scrivere, 1704 tratto trarre, 1793 formato formare, 1855 punta puntare, 1903 venuto venire,
1942 aggiunto aggiungere. An override row keeps one noun and reads every verb form it shares as the
noun; the owner may name rows, each with its reason, at any reduction. Two of them came with the
names rule's narrowing (the review of 2026-10-23) and are left for change 17's refinements: `dato`
reads as *dare* (MarkIT's counts, 9 *dare* against 3 *dato*) and `dati` with it (« i dati »), and
`lega` as *legare* (one count), whose `leghe` is then no form.

## Licences

The English Wiktionary's content (via kaikki.org): CC BY-SA 4.0 and GFDL. wordfreq's data: CC BY-SA
4.0. UD Italian-MarkIT: CC BY 4.0. Morph-it!: CC BY-SA 2.0 (dual with the LGPL), read under CC BY-SA
2.0. The NOTICE names each licence's address. UD Italian-PUD (CC BY-SA 3.0) is measured against and
never shipped. The tables are published under CC BY-SA 4.0 with the pack.
