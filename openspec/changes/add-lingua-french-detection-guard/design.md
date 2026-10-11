# Design — add-lingua-french-detection-guard

## Context

`analysis/language.rs` on `main`:
- `block_is_studied(text, studied)` keeps a block of 12 bytes or more when `detect(trimmed)` is the
  studied language's whichlang class;
- `detect_document_language(blocks, candidates, hint)` weighs each such block, by its length, for
  the candidate `detect` names;
- `detect(trimmed)` is `whichlang::detect_language` with Spanish's guard: a block read as Spanish
  whose Catalan or Galician markers outnumber its Spanish ones comes back as Portuguese, a language
  nobody studies (`add-lingua-spanish-detection-guard`, `iberian_neighbour`).

French has been a studied language since change 39 (`StudiedLanguage::French`, whichlang's `Fra`,
analyser `0.1.0`). whichlang 0.1.1 has sixteen classes (Ara, Cmn, Deu, Eng, Fra, Hin, Ita, Jpn, Kor,
Nld, Por, Rus, Spa, Swe, Tur, Vie): Catalan, Occitan, Romanian, Franco-Provençal, Picard, Walloon,
Latin and Haitian Creole have none, and land on the nearest class. The French corpus
(`crates/lingua-wasm/tests/baseline/pages-fr.txt`) holds a `mixte` page for this change: English,
Spanish, Catalan, Occitan and Italian blocks around two French ones. Today whichlang reads its
Catalan block as Italian and its Occitan block as French, and `fr-en.golden` analyses that Occitan
block's eighteen tokens as French.

Nothing ships French before change 52: no listed pair studies it, so `detect` never meets French as
a candidate in a released build. Change 40 (`add-lingua-french-tokenisation`, `0.2.0`) is
proposed; change 41 (`add-lingua-french-analysis`, `1.0.0` by change 39's D4) is not yet. Neither
touches detection.

## Goals / Non-Goals

**Goals:**
- Catalan, Occitan and Romanian blocks no longer analysed, counted or voted for as French, measured.
- French kept: no measurable false refusal on real French — short blocks, proper-noun-heavy text,
  Québécois, Belgian and Swiss French.
- English and Spanish detection unchanged, and en-fr, es-fr, es-en and en-es output byte for byte.
- A cost a reader of French can ignore, and none for a reader who does not study French.

**Non-Goals:**
- Recognising Catalan, Occitan or Romanian as languages of their own: no reader studies them.
- Franco-Provençal, Picard, Walloon, Latin, Haitian Creole (D2, measured, left as they are).
- Spanish's leak on Occitan (19.2 % of Occitan text read as Spanish): a Spanish change, since
  Spanish detection does not move here (open question 2).
- Routing per block: a page mixing French and Occitan keeps its French blocks and excludes the
  others, as any other language's.

## Measurement

Fetched on 2026-10-09 into the scratchpad and kept out of the repository, as the Spanish guard's
Wikipedia extracts were; the fixtures committed are sentences written for the change. Blocks are
the samples' sentences or paragraphs, trimmed, of 12 bytes or more (`MIN_BLOCK_BYTES`); 148,267 in
all.

| Source | What | Licence |
|---|---|---|
| Universal Dependencies (`# text` lines, `master`) | French GSD (test, dev), Sequoia (test, dev), PUD, FQB (questions), ParisStories and Rhapsodie (spoken), ParTUT; Catalan AnCora (test, dev); Occitan TTB (all); Italian ISDT, PUD; Romanian RRT; Latin PROIEL; Haitian Creole Autogramm; Spanish AnCora, Galician CTG, Portuguese Bosque, Ligurian, Neapolitan, Sicilian | CC BY-SA, CC BY, LGPL-LR, CC BY-NC-SA, per treebank |
| Tatoeba per-language exports | French 20,000 of 727,867 (seed 42), Catalan 10,963, Occitan 25,112, Italian, Spanish, Portuguese 10,000 each, Romanian, Latin 5,000 each, Picard 1,352, Walloon 53, Haitian Creole 163, and the small others | CC BY 2.0 FR |
| Wikipedia, plain-text extracts | ca 10 articles, oc 22, it 6, frp 5, pcd 3, wa 5; fr: 18 on Catalan and Occitan subjects (Catalogne, Barcelone, Occitan, Félibrige, Joan Miró, Perpignan…), 12 on regional French and the regional languages (Français québécois, Joual, Français de Belgique, Helvétisme, Montréal, Lausanne, Arpitan, Picard, Wallon…), 2 on Italian cities | CC BY-SA 4.0 |
| Project Gutenberg | Louis Hémon, *Maria Chapdelaine* (1913, #13525, Québec dialogue); Louis Fréchette, *Félix Poutré* (1862, #15361, Québec drama); Camille Lemonnier, *Au cœur frais de la forêt* (#65494, Belgian); Victor Cherbuliez, *Jacquine Vanesse* (#29857, Genevan) | public domain |

A Python prototype chose and ablated the tables; a Rust one (whichlang 0.1.1, the guard as D3 and
D4 write it) agrees with it on all 62,076 blocks read as French, before and after the review's four
removals (D3), and timed it (D5). Then the guard was applied to a scratch checkout of `main` and
every baseline run (D7).

**Before and after**, share of blocks read as French (share of the text in brackets):

| Neighbour | Corpus | Blocks | Before | After |
|---|---|---|---|---|
| Catalan | UD AnCora (sentences) | 3,539 | 30.1 % (29.9) | 2.0 % (1.2) |
| | Tatoeba (short sentences) | 10,621 | 25.0 % (26.5) | 8.0 % (6.2) |
| | Wikipedia (paragraphs) | 2,393 | 33.0 % (34.1) | 11.2 % (1.9) |
| Occitan | UD TTB | 1,467 | 58.5 % (60.6) | 15.1 % (8.4) |
| | Tatoeba | 24,098 | 41.8 % (44.4) | 26.1 % (22.2) |
| | Wikipedia | 2,065 | 50.1 % (79.1) | 18.6 % (5.6) |
| Romanian | UD RRT | 727 | 30.0 % (28.9) | 3.0 % (1.6) |
| | Tatoeba | 4,904 | 22.5 % (24.0) | 7.5 % (5.7) |
| *Not guarded (D2)* | Franco-Provençal, Wikipedia | 279 | 62.7 % (91.5) | 60.9 % (86.9) |
| | Picard, Tatoeba and Wikipedia | 1,366 | 90.3 % (91.9) | 87.9 % (86.6) |
| | Walloon, Tatoeba and Wikipedia | 291 | 50.5 % (77.6) | 45.4 % (63.2) |
| | Italian, UD, Tatoeba, Wikipedia | 13,946 | 0.2 % (0.1) | 0.2 % (0.1) |
| | Latin, UD and Tatoeba | 6,062 | 18.1 % (22.2) | 17.7 % (21.6) |
| | Haitian Creole, UD and Tatoeba | 296 | 25.3 % (33.0) | 23.3 % (27.4) |

Pooled: Catalan 27.2 → 7.2 % of blocks (30.6 → 2.8 % of the text), Occitan 43.3 → 25.0 % (55.4 →
16.4 %), Romanian 23.5 → 6.9 % (25.6 → 4.3 %). What leaks is short lines without a function word of
either table (« Soi content de te veire. »), sentence-initial capitals (« În anul următor… », D4),
and Tatoeba's Gascon and Provençal.

**French refused.** Of 42,961 French blocks, whichlang reads 41,312 as French (the other 3.8 % —
headings, lists, names and titles — were already lost, guard or not). The guard refuses 26
of them (0.063 %; 0.081 % of the text):

| French corpus | Read as French | Refused |
|---|---|---|
| UD: GSD, Sequoia, PUD, FQB, ParisStories, Rhapsodie, ParTUT | 7,412 | 0 |
| Tatoeba | 19,827 | 0 |
| Québec (Hémon, Fréchette), Belgian (Lemonnier), Swiss (Cherbuliez) books | 4,193 | 0 |
| Wikipedia on Catalan and Occitan subjects | 5,884 | 20 |
| Wikipedia on Italian cities | 720 | 4 |
| Wikipedia on regional French and the regional languages | 3,276 | 2 |

By length: 6 of 15,419 blocks under 40 bytes, 8 of 14,328 from 40 to 99, 12 of 11,565 from 100.
Fourteen of the 26 are not French: Catalan place lists and a park's name (« Gràcia : Vallcarca i
els Penitents… »), Catalan bibliography entries, Occitan and Provençal verse and prose quoted, an
Occitan glossary and a heading naming an Occitan school, two Arpitan quotations. Twelve are French:
six sentences about letters and forms (« [e] fermé en roussillonnais. », « L'occitan ignore …
(type e muet). »), two quoting Occitan words (« disaient lo romans… », « publication de la
Gramatica occitana segon los parlars… »), four Italian captions with `del` or `dei` (« Piazza del
Duomo, la célèbre place de la cathédrale »). Québécois « qu'i s'aident pas pantoute » (*i* for
*il*), Belgian « septante » and « une fois », inclusive writing and ordinals are kept.

## Decisions

### D1 — Function words, counted, compared: Spanish's rule

For a block whichlang reads as French, count the words of two tables (D3): neighbour markers, the
function words Catalan, Occitan and Romanian write and French rarely does, and French markers,
French function words they rarely write. The block is refused as French when neighbour markers
outnumber French ones; a tie, or no marker, keeps it French — a short line cannot be judged, and
French recall comes first. One neighbour table, not three: the guard asks « French or not », not
which neighbour. That departs from Spanish's guard, which compares each neighbour's count with
Spanish's in turn; measured with the table split by language, comparing each neighbour with French
spares one block of the French corpora (an Arpitan quotation) and lets 0.1 point more of Romanian
through, which three tables are not worth.

*Rejected — a second detector with these classes.* A new dependency and a new model in the WASM
bundle (whichlang alone is 65,536 weights, 256 KiB), for a question a few hundred function words
answer; and Occitan has no class in the small detectors either.

*Rejected — two markers at least before refusing.* Measured: Catalan 7.2 → 14.3 % of blocks still
read as French, Occitan 25.0 → 35.8 %, Romanian 6.9 → 14.1 %, to spare 15 of the 26 French refusals,
most of them quoted Catalan and Occitan. « More than French » has no parameter to tune.

### D2 — Catalan, Occitan and Romanian; not the others

Measured above. Catalan and Occitan are the change's reason; Romanian joins them because whichlang
reads a quarter of it as French and its function words (`în`, `să`, `și`, `cu`, `pe`, `pentru`,
`fost`…) are not French words — 32 entries; the one more block they refuse in the French corpora
is an Arpitan quotation (`pe`).
The others are left as they are, with what was measured:
- **Italian**: whichlang has an Italian class; 0.2 % of Italian blocks read as French. Nothing to do.
- **Franco-Provençal**: it shares French's own function words (`et`, `est`, `du`, `des`, `sont`,
  `le`), so it outweighs any neighbour table. Eight Arpitan words that French does not write
  (`avouéc`, `pués`, `adonc`, `dedens`, `dês`, `celi`, `étot`, `solament`) take its text read as
  French from 86.9 to 59.8 % with no French block refused — on 279 paragraphs from five articles,
  two of the words Middle French. Open question 3.
- **Picard and Walloon**: Oïl languages, French's own function words; the words that tell them
  apart are informal French's and Québécois spellings (`pi`, `ti`, `alle`, `bin`, `pa`, `ki`, `kel`,
  `min`). A table of them took Walloon from 63.2 to 12.8 % of the text, Picard from 86.6 to 69.5 %,
  and refused five more blocks of the French corpora, which hold almost no chat French: rejected.
- **Latin** (18 %) shares `et` and `est`; **Haitian Creole** (25 %) writes `pa`, `ki`, `pou`, `sa`,
  chat French's spellings. Left as measured.

### D3 — The tables, measured word by word

Closed-class words — articles, pronouns, prepositions, conjunctions, adverbs, the copula and the
auxiliaries — kept only when the measurement, word by word, shows they catch neighbour text without
refusing French, and none as a common English, chat or Québécois spelling:

- **Neighbour markers, 152**: Catalan 64 (`amb`, `els`, `és`, `són`, `molt`, `més`, `també`,
  `però`, `aquest`, `això`, `una`, `el`, `del`, `al`, `vaig`, `està`, `té`…), Occitan 63 (`lo`,
  `los`, `las`, `e`, `per`, `dins`, `èra`, `foguèt`, `coma`, `aquò`, `çò`, `degun`, `dau`, `dei`,
  `lei`, `deu`, `èi`, `ua`, `dambe`…), seven shared (`al`, `amb`, `aquesta`, `aquestes`, `del`,
  `dels`, `una`), Romanian 32 (`în`, `să`, `și` and `şi`, `cu`, `pe`, `este`, `fost`, `că`,
  `pentru`, `din`, `sunt`, `care`…).
- **French markers, 69**: `le`, `et`, `est`, `une`, `du`, `des`, `au`, `aux`, `je`, `j`, `il`,
  `ils`, `elle`, `elles`, `nous`, `vous`, `dans`, `avec`, `pour`, `sur`, `cette`, `ces`, `sont`,
  `suis`, `était`, `été`, `être`, `avait`, `fait`, `mais`, `ou`, `où`, `ça`, `très`, `plus`,
  `aussi`, `leur`, `moi`, `toi`, `cela`, `dont`, `comme`, `tout`, `rien`, `jamais`, `toujours`,
  `encore`, `beaucoup`, `déjà`, `alors`, `donc`, `puis`, `bien`, `chez`, `sans`, `sous`,
  `depuis`, `pendant`, `avant`, `parce`, `à`, `là`, `ici`, `quoi`…

The first draft held four more, dropped in review, each measured: `uns` and `unes`, French's own
« les uns », « les unes » (« Les uns disent oui, les autres non. » was refused; without them Catalan
7.1 → 7.2 %, Occitan 24.9 → 25.0 %, no French block refused or spared in the corpora), and `ce` and
`lui`, Romanian's *what* and *his*, in 4.3 % and 3.1 % of its blocks (without them Romanian
7.8 → 6.9 %, nothing French refused).

What was left out, and why, each measured against the tables above:

| Word | Language | Caught | French refused | Reason |
|---|---|---|---|---|
| `es` | Occitan *is* | Occitan 25.0 → 21.7 % | +81 | « Tu es donc las… », « Tu n'es pas sale. » |
| `on` | Catalan, Occitan *where* | — | +44 | « on y trouve » (Spanish's table has it) |
| `i` | Catalan *and* | Catalan 7.2 → 6.6 % | +15 | Québécois « qu'i s'aident » (*il*), Catalan place lists |
| `mai` | Occitan *more* | 0.5 point | +4 | the month (« 27 mai 1944 ») |
| `ha` | Catalan *has* | Catalan 7.2 → 6.8 % | +3 | hectares, « ha ha ha » |
| `fou` | Catalan *was* | — | +2 | « tu n'es pas fou ? » (Spanish's table has it) |
| `o`, `pot` | Catalan, Occitan | — | +2, +1 | letters discussed, « le pot » |
| `soi`, `com`, `pus`, `jamai`, `hi`, `han`, `van`, `fins` | | nothing measured | nothing measured | French words or chat and Québécois spellings: « chez soi », `com` (*comme*), `pus` (*plus*), `jamai`, « hi hi », `Han`, `van`, « à des fins de » |
| `les` (French table) | Catalan's article | Catalan 8.2 → 7.2 % without it | 2 spared with it | kept out |
| `pas`, `son`, `mon`, `qui`, `on`, `ont` (French table) | Occitan writes them | Occitan 27.2 % with `pas`, 28.4 % with all six | none spared | kept out |

Kept although shared somewhere: `le` (Toulouse Occitan, Italian, Romanian *them*), `et` (Catalan's
pronoun, Latin), `des` (Catalan's *des de*), `du`, `au` (Romanian *they have*, Gascon), `nous` and
`est` (Catalan *new*, *east*), `donc` (Occitan) — removing any catches at most 0.3 point more and
refuses up to 8 more French blocks. Kept although French writes them as other words: `las`
(*weary*), `mas` (*farmhouse*), `coma`, `sus` (*en sus*), `fi` (*faire fi*), `té` — each in at most
8 of the 42,961 French blocks, none refused; without them Occitan 25.0 → 26.0 %, Catalan
7.2 → 7.4 %, and one block spared, a Provençal verse (Risks).

The Spanish guard's three tables stay as they are; French's are separate constants, beside them,
sorted for binary search and disjoint (a test holds both). French's markers are this guard's, not
change 41's function-word tables: the two answer different questions (what French does not share,
what a reader is not shown), and keeping them apart leaves this change independent of 41.

### D4 — How the guard reads a block

A word is a run of letters and combining marks, joined across a hyphen, a middle dot (U+00B7), a
non-breaking hyphen or a full stop between two letters; an apostrophe, a space or other punctuation
ends it; a run holding a digit is not a word. A word holding a combining mark is composed (NFC)
before it is looked up. A neighbour marker counts only where it is written in lowercase; a French
marker counts in any case. Words longer than the longest marker (9 bytes) are not looked up.

Each rule is measured against the alternative, with the tables of D3:
- **Splitting on every non-letter, as Spanish's guard does**: 52 French blocks refused instead of
  26, at the same recall — `2e` and `1re` give `e`, `ingénieur-e-s` and `étudiant·e·s` give `e`,
  `e-mail` gives `e`, and `e` is Occitan's *and*. The joiners keep `col·lecció` one Catalan word too.
- **Counting capitalised neighbour words**: Catalan 7.2 → 5.4 %, Occitan 25.0 → 20.5 %, Romanian
  6.9 → 5.9 %, but 47 French blocks refused instead of 26, among them FQB's « Que signifie "El
  Niño" en espagnol ? », « Quand eurent lieu les émeutes de Los Angeles ? », « Que signifie "E
  pluribus unum" ? » and Lausanne's coordinates (« 6° 37′ 13″ E »). A capital makes a name, as the
  Spanish names rule reads it; the cost is a neighbour sentence opening on a marker (« În anul
  următor… »).
- **Combining marks as separators, no composition**: an NFD block (the corpus's `technique` page
  holds one) splits `mémoire` into `me` + `moire` and `été` into `e` + `te`, and `e` is Occitan's
  *and*: « La réunion a été reportée à la semaine prochaine. », decomposed, is refused. Read as one
  word and composed, a decomposed Catalan block is still refused and a decomposed French one kept
  (both tested).

The French corpus's blocks, read this way: whichlang reads 62 of its 66 as French; the guard refuses
one, the `mixte` page's Occitan block (`e`, `degun`, `qual` against none; its opening `Lo` is
capitalised). Its Catalan block is read as Italian before and after.

### D5 — One detection function; a guard runs only for the language it protects

```rust
fn detect(trimmed: &str, languages: &[StudiedLanguage]) -> Option<StudiedLanguage>
```

whichlang's answer, if it is one of `languages`, unless that language's guard refuses the block:
none for English, `iberian_neighbour` for Spanish, `romance_neighbour` for French. The gate asks
`detect(trimmed, &[studied]).is_some()`; the vote gives the block's weight to the candidate `detect`
returns among the candidates. Spanish's sentinel (a guarded block « read as Portuguese ») goes:
`None` says it.

For English and Spanish this is the same function as today: whichlang's English answer was never
rewritten; a Spanish block is refused exactly when `iberian_neighbour` says so, in the gate and in
the vote; a French answer was never a Spanish or English one. So their analyser versions do not
move, and the four baselines prove it (D7).

The guard's cost, timed on the 2026-10-09 samples' French blocks (UD, Tatoeba, Wikipedia, the
books; Apple M2 Max, release build; WebAssembly under Node 22, the engine's runtime family), as D3
and D4 write it — two tables, a scan by character — and in the prototype's faster shape, one merged
sorted table of (word, kind) and a byte scan for ASCII, which answers the same on all 62,076 blocks:

| Text read as French, µs per KB | whichlang | French guard | merged table | Spanish guard (for scale) |
|---|---|---|---|---|
| native | 16.5–17.2 | 16.5–21.1 (100–123 % of whichlang) | 12.5–16.0 (75–94 %) | 21.5–24.0 |
| WebAssembly | 14.0–15.1 | 19.4–23.8 (138–159 %) | 14.6–19.0 (103–126 %) | 19.9–21.5 |

A page analysis (`analyse_page`, the es-fr pack on 40-block pages of Spanish UD and Tatoeba text,
native) costs 233–251 µs per KB, whichlang 7 % of it; the French guard would be 7–9 % of it on the
blocks it runs on (5–6.5 % merged). A 20 KB French article costs about 0.4 ms more in WebAssembly
(0.3 ms merged). The implementation may take the merged shape; the tests do not change.

Why the guard runs only when French is asked about: a reader studying English — today's every
reader, French-native — reads French pages all day; their blocks are read as French and excluded.
Guarding them anyway (`detect` rewriting every `Fra` answer, as it rewrites every `Spa` answer
today) would more than double detection's cost on those pages, in WebAssembly, for an answer that
cannot change: French is not their language either way. With D5's shape, that reader pays nothing,
and also stops paying Spanish's guard on Spanish blocks.

*Rejected — the guard inside today's `detect(trimmed)`, French refused as Portuguese.* The
template's shape, measured above as +140–160 % of detection's cost in WebAssembly on the en-fr
reader's French pages.

### D6 — French's analyser version, bumped after change 41's

The guard changes which French blocks are analysed, which the analyser version exists to signal
(*An analyser version per studied language*). English and Spanish do not move (D5).

The requirement states the bump, not the number. The number follows the order, and one order leaves
every French requirement true:
- after change 41 (`1.0.0`): `1.1.0`, as Spanish's guard (`1.1.0`) followed Spanish's analysis
  (`1.0.0`);
- not between 40 and 41: change 40's MODIFIED *French is a studied language served by the baseline
  analysis* holds French at « `0.2.0` while its lemmatisation is the baseline's », and its scenarios
  *Each language reports its own version* and *What the baseline shows today* read `0.2.0`. A
  `0.3.0` there would have this change MODIFY both of change 40's requirements and archive after it
  (change 40's D9) — the owner's call (open question 4);
- not before change 40, whose spec names `0.2.0` (change 39's D4 reserved it).

Change 41 is not proposed yet; it rewrites the same requirement for its cascade, and should word
French's version so that a later French bump does not contradict it — Spanish's requirements still
name `1.0.0` and `1.1.0` beside a `1.2.0` core.

The bump moves what every French version bump moves: `FRENCH_ANALYZER_VERSION` and its doc line,
the fixture's `manifest.json`, and the tests that name French's version (`language.rs`,
`engine.rs`, `packs/pack.rs`, `crates/lingua-pack` `lib.rs` and `tests/pipeline_testdata.rs`,
`french_baseline.rs`, `languages.rs`, the extension's `test/packs.spec.ts`).

### D7 — What the golden shows; what cannot move

Applied to a scratch checkout of `main` (`03bdb0d3`, French at `0.1.0`, bumped to `0.2.0` for the
run): every lingua-core, lingua-pack and lingua-wasm host test passes, clippy is clean, and
- `english_baseline`, `spanish_baseline`, `es_en_baseline`, `en_es_baseline` and `cross_native`
  pass **without re-blessing**: en-fr, es-fr, es-en and en-es output is byte for byte `main`'s. By
  construction too: their engines hold no French pack, so French is never a language `detect` is
  asked about, and English's and Spanish's paths are D5's identity;
- `french_baseline` fails on its first difference and is re-blessed once. `fr-en.golden` moves on
  18 of its 136 probes:
  - `pack`: the fixture's `analyzer_version` (4,948 bytes either way at the same length);
  - the 13 `analyse new-reader` and 4 `analyse reader` lines: `analyzer_version`;
  - `analyse new-reader mixte` and `analyse reader mixte`: block 4, the Occitan « Lo far se
    quilhava a la broa de la falèsa, e degun se remembrava pas de qual l'aviá bastit. », leaves the
    analysis — its 18 tokens (17 unknown and one name for the new reader; 12 unknown, 5 known and
    one name for the reader) — so the page counts 29 tokens instead of 46, the new reader 0 known
    (0 %) as before, the reader 12 known instead of 17, 41 % instead of 37 %: the French words they
    know (`a`, `la` twice, `de` twice) no longer counted inside an Occitan sentence;
  - nothing else: the phrase glosses of the `mixte` blocks (« El far s'alçava », « Lo far se
    quilhava »…) are selections, glossed in the language the reader names without detection, and
    do not move.

These figures are measured on `main` (`0.1.0`). Change 40 moves 26 probes of the same golden,
the `mixte` lines among them (French tokens split at `0.2.0`, « l'aviá » read as `le` + `aviá`), and
change 41 more; this change's re-bless is re-measured on top of both at implementation (D6), and its
pull request states what moved: the version on every `analyse` line and the pack line, and the
Occitan block leaving the two `mixte` lines. The corpus is not touched: its
Catalan block reads as Italian, so Catalan and Romanian are carried by the unit tests (D8) rather
than by a new page the owner has not read (change 39's task 5.1).

### D8 — Tests

In `language.rs`: the spec's scenarios, each on a sentence written for the change and checked to be
read as French by whichlang first (so the test fails if a whichlang update stops reading it so,
rather than passing for the wrong reason); the corpus's Occitan block; the tables sorted, disjoint,
and `LONGEST_MARKER_BYTES` their longest entry; the word reading (`L'e-mail des étudiant·e·s, 2e
cycle, XIIe — qu'i vient.` reads `L`, `e-mail`, `des`, `étudiant·e·s`, `cycle`, `XIIe`, `qu`, `i`,
`vient`); a decomposed Catalan block refused and a decomposed French one kept; a capitalised marker
not counted; a tie or no marker kept French; French's own « Les uns disent oui, les autres non. »
kept; `detect` answering English and Spanish as before for any candidates, and a Catalan block read
as French answering nothing for `[English, Spanish]`. The two existing tests that call `detect`
directly are rewritten for its new signature, their assertions kept. In `french_baseline.rs`: the
`mixte` page's analysis holds blocks 0 and 6 only.

All of this ran green in the prototype (38 tests in `analysis::language`) with the first draft's
tables; the review's four removals (D3) were re-checked on every scenario sentence and on the
corpus, which reads as above.

### D9 — Gates and OpenSpec

1. en-fr, es-fr and es-en goldens and en-es's untouched: `git diff --stat origin/main --
   crates/lingua-wasm/tests/baseline/{en-fr,es-fr,es-en,en-es}.golden scripts/lingua-data/tables`
   is empty; a re-bless of any of them in this pull request is a review failure.
2. `fr-en.golden` re-blessed once, its diff as D7 says.
3. The workspace gates (fmt, clippy `-D warnings`, tests — `lingua-agent`'s among them, it calls
   `detect_document_language` —, `llvm-cov` ≥ 80 %, `wasm-pack test --node`), the extension's
   (`yarn lint`, `format:check`, `typecheck`, `test`, `build`, `check:variants`), and the
   lingua-data Python suite, which reads nothing that moves.

OpenSpec: one ADDED requirement in `lingua-analysis`, *Catalan, Occitan and Romanian are not read
as French*. It names no version number (D6) and rewrites nothing: change 39's two requirements are
held by 39 and reworded by change 40, and are left to them. `archiveAfter` names
`add-lingua-french-baseline`, whose requirement makes French a studied language, and changes 40
and 41 (`add-lingua-french-tokenisation`, `add-lingua-french-analysis`), implemented before it
(D6); `openspec_archive_order.py` exits 10 naming those still open.

## Risks / Trade-offs

- [Chat French is barely in the measurement] → No free SMS corpus was fetched; the tables leave
  out every chat and Québécois spelling found (`com`, `pus`, `jamai`, `ki`, `pi`, `alle`, D3), and
  the corpus's `informel` page is read as French before and after.
- [A text about letters or sounds] → « le e muet », « [e] fermé »: refused, measured (six blocks, all
  in the French Wikipedia's articles on Occitan and Catalan). Accepted.
- [An Italian caption in a French page] → « Piazza del Duomo, la célèbre place de la cathédrale »:
  refused, 4 of 720 blocks of two French articles on Italian cities. Dropping `del` and `dei` would
  let 75 Catalan and 88 Occitan blocks back in; accepted.
- [The leak] → Short lines without a marker (a quarter of Tatoeba's Occitan, mostly Gascon and
  Provençal), and sentence-initial markers. The spec says so; Wikipedia-sized paragraphs leak 1.9 %
  (Catalan) and 5.6 % (Occitan) of their text.
- [A page quoting Catalan or Occitan at length] → Its quotations are excluded: they are not French.
- [Spanish wins an Occitan page it did not win before] → For a reader of Spanish and French, an
  Occitan page voted French (55.4 % of its text against 19.2 % for Spanish); with the guard, French
  keeps 16.4 %, Spanish 19.2 %. The page then reads its Spanish-looking blocks as Spanish, as it
  would for a reader of Spanish alone today. Open question 2.
- [A whichlang update] → The tables were chosen against whichlang 0.1.1's answers; the tests check
  that each scenario sentence is still read as French first, and a bump re-runs the measurement.
- [Romanian beyond the change's name] → 32 words; the one French-corpus block they refuse is an
  Arpitan quotation (D2). The owner may drop them (open question 1), which removes one scenario and
  takes Romanian blocks read as French from 6.9 back to 22.4 %; `ce` and `lui` stay out of the
  French table either way (D3).
- [A French noun a neighbour writes as a function word] → `las`, `mas`, `coma`, `sus`, `fi`, `té`
  (D3): a short French line holding one and no French marker is refused (« Un vieux mas provençal
  restauré. »); no such line among the 42,961 French blocks measured. Accepted for the point of
  Occitan they catch.

## Migration Plan

Nothing to migrate: no stored format, wire field, pack or pin moves; no released build analyses
French. Rollback is a revert, with the golden re-blessed back.

## Open Questions

For the owner, none blocking:
1. **Romanian in the guard** (D2) — measured worth it and free; it widens the change's name from
   « Catalan and Occitan ». Keep it, or leave Romanian to whichlang?
2. **Spanish's leak on Occitan** — 19.2 % of the Occitan text is read as Spanish and Spanish's guard
   has no Occitan table. A Spanish change could give it one; it would bump Spanish's analyser
   version, and es-fr's corpus (`pages-es.txt`) has no Occitan block to move. Worth a change of its
   own after this one?
3. **Franco-Provençal** (D2) — eight Arpitan words take its French-read text from 86.9 to 59.8 % with
   nothing French refused, on a thin sample. Add them now, or leave Franco-Provençal with Picard and
   Walloon?
4. **The order** (D6) — after change 41 (`1.1.0`), the one order that contradicts no French
   requirement; sooner (after 40, `0.3.0`) means this change MODIFIES change 40's two requirements
   and archives after it. After 41?
