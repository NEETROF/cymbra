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
