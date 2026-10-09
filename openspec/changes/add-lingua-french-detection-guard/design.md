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
a candidate in a released build. Change 40 (`add-lingua-french-tokenisation`, `0.2.0`) and change 41
(`add-lingua-french-analysis`, `1.0.0`) are proposed beside this one; neither touches detection.

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
D4 write it) agrees with it on all 62,076 blocks read as French, and timed it (D5). Then the guard
was applied to a scratch checkout of `main` and every baseline run (D7).

**Before and after**, share of blocks read as French (share of the text in brackets):

| Neighbour | Corpus | Blocks | Before | After |
|---|---|---|---|---|
| Catalan | UD AnCora (sentences) | 3,539 | 30.1 % (29.9) | 1.9 % (1.1) |
| | Tatoeba (short sentences) | 10,621 | 25.0 % (26.5) | 7.9 % (6.1) |
| | Wikipedia (paragraphs) | 2,393 | 33.0 % (34.1) | 11.2 % (1.9) |
| Occitan | UD TTB | 1,467 | 58.5 % (60.6) | 15.1 % (8.4) |
| | Tatoeba | 24,098 | 41.8 % (44.4) | 26.1 % (22.2) |
| | Wikipedia | 2,065 | 50.1 % (79.1) | 18.6 % (5.6) |
| Romanian | UD RRT | 727 | 30.0 % (28.9) | 3.4 % (2.1) |
| | Tatoeba | 4,904 | 22.5 % (24.0) | 8.5 % (6.7) |
| *Not guarded (D2)* | Franco-Provençal, Wikipedia | 279 | 62.7 % (91.5) | 60.9 % (86.9) |
| | Picard, Tatoeba and Wikipedia | 1,366 | 90.3 % (91.9) | 87.9 % (86.6) |
| | Walloon, Tatoeba and Wikipedia | 291 | 50.5 % (77.6) | 45.4 % (63.2) |
| | Italian, UD, Tatoeba, Wikipedia | 13,946 | 0.2 % (0.1) | 0.2 % (0.1) |
| | Latin, UD and Tatoeba | 6,062 | 18.1 % (22.2) | 17.7 % (21.6) |
| | Haitian Creole, UD and Tatoeba | 296 | 25.3 % (33.0) | 23.3 % (27.4) |

Pooled: Catalan 27.2 → 7.1 % of blocks (30.6 → 2.7 % of the text), Occitan 43.3 → 24.9 % (55.4 →
16.3 %), Romanian 23.5 → 7.8 % (25.6 → 5.2 %). What leaks is short lines without a function word of
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
function words Catalan, Occitan and Romanian write and French does not, and French markers, French
function words none of them uses. The block is refused as French when neighbour markers outnumber
French ones; a tie, or no marker, keeps it French — a short line cannot be judged, and French recall
comes first. One neighbour table, not three: the guard asks « French or not », not which neighbour.

*Rejected — a second detector with these classes.* A new dependency and a new model in the WASM
bundle (whichlang alone is 65,536 weights, 256 KiB), for a question a few hundred function words
answer; and Occitan has no class in the small detectors either.

*Rejected — two markers at least before refusing.* Measured: Catalan 7.1 → 14.2 % of blocks still
read as French, Occitan 24.9 → 35.8 %, Romanian 7.8 → 14.2 %, to spare 15 of the 26 French refusals,
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
auxiliaries — kept only when the measurement shows no French use, and none as a common English,
chat or Québécois spelling:

- **Neighbour markers, 154**: Catalan 66 (`amb`, `els`, `és`, `són`, `molt`, `més`, `també`,
  `però`, `aquest`, `això`, `una`, `el`, `del`, `al`, `vaig`, `està`, `té`…), Occitan 63 (`lo`,
  `los`, `las`, `e`, `per`, `dins`, `èra`, `foguèt`, `coma`, `aquò`, `çò`, `degun`, `dau`, `dei`,
  `lei`, `deu`, `èi`, `ua`, `dambe`…), seven shared (`al`, `amb`, `aquesta`, `aquestes`, `del`,
  `dels`, `una`), Romanian 32 (`în`, `să`, `și` and `şi`, `cu`, `pe`, `este`, `fost`, `că`,
  `pentru`, `din`, `sunt`, `care`…).
- **French markers, 71**: `le`, `et`, `est`, `une`, `du`, `des`, `au`, `aux`, `je`, `j`, `il`,
  `ils`, `elle`, `elles`, `nous`, `vous`, `dans`, `avec`, `pour`, `sur`, `ce`, `cette`, `ces`,
  `sont`, `suis`, `était`, `été`, `être`, `avait`, `fait`, `mais`, `ou`, `où`, `ça`, `très`, `plus`,
  `aussi`, `leur`, `lui`, `moi`, `toi`, `cela`, `dont`, `comme`, `tout`, `rien`, `jamais`,
  `toujours`, `encore`, `beaucoup`, `déjà`, `alors`, `donc`, `puis`, `bien`, `chez`, `sans`,
  `sous`, `depuis`, `pendant`, `avant`, `parce`, `à`, `là`, `ici`, `quoi`…

What was left out, and why, each measured against the tables above:

| Word | Language | Caught | French refused | Reason |
|---|---|---|---|---|
| `es` | Occitan *is* | Occitan 24.9 → 21.7 % | +80 | « Tu es donc las… », « Tu n'es pas sale. » |
| `on` | Catalan, Occitan *where* | — | +39 | « on y trouve » (Spanish's table has it) |
| `i` | Catalan *and* | Catalan 7.1 → 6.5 % | +15 | Québécois « qu'i s'aident » (*il*), Catalan place lists |
| `mai` | Occitan *more* | 0.4 point | +4 | the month (« 27 mai 1944 ») |
| `ha` | Catalan *has* | Catalan 7.1 → 6.7 % | +3 | hectares, « ha ha ha » |
| `fou` | Catalan *was* | — | +1 | « tu n'es pas fou ? » (Spanish's table has it) |
| `o`, `pot` | Catalan, Occitan | — | +2, +1 | letters discussed, « le pot » |
| `soi`, `com`, `pus`, `jamai`, `hi`, `han`, `van`, `fins` | | nothing measured | nothing measured | French words or chat and Québécois spellings: « chez soi », `com` (*comme*), `pus` (*plus*), `jamai`, « hi hi », `Han`, `van`, « à des fins de » |
| `les` (French table) | Catalan's article | Catalan 8.1 → 7.1 % without it | 2 spared with it | kept out |
| `pas`, `son`, `mon`, `qui`, `on`, `ont` (French table) | Occitan writes them | Occitan up to 27.1 % with them | none spared | kept out |

Kept although shared somewhere: `le` (Toulouse Occitan, Italian), `et` (Catalan's pronoun, Latin),
`des`, `du`, `au` — removing any catches at most 0.3 point more and refuses up to 8 more French
blocks.

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
- **Splitting on every non-letter, as Spanish's guard does**: 53 French blocks refused instead of
  26, at the same recall — `2e` and `1re` give `e`, `ingénieur-e-s` and `étudiant·e·s` give `e`,
  `e-mail` gives `e`, and `e` is Occitan's *and*. The joiners keep `col·lecció` one Catalan word too.
- **Counting capitalised neighbour words**: Catalan 7.1 → 5.2 %, Occitan 24.9 → 20.4 %, but 48
  French blocks refused instead of 26, among them FQB's « Que signifie "El Niño" en espagnol ? »,
  « Quand eurent lieu les émeutes de Los Angeles ? », « Que signifie "E pluribus unum" ? »,
  Lausanne's coordinates (« 6° 37′ 13″ E ») and Fréchette. A capital makes a name, as the Spanish
  names rule reads it; the cost is a neighbour sentence opening on a marker (« În anul următor… »).
- **No composition**: an NFD block (the corpus's `technique` page holds one) splits `mémoire` into
  `me` + `moire`; composed, a decomposed Catalan block is still refused and a decomposed French one
  kept (both tested).

The French corpus's blocks, read this way: whichlang reads 62 of its 66 as French; the guard refuses
one, the `mixte` page's Occitan block (`lo`, `e`, `degun`, `qual` against none). Its Catalan block
is read as Italian before and after.

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

The guard's cost, timed on the 2026-10-09 samples (Apple M2 Max, release build; WebAssembly under
Node 22, the engine's runtime family):

| Text read as French | whichlang | French guard | Spanish guard (for scale) |
|---|---|---|---|
| native, µs per KB | 15.9–16.5 | 12.5–15.4 (76–93 % of whichlang) | 21.5–22.8 |
| WebAssembly, µs per KB | 14.0–14.6 | 14.4–17.6 (102–120 %) | 20.0–21.3 |

A page analysis (`analyse_page`, the es-fr pack on 40-block pages of Spanish UD and Tatoeba text,
native) costs 233–243 µs per KB, whichlang 7 % of it; the French guard would be 5.2–6.0 % of it, on
the blocks it runs on. A 20 KB French article costs about 0.3 ms more in WebAssembly.

Why the guard runs only when French is asked about: a reader studying English — today's every
reader, French-native — reads French pages all day; their blocks are read as French and excluded.
Guarding them anyway (`detect` rewriting every `Fra` answer, as it rewrites every `Spa` answer
today) would double detection's cost on those pages, in WebAssembly, for an answer that cannot
change: French is not their language either way. With D5's shape, that reader pays nothing, and
also stops paying Spanish's guard on Spanish blocks.

*Rejected — the guard inside today's `detect(trimmed)`, French refused as Portuguese.* The
template's shape, measured above as +100–120 % on the en-fr reader's French pages.

### D6 — French's analyser version, bumped; the number follows the order

The guard changes which French blocks are analysed, which the analyser version exists to signal
(*An analyser version per studied language*). English and Spanish do not move (D5).

The requirement states the bump, not the number, so this change does not depend on 40 or 41:
- in the programme's order, after change 41 (`1.0.0`): `1.1.0`, as Spanish's guard (`1.1.0`)
  followed Spanish's analysis (`1.0.0`) — the recommendation;
- after change 40 (`0.2.0`) and before 41: `0.3.0`, and 41 keeps `1.0.0`;
- not before change 40, whose spec names `0.2.0` (change 39's D4 reserved it).

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
    know (`la`, `de`, `se`, `pas`) no longer counted inside an Occitan sentence;
  - nothing else: the phrase glosses of the `mixte` blocks (« El far s'alçava », « Lo far se
    quilhava »…) are selections, glossed in the language the reader names without detection, and
    do not move.

These figures are measured on `main` (`0.1.0`). Change 40 moves 26 probes of the same golden,
the `mixte` lines among them (French tokens split at `0.2.0`, « l'aviá » read as `le` + `aviá`), and
change 41 more; whichever of them merges first, this change's re-bless is re-measured on top of it
at implementation, and its pull request states what moved: the version on every `analyse` line and
the pack line, and the Occitan block leaving the two `mixte` lines. The corpus is not touched: its
Catalan block reads as Italian, so Catalan and Romanian are carried by the unit tests (D8) rather
than by a new page the owner has not read (change 39's task 5.1).

### D8 — Tests

In `language.rs`: the spec's scenarios, each on a sentence written for the change and checked to be
read as French by whichlang first (so the test fails if a whichlang update stops reading it so,
rather than passing for the wrong reason); the corpus's Occitan block; the tables sorted, disjoint,
and `LONGEST_MARKER_BYTES` their longest entry; the word reading (`L'e-mail des étudiant·e·s, 2e
cycle, XIIe — qu'i vient.` reads `L`, `e-mail`, `des`, `étudiant·e·s`, `cycle`, `XIIe`, `qu`, `i`,
`vient`); a decomposed Catalan block refused and a decomposed French one kept; a capitalised marker
not counted; a tie or no marker kept French; `detect` answering English and Spanish as before for
any candidates, and a Catalan block read as French answering nothing for `[English, Spanish]`. The
two existing tests that call `detect` directly are rewritten for its new signature, their
assertions kept. In `french_baseline.rs`: the `mixte` page's analysis holds blocks 0 and 6 only.

All of this ran green in the prototype (38 tests in `analysis::language`).

### D9 — Gates and OpenSpec

1. en-fr, es-fr and es-en goldens and en-es's untouched: `git diff --stat origin/main --
   crates/lingua-wasm/tests/baseline/{en-fr,es-fr,es-en,en-es}.golden scripts/lingua-data/tables`
   is empty; a re-bless of any of them in this pull request is a review failure.
2. `fr-en.golden` re-blessed once, its diff as D7 says.
3. The workspace gates (fmt, clippy `-D warnings`, tests, `llvm-cov` ≥ 80 %, `wasm-pack test
   --node`), the extension's (`yarn test`, `yarn check:variants`).

OpenSpec: one ADDED requirement in `lingua-analysis`, *Catalan, Occitan and Romanian are not read
as French*. It names no version number (D6) and rewrites nothing: change 39's two requirements are
held by 39 and reworded by change 40, and are left to them. `archiveAfter` names
`add-lingua-french-baseline`, whose requirement makes French a studied language;
`openspec_archive_order.py` exits 10 naming it alone.

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
  keeps 16.3 %, Spanish 19.2 %. The page then reads its Spanish-looking blocks as Spanish, as it
  would for a reader of Spanish alone today. Open question 2.
- [A whichlang update] → The tables were chosen against whichlang 0.1.1's answers; the tests check
  that each scenario sentence is still read as French first, and a bump re-runs the measurement.
- [Romanian beyond the change's name] → 32 words, nothing French refused by them; the owner may
  drop them (open question 1), which removes one scenario and leaves 7.8 → 23.5 % of Romanian blocks
  read as French.

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
4. **The order** (D6) — after change 41 (`1.1.0`), as recommended, or earlier (after 40, `0.3.0`)
   if the guard is wanted on the golden sooner?
