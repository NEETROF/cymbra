# Design

## Context

See proposal.md — Why. What constrains the shape:

- `detect(trimmed, languages)` returns whichlang's answer when it is one of the candidates, unless
  that language's guard refuses the block: none for English, `iberian_neighbour` for Spanish
  (Catalan, Galician, Occitan: three comparisons, each neighbour's count against Spanish's markers
  that neighbour does not write), `romance_neighbour` for French (one pooled table of 152 Catalan,
  Occitan and Romanian markers against 69 French ones; `guard_words` reads a block as letters and
  combining marks joined across `-`, U+00B7, U+2011 and `.` between letters, composed, a digit
  ending a word, a neighbour marker counted in lowercase only). The gate and the vote call the
  same function.
- whichlang 0.1.1's classes: Arabic, Mandarin, German, English, French, Hindi, Italian, Japanese,
  Korean, Dutch, Portuguese, Russian, Spanish, Swedish, Turkish, Vietnamese. A Catalan sentence and
  a Galician one are recorded as read as Italian (`french_baseline.rs`, `language.rs`'s Galician
  probe); Latin's, Corsican's, Sardinian's, Sicilian's, Neapolitan's, Venetian's and Friulian's
  classes are unmeasured.
- Change 7 puts Italian at `1.0.0`; the Italian corpus's `misto` page holds English, Spanish,
  French, Catalan, Galician and Latin blocks around Italian ones, analysed as Italian where
  whichlang says so.
- The regional languages of Italy share much of Italian's surface — `la`, `di`, `in`, `che`, `no`
  — and differ in their articles, copulas and negations: Sicilian `lu`, `nun`, `sugnu`, `chistu`;
  Neapolitan `'o`, `'a`, `nun`, `chesto`, `songo`; Venetian `el`, `xe`, `ghe`, `par`; Sardinian
  `su`, `sos`, `sas`, `est`, `custu`, `cun`; Friulian `lis`, `jè`, `cun`, `ancje`, `dut`;
  Corsican `hè`, `sò`, `ùn`, `chì`, `quessu`. Latin shares `et`, `in`, `non` and differs in
  `est`, `sunt`, `qui`, `quae`, `quod`, `sed`, `cum`, `atque`, `enim`, `autem`. Galician shares
  `non`, `e`, `a`, `da` and differs in `os`, `as`, `unha`, `tamén`, `xa`, `moi`, `hai`, `polo`.

## Goals / Non-Goals

**Goals:**

- A page in a neighbour language is not counted, glossed or written into a reader's Italian
  statuses; a quotation in one, inside an Italian page, is left out of the analysis.
- Italian recall stays whole: a short line cannot be judged, and Italian comes first.
- Nothing of English, Spanish or French moves.

**Non-Goals:**

- A detector class for the neighbours (a dependency and a model in the WASM bundle, for a question
  a few hundred function words answer — change 42's rejection holds).
- Guarding Spanish or French against the regional languages of Italy: whichlang's Spanish and
  French classes are not where they land; measured, and left as measured.
- Dialectal Italian written in Italian spelling (regional vocabulary in Italian sentences): it is
  Italian.

## Decisions

### D1 — Function words, counted, compared: French's rule, neighbours pooled

For a block whichlang reads as Italian, count the words of two tables (D3): neighbour markers,
and Italian markers. The block is refused as Italian when neighbour markers outnumber Italian ones;
a tie, or no marker, keeps it Italian. One pooled neighbour table: the guard asks « Italian or
not », not which neighbour, and the regional languages overlap too much among themselves for
per-neighbour comparisons to mean anything (`nun` is Sicilian and Neapolitan, `cun` Sardinian and
Friulian, `el` Venetian and Catalan).

### D2 — The neighbours: kept by measurement

Candidates, each measured on its Wikipedia (a sample of articles, blocks of twelve bytes or more)
for the share whichlang reads as Italian with and without the guard:

| Neighbour | Why a candidate | Kept if |
|---|---|---|
| Catalan, Galician | read as Italian today (recorded) | measured share above a few per cent |
| Latin | quotations in Italian pages, no class | same |
| Sicilian, Neapolitan, Venetian, Sardinian, Friulian, Corsican | Wikipedias of their own, no class, Italian pages quote them | same |
| Occitan, Romanian | read as French or Spanish mostly; a share may land on Italian | same |

A neighbour below the threshold adds no words to the table (its words would only cost Italian
recall). Spanish, Portuguese and French have classes: a block in them is never read as Italian by
whichlang, and the guard never sees it.

### D3 — The tables, measured word by word

Closed-class words — articles, pronouns, prepositions, conjunctions, adverbs, the copula, the
auxiliaries, the negation — kept only when the measurement, word by word, shows they catch
neighbour text without refusing Italian, and none as an Italian word in any register or region:

- **Neighbour markers**: Catalan's (`amb`, `els`, `és`, `són`, `molt`, `més`, `també`, `però`,
  `aquest`, `això`), Galician's (`os`, `as`, `unha`,
  `tamén`, `xa`, `moi`, `hai`, `polo`, `pola`, `cos`, `coas`), Latin's (`est`, `sunt`, `qui`,
  `quae`, `quod`, `sed`, `cum`, `atque`, `enim`, `autem`, `ut`, `nec`, `igitur`, `esse`, `ad`),
  Sicilian's (`lu`, `li`, `nun`, `cu`, `pi`, `chistu`, `chiddu`, `sugnu`, `avi`, `comu`, `unni`,
  `pirchì`), Neapolitan's (`'o`, `'a`, `'e`, `'nu`, `'na`, `pe`, `chesto`, `chello`, `comme`,
  `songo`, `tene`, `addó`), Venetian's (`el`, `xe`, `ze`, `ghe`, `par`, `co`, `anca`, `cussì`,
  `gà`, `sto`), Sardinian's (`sos`, `sas`, `est`, `cun`, `dae`, `pro`, `custu`, `cussu`, `tenet`,
  `at`), Friulian's (`lis`, `jè`, `cun`, `par`, `cheste`, `chest`, `ancje`, `dut`), Corsican's
  (`hè`, `sò`, `ùn`, `chì`, `quessu`, `avà`, `tandu`) — each candidate measured, the words Italian
  writes (`e`, `a`, `in`, `non`, `no`, `del`, `al`, `su`, `sa`, `la`, `di`, `che`) left out
  whatever a neighbour writes them for;
- **Italian markers**: Italian function words the neighbours rarely write — `il`, `lo`, `gli`,
  `della`, `dello`, `degli`, `delle`, `nella`, `nello`, `negli`, `nelle`, `sono`, `siamo`,
  `anche`, `perché`, `questo`, `questa`, `quello`, `molto`, `sempre`, `nel`, `sul`, `dal`,
  `anche`, `essere`, `avere`, `hanno`, `abbiamo`, `c'è`, `ci`, `vi` — measured out where a
  neighbour writes them (`sempre` is Sicilian too; `il` and `la` Friulian).

Both tables sorted, with a doc line per entry naming the neighbour; the words left out named too,
with the register or region that writes them in Italian.

### D4 — How the guard reads a block

As French's `guard_words` reads it (D4 of change 42), with one Italian addition: an apostrophe
ends a word, and a word that begins with an apostrophe — the Neapolitan elided articles `'o`, `'a`,
`'e`, `'nu`, `'na` — is looked up with it, as Spanish's guard looks up Occitan's `qu'`, `m'`, `t'`.
An Italian elided piece (`l'`, `dell'`) ends at its apostrophe and is not a marker on its own.
Measured against the alternatives: splitting on every non-letter (`e-mail` gives `e`, not a
marker here, but `2a` gives `a`); counting capitalised neighbour words (Catalan and Sicilian
titles open on `Lo`, `El`, `Lu`); combining marks as separators (the `tecnico` page's NFD block).

### D5 — One detection function; a guard runs only for the language it protects

`detect` gains an Italian arm: `italic_neighbour` beside `iberian_neighbour` and
`romance_neighbour`, called only when whichlang's answer is Italian and Italian is a candidate.
English's, Spanish's and French's arms are untouched; the six baselines prove it (D7).

The guard's cost is measured as change 42 measured French's: on the Italian corpora's blocks,
native and WebAssembly, in the shape French's guard has (two tables, a scan by character), and
reported in the pull request.

### D6 — Italian's analyser version: `1.0.0` → `1.1.0`

A block the guard refuses is one the analysis no longer counts; the version moves. On top of
change 7, not before it. The fixture's manifest and the extension's constant map follow.

### D7 — What the golden shows; what cannot move

The re-bless of `it-fr.golden` is the `misto` page's: its Catalan, Galician and Latin blocks leave
the analysis; every other page is byte for byte. The six goldens, S0, every pin and the three
other guards are untouched.

### D8 — Tests

Unit tests in `language.rs`: each scenario's paragraph; one sentence per kept neighbour; every
kept marker counted once in a block that holds it; the Italian-naming sentence kept; capitals;
the Neapolitan apostrophe; an NFD block; the leak the guard leaves; the French and Spanish guards'
tests unchanged. The baseline's `misto` assertion names the blocks kept.

## Risks / Trade-offs

- [Italian recall] → measured on UD Italian-PUD, MarkIT, Tatoeba, Wikipedia and the public-domain
  books; every Italian block refused is named in the pull request; a word that refuses one is
  dropped.
- [A neighbour whichlang files elsewhere] → measured; a neighbour not read as Italian adds no
  word.
- [Dialect quoted inside Italian] → a block, not a document, is refused: an Italian page keeps its
  Italian blocks.
- [`e`, `a`, `in`, `non`, `del` shared with the neighbours] → never markers, on either side.

## Migration Plan

Nothing deploys and nothing is released; no listed pair studies Italian. Rollback is a revert
with the golden's.

## Open Questions

- Which of Occitan and Romanian, measured, are worth their words (the owner, on the figures,
  task 4.1).
