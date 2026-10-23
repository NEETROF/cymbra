# Design

## Context

See proposal.md — Why. What constrains the shape:

- `detect(trimmed, languages)` returns whichlang's answer when it is one of the candidates, unless
  that language's guard refuses the block — already per candidate: none for English,
  `iberian_neighbour` for Spanish (Catalan, Galician, Occitan: three comparisons, each neighbour's
  count against Spanish's markers that neighbour does not write), `romance_neighbour` for French
  (one pooled table of 152 Catalan, Occitan and Romanian markers against 69 French ones).
  `guard_words` reads a block as runs of letters and combining marks joined across `-`, U+00B7,
  U+2011 and `.` between letters; a run holding a digit anywhere is dropped whole; an apostrophe,
  a space or any other character ends a run. `romance_neighbour` composes a decomposed word (NFC)
  before it looks it up, counts a neighbour marker in lowercase only, and refuses the block when
  the neighbours' count exceeds French's. The gate and the vote call the same function.
- whichlang 0.1.1's classes: Arabic, Mandarin, German, English, French, Hindi, Italian, Japanese,
  Korean, Dutch, Portuguese, Russian, Spanish, Swedish, Turkish, Vietnamese. A Catalan sentence is
  recorded as read as Italian in the French baseline (`pages-fr.txt`'s `mixte` page, block 3) and a
  Galician one in a `language.rs` unit test. **Probed with whichlang 0.1.1 itself** (a scratch
  binary over the registry crate, 2026-10-22), one or two sentences per neighbour:

  | Neighbour | Read as Italian | Sentence |
  |---|---|---|
  | Catalan | yes | « El far s'alçava a la vora del penya-segat, i ningú no recordava qui l'havia construït. » |
  | Galician | yes | « Non hai unha solución sinxela, pero o concello xa traballa nela. » |
  | Venetian | yes | « Ghe xe tanta zente in piassa e tuti i parla. » (« El canal xe pien de barche… » → Spanish) |
  | Sicilian | yes | « Lu patruni dici ca nun ci su' sordi, ma iddu campa bonu. » |
  | Neapolitan | yes (4 of 6) | « Tengo 'na casa piccerella vicino 'o mare, e 'a sera guardo 'e stelle. » (« 'O sole mio… » → Portuguese) |
  | Sardinian | yes | « Sa domo est manna e sos pitzinnos giogant in sa pratza. » |
  | Corsican | yes | « A casa mea hè vicinu à a chjesa, è u paese hè chjucu. » |
  | Friulian | yes (2 of 3) | « Cheste e je la cjase dai miei nonos, e jo o soi nassût culì. » |
  | Latin | 1 of 12 | « Gallia est omnis divisa… » → French; « Pater noster… » → English; « Ubi concordia, ibi victoria… » → Italian |

  Every scenario sentence of the spec is one of these, checked to be read as Italian first, as
  change 42 checked its own.
- Change 7 puts Italian at `1.0.0`; the Italian corpus's `misto` page holds English, Spanish,
  French, Catalan, Galician and Latin blocks around Italian ones — change 5 authored the Catalan
  and Galician blocks to read as Italian; the Latin one lands under its nearest class.
- The regional languages of Italy share much of Italian's surface — `la`, `di`, `in`, `che`, `no`,
  `e`, `a`, `non` — and differ in their articles, copulas and negations: Sicilian `lu`, `nun`,
  `sugnu`, `chistu`, `iddu`; Neapolitan `'o`, `'a`, `'e`, `'nu`, `'na`, `nun`, `chesto`, `songo`;
  Venetian `el`, `xe`, `ghe`, `ga`; Sardinian `sos`, `sas`, `custu`, `cun`, `totu`; Friulian `lis`,
  `jè`, `cun`, `ancje`, `dut`, `cheste`, `jo`, `soi`; Corsican `hè`, `sò`, `ùn`, `chì`, `u`,
  `quessu`. Galician shares `non`, `e`, `a`, `da` and differs in `os`, `as`, `unha`, `tamén`, `xa`,
  `moi`, `nela`. Catalan differs in `els`, `dels`, `amb`, `és`, `són`, `havia`, `ningú`.

## Goals / Non-Goals

**Goals:**

- A page in a neighbour language is not counted, glossed or written into a reader's Italian
  statuses; a quotation in one, inside an Italian page, is left out of the analysis.
- Italian recall stays whole, by a gate: a short line cannot be judged, and Italian comes first.
- Nothing of English, Spanish or French moves.

**Non-Goals:**

- A detector class for the neighbours (a dependency and a model in the WASM bundle, for a question
  a few hundred function words answer — change 42's rejection holds).
- Guarding Spanish or French against the regional languages of Italy: whichlang's Spanish and
  French classes are not where they usually land; measured, and left as measured.
- Dialectal Italian written in Italian spelling (regional vocabulary in Italian sentences): it is
  Italian.

## Decisions

### D1 — Function words, counted, compared: French's rule, neighbours pooled

For a block whichlang reads as Italian, count the words of two tables (D3): neighbour markers,
and Italian markers. The block is refused as Italian when neighbour markers outnumber Italian ones;
a tie, or no marker, keeps it Italian. One pooled neighbour table, change 42's choice for French:
the guard asks « Italian or not », not which neighbour, and the regional languages overlap too much
among themselves for per-neighbour comparisons to mean anything (`nun` is Sicilian and Neapolitan,
`cun` Sardinian and Friulian, `el` Venetian and Catalan). Task 1.1 re-measures per neighbour, as
change 42's D1 measured per-language tables before pooling, and records the figures.

### D2 — The neighbours: kept by measurement, defaults the implementation follows

Candidates, each measured on its Wikipedia (a sample of articles, blocks of twelve bytes or more)
for the share whichlang reads as Italian with and without the guard:

| Neighbour | Why a candidate | Default |
|---|---|---|
| Catalan, Galician | read as Italian today (recorded, probed) | kept |
| Venetian, Sicilian, Neapolitan, Sardinian, Corsican, Friulian | Wikipedias of their own, no class, Italian pages quote them; each read as Italian on its probe sentence | kept |
| Latin | quotations in Italian pages, no class; 1 probe of 12 read as Italian | left out |
| Occitan, Romanian | French's and Spanish's guards hold them; a share may land on Italian | left out |

The rule: a neighbour is kept when at least one tenth of its sampled blocks is read as Italian
(change 42 kept Romanian at 23.5 % and left Latin out for French at 18 % because it shares `et`
and `est`); a neighbour below it adds no words to the table (its words would only cost Italian
recall). The defaults above hold until the measurement says otherwise, so the implementation
never waits on a question; the owner reads the figures (task 4.1). Spanish, Portuguese and French
have classes: a block in them is rarely read as Italian by whichlang (change 42 measured 0.2 % of
Italian blocks read as French, the reverse is of the same order), and the guard never sees it.

### D3 — The tables, measured word by word

Closed-class words — articles, pronouns, prepositions, conjunctions, adverbs, the copula, the
auxiliaries, the negation — kept only when the measurement, word by word, shows they catch
neighbour text without refusing Italian, and **none that is an Italian word in any register or
region**, whatever the neighbour writes it for. Candidates (the measurement decides; every word
left out is named in the code with its Italian reading):

- **Neighbour markers**: Catalan's, seeded from Spanish's `CATALAN_MARKERS` less the Italian words
  `per`, `però`, `sempre` (among them `dels`, `pels`, `els`, `amb`, `és`, `són`, `havia`, `eren`, `fou`, `després`, `fins`, `llavors`,
  `segons`, `seva`, `seus`, `també`, `aquest`, `això`, `ningú`), Galician's (`os`, `as`, `unha`,
  `tamén`, `xa`, `moi`, `nela`, `pola`, `coas`), Sicilian's (`lu`, `nun`, `cu`, `chistu`, `chiddu`,
  `sugnu`, `avi`, `comu`, `unni`, `pirchì`, `iddu`, `idda`), Neapolitan's (`'o`, `'a`, `'e`, `'nu`,
  `'na`, `pe`, `chesto`, `chello`, `comme`, `songo`, `tene`, `addó`, `nisciuno`), Venetian's (`el`,
  `xe`, `ze`, `ghe`, `ga`, `cussì`, `gà`, `tuti`, `zente`), Sardinian's (`sos`, `sas`, `cun`,
  `dae`, `custu`, `cussu`, `tenet`, `totu`, `at`), Friulian's (`lis`, `jè`, `cun`, `cheste`,
  `chest`, `ancje`, `dut`, `jo`, `soi`, `je` — the last French's *I*, which an Italian block quoting
  « je ne sais quoi » would count: measured, as change 42b measured `ne`), Corsican's (`hè`, `sò`, `ùn`, `chì`, `quessu`,
  `avà`, `tandu`, `u`); Latin's (`sunt`, `quae`, `quod`, `sed`, `cum`, `atque`, `enim`, `autem`,
  `ut`, `nec`, `igitur`, `ubi`, `ibi`) only if Latin is kept (D2).
  **Left out as Italian words**: `però` (Catalan), `hai` (*you have*; Galician), `polo` (« polo
  nord »), `cos` (« cos'è » reads `cos` + `è`), `pero` (the pear tree), `o` (*or*; Galician's article),
  `qui` (*here*; Latin), `ad` (« ad esempio »), `est` (*east*; Latin's and Sardinian's copula),
  `esse` (*they*), `li` (the clitic; Sicilian's article), `sto` (« sto bene »; Venetian), `anca`
  (*hip*), `par` (« par condicio »; Venetian's and Friulian's), `pro` (« pro e contro »; Sardinian),
  `sa` (*he knows*; Sardinian's article), `so` (*I know*; Venetian's), `pi` (the letter), `co`
  (kept: `co-fondatore` joins across its hyphen and never gives `co`), and every word Italian
  writes (`e`, `a`, `in`, `non`, `no`, `del`, `al`, `su`, `la`, `di`, `che`, `ma`, `da`, `si`).
- **Italian markers**: Italian function words the neighbours rarely write — `gli`, `degli`,
  `delle`, `della`, `dello`, `nella`, `nello`, `negli`, `nelle`, `dalla`, `dalle`, `sono`, `siamo`,
  `hanno`, `abbiamo`, `anche`, `perché`, `questo`, `questa`, `quello`, `quella`, `molto`, `essere`,
  `avere`, `nel`, `dal`, `loro`, `questi` (`nel` and `dal` measured against Venetian). **Left out as
  the neighbours' words**: `del` (Catalan's, Occitan's and Venetian's — the code records it among the
  seven both Catalan and Occitan write), `il` and `la` (Friulian's articles), `i` (Catalan's *and*), `sempre` (Sicilian's), `ci` and `vi` (Sicilian's
  and Corsican's clitics), `lo` (Occitan's and western Catalan's article — on Italian's side it would
  make an Occitan block vote Italian, the defect change 42b fixed for Spanish), `è` (Corsican's
  *and*), `sul` (Venetian's), `una` (Sardinian's), `miei` (Friulian's), `che` (every neighbour's),
  and `c'è`, unreachable (the apostrophe ends the word).

Both tables sorted and disjoint, every entry at most `LONGEST_MARKER_BYTES`, held by the tests change
42 has (`language.rs`'s sorted/disjoint/length test), with a doc line per entry naming the
neighbour; the words left out named too, with the register or region that writes them in Italian.

### D4 — How the guard reads a block

As French's `guard_words` reads it (D4 of change 42): a run of letters and combining marks joined
across `-`, U+00B7, U+2011 and `.` between letters, a run holding a digit dropped whole
(`co-fondatore` is one word and never gives `co`; `2a` is no word), a decomposed word composed
before it is looked up, a neighbour marker counted in lowercase only, the apostrophe ending a word
(an Italian elided piece `l'`, `dell'` ends at its apostrophe and is no marker on its own). One
Italian addition: a word opened by an apostrophe (U+0027 or U+2019, which typeset Neapolitan
writes) — the Neapolitan elided articles `'o`, `'a`, `'e`, `'nu`, `'na` — is looked up with its
apostrophe, as Spanish's guard looks up Occitan's `qu'`, `m'`, `t'`. It lives in a second entry
point, `guard_words_opened(text, f)`, which hands Italian's callback the word and whether an
apostrophe opened it; `guard_words` and French's and Spanish's readings are byte for byte what they
were, held by their existing tests and by a new case of `the_french_guard_reads_words_as_its_doc_says`
with an opening single quote (« 'et' » still reads `et`).

### D5 — One detection function; a guard runs only for the language it protects

`detect` gains an Italian arm: `italic_neighbour` beside `iberian_neighbour` and
`romance_neighbour`, called only when whichlang's answer is Italian and Italian is a candidate.
English has no guard; Spanish's and French's arms are untouched; the six baselines prove it (D8).

The guard's cost is measured as change 42 measured French's: on the Italian corpora's blocks,
native and WebAssembly, in the shape French's guard has (two tables, a scan by character), and
reported in the pull request.

### D6 — The recall gate

Measured on the Italian corpora — UD Italian-PUD, MarkIT, Tatoeba's Italian export, a sample of
Italian Wikipedia articles, the full texts of the corpus's two public-domain excerpts (Manzoni's
*I promessi sposi*, Collodi's *Pinocchio*, from Liber Liber or Project Gutenberg) — and on the
baseline's `pages-it.txt`: at most one Italian block in a thousand refused across the corpora, and
none of `pages-it.txt`'s; every refused block listed in the pull request with the markers that
refused it and its reason (a quotation in a neighbour language, a name, a genuine misfire). A word
is kept when it catches neighbour text and refuses no Italian block that is not a quotation or a
name — change 42 kept 26 refusals of 42,961 French blocks with reasons, change 42b words refusing
one or two Spanish blocks; a zero-tolerance rule would drop `el` at the first Italian block quoting
Spanish.

### D7 — Italian's analyser version: `1.0.0` → `1.1.0`

A block the guard refuses is one the analysis no longer counts; the version moves. On top of
change 7, not before it; whichever of this change and change 10 lands second takes the next minor.
The fixture's manifest and the extension's test pattern follow.

### D8 — What the golden shows; what cannot move

The re-bless of `it-fr.golden` is the `misto` page's: its Catalan and Galician blocks leave the
analysis — its Latin block too only if Latin is kept (D2) and the golden shows whichlang read it as
Italian; otherwise it stays as it is —; every other page is byte for byte. Change 7's scenario « What the baseline shows
today » says the Catalan and Galician blocks are analysed as Italian: this change MODIFIES that
requirement (change 7 is in `archiveAfter`) so the scenario says they are excluded. The six
goldens, S0, every pin and the two other guards are untouched.

### D9 — Tests

Unit tests in `language.rs`: each scenario's sentence, asserted read as Italian by whichlang first
(`read_as_italian`, as `read_as_spanish` and `read_as_french` do) and then refused or kept; one
sentence per kept neighbour; every kept marker counted once in a block that holds it; the tables
sorted, disjoint and within `LONGEST_MARKER_BYTES`; the Italian-naming sentence kept; capitals; the
apostrophe-opened word, straight and typographic; an NFD block; the leak the guard leaves; the
French and Spanish guards' tests unchanged, French's gaining the opening-quote case. The
baseline's `misto` assertion names the blocks kept.

## Measurement

Fetched on 2026-10-23 and pinned in `scripts/lingua-data/measure/it-guard-corpus.json`: each
Wikipedia article by its revision (its wikitext's sha1 and sha256), every other file by URL and
sha256, UD by commit. `it_guard.py` (run by `it-guard.sh`) fetches them, each source into a
directory of its own, and then works offline; neither the samples nor the figures are committed.
lingua-core's own gate reads every block (the `italian_guard_measure` example); a port of the
guard's reading explains each refusal and agrees with the gate on all 50,219 blocks read as
Italian. A block is a paragraph (Wikipedia wikitext rendered to text: templates, tables,
references, files and categories out, the text of `lang` and quotation templates kept; Project
Gutenberg paragraphs), a sentence (UD, Tatoeba) or a line (`pages-it.txt`), without soft hyphens,
trimmed, of 12 bytes or more.

| Source | What | Licence |
|---|---|---|
| Wikipedia, eleven neighbours | ca, gl, vec, scn, nap, sc, co, fur, la, oc, ro: the articles 75 English seed titles link to (56–75 per wiki, 753 in all) | CC BY-SA 4.0 |
| UD Italian-PUD | test, `71d932aa` | CC BY-SA 3.0 |
| UD Italian-MarkIT | train, dev and test, `e210079d` | CC BY 4.0 |
| Tatoeba | the Italian export of 2026-10-17, 20,000 sentences (seed 42, French's choice) | CC BY 2.0 FR |
| Italian Wikipedia | 64 articles: the regional languages and their places, songs and authors that quote them (*'O sole mio*, Eduardo, Camilleri…), Spanish and Catalan names, everyday subjects | CC BY-SA 4.0 |
| Project Gutenberg | Manzoni, *I promessi sposi* (#45334); Collodi, *Pinocchio* (#52484) | public domain |
| `pages-it.txt` | its 65 Italian blocks (the `misto` page's six foreign ones aside) | — |

**What "better" means, written before any figure.** Rule 1 (D2): a neighbour is kept when whichlang
reads a tenth of its Wikipedia blocks as Italian or more. Rule 2 (D3): no candidate that is a
current Italian word or written form, in any register or region, elided and truncated pieces
included. Rule 3 (D3, D6): a neighbour word is kept when it counts in a kept neighbour's block read
as Italian, and every Italian block it helps refuse is, read by hand, a quotation or a name — a
misfire drops every neighbour word counting in it. Rule 4 (D3): an Italian word is kept when every
kept neighbour writes it at most a tenth as often, per thousand words, as the Italian corpora do.
Rule 5 (D6): the recall gate, at most one refused block in a thousand of the Italian corpora's
blocks read as Italian and none of `pages-it.txt`'s; while it fails, the neighbour word counting in
the most refused blocks goes (tie: the one catching fewer neighbour blocks). The script implements
each rule (`it_guard.py words`).

**Amended before any table was settled, each recorded with its reason:**
- *Catalan and Galician are kept against rule 1* (9.7 % and 3.2 % of their blocks read as Italian):
  the requirement is named for them, two scenarios refuse their sentences, and the baseline's
  scenario excludes the `misto` page's blocks. Rule 3's first half does not apply to their words —
  their Wikipedia samples are seldom read as Italian, so "catching the sample" cannot test words
  kept for the sentences the spec refuses; rule 3's second half and the gate still do.
- *Rule 2 applied systematically*: every candidate checked against an Italian forms list (change
  9's prototype tables) and `tokenize.rs`'s elided forms and truncations, after the first list of
  refusals showed `quand` (« quand'è ») and `entrò` missed by hand.
- *Rule 4 on the neighbours' own sentences*: as first written, it counted Italian quoted in the
  neighbours' articles (bibliographies, an Italian law's title in a Catalan article) as their
  writing, and took 19 of 28 words out. Measured on the blocks that hold one of the neighbour's own
  words, then each flag read: `dal` (Venetian, Friulian, Occitan), `essere` and `hanno`
  (Neapolitan), `quella` and `questi` (Corsican) are their own words; every other flag was an
  Italian name, title or quotation (« Università degli studi di Udine », « Tu scendi dalle
  stelle », « Gallicano nel Lazio »).

**Rule 1, and the neighbours with the guard** (share of blocks read as Italian; of the text in
brackets):

| Neighbour | Blocks | Without | With | D2's default | Kept |
|---|---:|---:|---:|---|---|
| Catalan | 9,325 | 9.7 % (1.7) | 9.0 % (1.3) | kept | yes, by the spec |
| Galician | 6,132 | 3.2 % (0.4) | 3.2 % (0.4) | kept | yes, by the spec |
| Venetian | 1,595 | 58.9 % (77.1) | 28.9 % (14.1) | kept | yes |
| Sicilian | 1,557 | 82.9 % (95.4) | 62.6 % (37.5) | kept | yes |
| Neapolitan | 760 | 71.3 % (86.0) | 36.8 % (11.5) | kept | yes |
| Sardinian | 1,437 | 37.6 % (33.5) | 26.1 % (6.1) | kept | yes |
| Corsican | 1,514 | 83.2 % (96.9) | 53.3 % (29.9) | kept | yes |
| Friulian | 1,582 | 37.3 % (31.5) | 31.0 % (16.5) | kept | yes |
| Latin | 5,233 | 23.1 % (13.9) | 20.3 % (6.8) | out | yes, by rule 1 |
| Occitan | 4,723 | 14.6 % (3.2) | 12.6 % (1.6) | out | yes, by rule 1 |
| Romanian | 6,655 | 31.9 % (30.5) | 9.3 % (1.0) | out | yes, by rule 1 |
| Pooled | 40,513 | 25.4 % (18.8) | 16.5 % (4.2) | | |

Catalan's and Galician's blocks read as Italian are mostly short — headings, names, Italian
titles in their bibliographies (22 % of Catalan's blocks under 40 bytes, 1.5 % from 100); long
Catalan and Galician prose is read as Spanish, French or Portuguese. What leaks elsewhere is short
lines without a marker of either table, sentence-initial capitals (« Lu patruni… »), and
Italian quoted in the neighbours' own articles.

**The words.** Of 258 candidates (D3's, and the Occitan and Romanian tables of Spanish's and
French's guards once rule 1 kept them), rule 2 takes 19 out, rule 3 66 that no neighbour block
read as Italian holds (Occitan's mostly; Catalan's and Galician's stay, 14 of them), and one
misfire: `u`, Corsican's article, is the letter's Italian name in 13 refused blocks about spelling
(« Maschile singolare finito in consonante o in u: »). The gate then failed — 73 blocks of
39,934, every one a quotation or a name — and rule 5 took `iddu` (17 refused blocks; 13
neighbour blocks caught), `lu` (9; 368), `chì` (7; 288), `el` (6; 495) and `'e` (5; 307),
leaving 167 neighbour words and 23 Italian ones.

**Italian refused.** 37 of 39,934 blocks read as Italian (0.093 %), all in Italian Wikipedia's
articles on the neighbour languages; none in UD (992 and 1,289 read as Italian), Tatoeba (19,699),
the two books (3,044 and 1,851) or `pages-it.txt` (65). Thirty are quotations — Sardinian,
Corsican, Venetian, Sicilian and Catalan paradigms and phrasebook lines, Neapolitan lyrics, the
Sicilian School's « Per la vertute de la calamita », a romanesco « je volevamo solà er
portafoglio », a Romanian line, French « pas dosé » —, seven are names and titles (Eduardo's « È
arrivato 'o trentuno », « Expositio seu comentum super Comedia », « Grafia dell'Escòla dau Pò »).
No misfire is left. The pull request lists all 37.

**Cost** (D5), on the 39,934 Italian blocks read as Italian (6.3 MB; Apple M2 Max, release build;
WebAssembly under Node 22), best of seven: the guard is the difference between the gate for
Italian and the gate for English, which reads the block the same way and has no guard.

| µs per KB | whichlang | Italian's guard | French's, for scale (change 42) |
|---|---|---|---|
| native | 17.0–17.2 | 16.0–16.5 (94–97 %) | 16.5–21.1 (100–123 %) |
| WebAssembly | 14.7–15.0 | 18.7–23.7 (126–158 %) | 19.4–23.8 (138–159 %) |

A 20 KB Italian article costs about 0.4 ms more in WebAssembly; a reader of English, Spanish or
French pays nothing.

**For the owner (task 4.1)** — the same samples, other choices:

| Variant | Neighbour words | Pooled neighbours read as Italian with the guard | Italian refused |
|---|---:|---|---:|
| Implemented: rule 1 with Catalan and Galician, rule 5 applied | 167 | 16.5 % (4.2) | 37 (0.093 %) |
| Without rule 5's five words (`iddu`, `lu`, `chì`, `el`, `'e`) | 172 | Sicilian 54.3, Neapolitan 27.6, Corsican 47.2, Venetian 23.4 % | 73 (0.183 %) — fails the gate |
| D2's defaults (no Latin, Occitan or Romanian word) | 86 | Romanian 24.7 % (16.0), Latin 22.8 %, Occitan 14.1 % | 32 (0.080 %) |

### Deviations from the decisions above

- **D2**: Latin, Occitan and Romanian are kept, by D2's own rule; Catalan and Galician are kept
  against it, because the spec names them. The owner confirms or moves both (task 4.1).
- **D3, neighbour markers**: `u` (Corsican) is left out as the letter's Italian name, and
  `iddu`, `lu`, `chì`, `el` and `'e` for the recall gate. Rule 2 leaves out, beyond D3's list,
  `molt`, `tot`, `avi`, `pe`, `pola`, `unni`, and Occitan's and Romanian's Italian words (`ma`,
  `sul`, `qual`, `sia`, `lor`, `quand`, `entrò`, `ambe`, `care`, `dar`, `fi`, `din`). D3's `co`
  is in no neighbour's list and « co' » (*coi*) is an Italian truncation: not added.
- **D3, Italian markers**: `dal`, `essere`, `hanno`, `quella` and `questi` are left out as
  measured (`dal` against Venetian, as D3 anticipated, and Friulian).
- **D5**: the guard's cost is the gate's difference, Italian against English: `italic_neighbour`
  is private, and the example uses the crate's public gate.
- **The spec**: the Neapolitan scenario counts `'na`, `'o` and `'a` (`'e` is left out), and the
  requirement names Latin, Occitan and Romanian among the neighbours, with a scenario for them.

## Risks / Trade-offs

- [Italian recall] → D6's gate; every Italian block refused is named in the pull request.
- [A neighbour whichlang files elsewhere] → D2's rule; a neighbour below a tenth adds no word.
- [Dialect quoted inside Italian] → a block, not a document, is refused: an Italian page keeps its
  Italian blocks.
- [Words shared with the neighbours] → never markers on either side (D3).
- [A word opened by a quote] → only Italian's reading sees it, and only the five Neapolitan
  articles are markers; « 'o » quoted in Italian is one word in a block Italian's markers carry.

## Migration Plan

Nothing deploys and nothing is released; no listed pair studies Italian. Rollback is a revert
with the golden's.
