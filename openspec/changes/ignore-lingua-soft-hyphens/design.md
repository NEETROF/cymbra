# Design — ignore-lingua-soft-hyphens

## Context

A soft hyphen (U+00AD, `&shy;` in HTML) marks where a word may break at a line's end. It is invisible
unless the line breaks there; in this design ‧ marks one (« vi‧da »). E-book tools insert one at
every syllable break, as calibre's *Add soft hyphens* (`ebook-polish --add-soft-hyphens`) does from
LibreOffice's hyphenation patterns, and some web pages write them by hand. Text with soft hyphens
reaches Lingua unchanged:

- **The extension.** `collectBlocks` (`apps/lingua-extension/src/reading/blocks.ts:93`) walks a page's
  text nodes and concatenates their `nodeValue` per block. It normalises nothing: no NFC, no
  whitespace, no invisible characters. The EPUB reader (foliate) attaches the same reading session
  to each section's document (`reader/app.ts` → `session.attach`), so a book's text reaches the same
  walker as it is stored.
- **The tokeniser** (`crates/lingua-core/src/analysis/tokenize.rs:88`). UAX #29, through
  `unicode-segmentation` 1.13.3, gives U+00AD the word-break class *Format*, which rule WB4 attaches
  to the character before it. So « vi‧da » is one word and its span is right. Every rule after that,
  though, reads the word's slice with the hyphen still inside:
  - `push_word` (l. 154) looks up `vi‧da`, which no pack holds, and the plural fallbacks build
    `co‧sa` from `co‧sas` and `tou‧jour` from `tou‧jours`;
  - English's `could‧n't` strips `n't` and leaves `could‧`;
  - French's elisions compare `lors‧qu` with `lorsqu` and fail, so « lors‧qu’il » stays one token.
- **Detection** (`language.rs`):
  - `detect` (l. 124) hands the block to whichlang 0.1.1, which reads any non-ASCII character as
    two features (its 128-character bucket and its class; U+00AD has a class of its own). It also
    restarts its ASCII n-grams after it, so « vi‧da » loses `vid`, `ida` and ` vid`.
  - Spanish's guard (`iberian_neighbour`, l. 325) splits the block on every character that is not a
    letter or an apostrophe. U+00AD is not a letter (Cf).
  - French's guard (`guard_words`, l. 590) ends a word at anything but letters, marks, digits and
    its four joiners. Both guards read syllables.
- **The selection and the card** go through the tokeniser: `gloss_phrase` (`engine.rs:638`), and
  `word_grammar` (l. 752) for the word as the extension saw it — `hit.range.toString()`
  (`session.ts:897`), soft hyphens included.
- **The extension's own readings** (`reading/selection.ts`):
  - the selection is snapped to whole words through `WORD_CHAR = /[\p{L}\p{N}'’-]/u` (l. 132),
    which stops at U+00AD: a selection inside « vi‧da » snaps to « vi » or « da »;
  - the captured text (`captureFrom`, l. 158) and the sentence with the selection's offsets in it
    (`sentenceAndSelection`, through `collapse`, l. 32) keep the hyphens. They are what a loose-word
    or expression card is headed and keyed by (`selection-card.ts`), what the deck stores as form
    and sentence and syncs (`client_id`, `surface_form`, `source_sentence`), what translation is
    sent, and what read-aloud speaks.
- **Highlights.** The core's spans are UTF-8 byte offsets into the block as written.
  `byteToCharOffset` (`blocks.ts:179`) walks the block as written, U+00AD counting 2 bytes and 1
  UTF-16 unit, so a span over « vi‧da » paints the whole word.

## Goals / Non-Goals

**Goals:**
- A word holding soft hyphens is read, everywhere and in every studied language, as the word
  without them: the same token, dictionary form, class, gloss, card, deck key, detection and vote.
- Spans keep indexing the text as written, so highlighting and hit-testing do not change.
- No answer moves on text without soft hyphens, measured, and its cost is negligible.

**Non-Goals:**
- Other invisible characters: the zero-width space (U+200B), the word joiner (U+2060) and U+FEFF
  (open question 2).
- A visible hyphen at a line's end (« vi-\nda »), which pages and books rarely keep in their text.
- Rewriting what readers saved under a key holding a soft hyphen (open question 1).
- How the reader shows a soft hyphen: the book's CSS (`hyphens: auto`, `reader/book-style.ts`) is
  not touched.

## Measurement

Run on 2026-10-10 in the scratchpad, kept out of the repository. The corpora are change 42b's and
change 42's, plus the English Wiktionary. The scenario sentences committed are written for this
change. Blocks are sentences or paragraphs of 12 bytes or more, once trimmed:

| Language | Blocks | Sources | Soft hyphens inserted |
|---|---|---|---|
| Spanish | 459,666 | Change 42b's Spanish: Tatoeba's whole export (438,131), UD AnCora, GSD, PUD and COSER (5,279), 58 Spanish Wikipedia articles (11,275), five Argentine, Peruvian and Uruguayan books (4,981) | 3,444,427 in 461,520 blocks, by calibre (`hyph_es`) |
| French | 42,960 | Change 42's French: UD FQB, GSD, PUD, Sequoia, Rhapsodie, ParisStories, ParTUT (7,503), 20,000 Tatoeba sentences, French Wikipedia (11,279), four Swiss, Belgian and Québec books (4,276) | 407,840 in 40,003, by macOS (`fr_FR`) |
| English | 77,642 | The English Wiktionary's quotations and usage examples of English entries (kaikki), one in eight by a hash of the text, 40–400 characters, deduplicated | 783,802 in 76,274, by calibre (`hyph_en_US`) |
| Galician, Catalan | 8,932, 16,553 | Change 42's (Tatoeba, UD CTG and AnCora, Catalan Wikipedia) | by calibre (`hyph_gl`), by macOS (`ca_ES`) |

Hyphenation is calibre 9.15's own (`calibre.utils.hyphenation.hyphenate.add_soft_hyphens_to_words`,
LibreOffice's patterns through libhyphen), which is what *Add soft hyphens* runs: « La vi‧da
em‧pie‧za cuan‧do te das cuen‧ta de quién eres real‧men‧te, ma‧ña‧na o nun‧ca. » calibre has no French
or Catalan dictionary, so those use macOS 26's hyphenation (CoreFoundation's
`CFStringGetHyphenationLocationBeforeIndex`, the system's own): « Il faut tou‧jours gar‧der
la tête froide pen‧dant les conver‧sa‧tions dif‧fi‧ciles. » In every block, removing the soft hyphens
gives back the text byte for byte.

A Rust program links lingua-core twice: at `main` (`eddaf712`) and with the prototype (D1–D4). It
uses packs built from the committed tables (en-fr, es-fr, fr-en) and a reader calibrated at the
5,000 commonest dictionary forms. It asks each block whether it is studied (`block_is_studied`), and
analyses pages of 20 consecutive blocks of one source (`analyse_page`). It runs each page through the
vote among English, Spanish and French two and three at a time (`detect_document_language`), and asks
probe words through `gloss_phrase` and `word_grammar`. Every question is asked of the text as
written and of the hyphenated text.

**Today, the blocks kept by the gate as written and lost once hyphenated:**

| Language | Source | Kept as written | Lost hyphenated | Share of the text |
|---|---|---|---|---|
| Spanish | Tatoeba | 432,719 | 161,330 (37.3 %) | 38.3 % |
| | UD | 5,231 | 1,708 (32.7 %) | 32.2 % |
| | Wikipedia | 10,148 | 3,196 (31.5 %) | 21.6 % |
| | Books | 4,869 | 1,290 (26.5 %) | 20.1 % |
| | All | 452,967 | 167,524 (37.0 %) | 34.7 % |
| French | Tatoeba / UD / Wikipedia / books | 19,826 / 7,412 / 9,854 / 4,193 | 438 / 157 / 505 / 25 | 2.0 / 1.4 / 1.3 / 0.1 % |
| | All | 41,285 | 1,125 (2.7 %) | 1.2 % |
| English | Wiktionary | 77,415 | 516 (0.7 %) | 0.4 % |

- **Spanish**: the guard refuses 130,301 and the detector 37,223 (Portuguese 33,203, French 2,418,
  Vietnamese 1,163). 820 blocks go the other way.
- **French**: the guard refuses 957 and the detector 168 (Portuguese 103, Spanish 33). 431 go the
  other way, nearly all through the detector.
- **English**: the detector alone (French 223, Portuguese 110, German 102, Vietnamese 47).

The syllables the guards count, over the Spanish blocks the guard refuses:
- Galician's `do` 82,450 (« to‧do », « cuan‧do »), `da` 37,669 (« vi‧da », « ca‧da »), `lle` 9,649
  (« ca‧lle »), `das` 5,143, `nun` 3,185;
- Occitan's `ma` 40,257 (« ma‧ña‧na »), `li` 16,181, `tas` 8,889, `sas` 4,018 (« co‧sas »), `sia`
  2,624, `jos` 1,828 (« le‧jos »), `lor` 1,534 (« va‧lor »);
- Catalan's `per` 6,972 (« per‧so‧na »);
- while Spanish's own markers are cut: `había` 2,209 times, `ahora` 1,362, `también` 935, `después`
  868.

In French, the neighbours' `té` 428 (« chan‧té », « san‧té »), `lo` 168 (« lo‧ge‧ment »), `cu`, `per`
(« per‧sonne »), `ser`, `fi` and `pe`.

**Today, the pages:**

| | Spanish | French | English |
|---|---|---|---|
| Pages | 23,239 | 2,185 | 17,950 |
| Known share as written → hyphenated | 92.0 → 54.9 % | 90.2 → 71.3 % | 85.4 → 71.3 % |
| Per page: mean, median, worst | −37.6, −38, −95 points | −18.3, −18, −41 | −13.8, −14, −64 |
| Tokens holding a soft hyphen | 50.7 % | 30.5 % | 25.7 % |
| Distinct dictionary forms holding one | 72,360 (`pa‧ra`, `es‧tá`, `co‧mo`) | 38,511 (`aus‧si`, `fran‧çais`, `de‧puis`) | 89,501 (`af‧ter`, `peo‧ple`, `be‧ing`) |

Some figures come from the pages whose blocks are all studied both ways: 894 French and 15,841
English pages, but 44 Spanish ones, since most Spanish pages lose a block.
- Every token holding a soft hyphen has another dictionary form and no gloss: 15,237 French and
  108,302 English glosses lost.
- French: 47,183 known tokens become unknown, and 7,071 become proper nouns, sentence-initial and
  outside the lexicon.
- English: 277,692 become unknown, and 53,577 proper nouns.

The vote moves too:
- English pages: 63 go to French among English and French, 4 the other way; 6 go to Spanish among
  English and Spanish, 9 the other way;
- Spanish pages: 5 go to English among English and Spanish, 1 the other way; 4 go to French among
  Spanish and French;
- French pages: 1 goes to Spanish.

The probes, today: the phrase gloss of « vi‧da » is one unknown token `vi‧da`, no gloss, where
« vida » is known and glossed *Vie*. The card of « can‧tá‧ba‧mos » for `cantar` names no reading,
where « cantábamos » names the first person plural imperfect. « lors‧qu’il » is one token, and
« l’évi‧dence » is `le` + `évi‧dence`, unknown.

**With the change** (the prototype, D1–D4, versions not yet bumped so the same packs load):
- Every block, page, vote and probe of the hyphenated corpora answers exactly as its text without
  soft hyphens:
  - 0 blocks lost or gained in any of the five corpora (605,753 blocks);
  - all 43,374 pages identical, spans aside: tokens, dictionary forms, classes, glosses, counts,
    percentages and analysability;
  - 0 votes moved;
  - the probes identical.
- Each span of a hyphenated word covers that word as written, soft hyphens included: 2,088,100
  Spanish, 263,406 French and 538,746 English tokens.
- **On the text as written, nothing moves.** A digest of every gate answer, every page analysis's
  JSON and every vote, over each of the five corpora, is the same with the prototype as on `main`.
  The 689 tests of lingua-core, lingua-pack and lingua-wasm pass without re-blessing, the five
  goldens among them. No golden corpus, table, fixture or extension test holds a soft hyphen.
- **Neighbour languages** are guarded on hyphenated text as on their clean text.
  - Hyphenated Galician read as Spanish goes back from 8.9 % to the 21.2 % of its clean text, and
    Catalan from 9.9 % to 13.7 %: today's lower figures are the detector reading the syllables as
    Portuguese, not the guard.
  - Catalan read as French falls from 9.4 % to 7.2 %.
- **Cost** on text as written, native, best of three runs in each of three alternated rounds:
  - page analysis, Spanish 199.4 → 201.3 µs/KB, French 208.1 → 211.7, English 126.2 → 126.6 (at most
    1.7 %);
  - the gate alone within the runs' own spread (Spanish 1.055 → 1.049 s, French 0.171 → 0.176 s,
    English 0.208 → 0.211 s).

  The check is one scan for a two-byte sequence per block and per word; only a word holding one is
  copied. WebAssembly is timed by the implementation (task 5.4).

The real occurrences: four of the 597,086 blocks of change 42b hold soft hyphens of their own (a
Spanish Wikipedia « con‧ciencias », a Tatoeba sentence opening on one, a Galician « perfecciona‧o »,
one French Tatoeba line). They were left out of the hyphenated corpora. Their gate and vote answers
are the same with the prototype as on `main`.

## Decisions

### D1 — In the core, not in the extension

Stripping the hyphens in the extension before the analysis would leave the core as it is and bump no
version. But the spans would then index a text the page does not hold: « vida » after one removed
hyphen paints « vi‧d », and every later token of the block drifts (`byteToCharOffset` and the
segments know the text as written). Each consumer would need a stripped-to-source offset table: the
extension's pages and books, Safari through the same build, and the agent, which links the core
natively. The core is where every consumer reads a word, and it already keeps a token's text apart
from its source span: a French token's text is composed (NFC) while its span covers the source's
combining marks (add-lingua-french-analysis D1). The soft hyphen follows that rule.

### D2 — The tokeniser reads a word without its soft hyphens

`without_soft_hyphens(&str) -> Cow<str>` (`tokenize.rs`) returns the text unchanged, borrowed, when it
holds no U+00AD, and a copy without it otherwise. UAX #29 already keeps U+00AD inside its word, so
segmentation, the hyphen run, the spans and every offset the pre-pass computes from the source slice
stay as they are. What changes is the text each rule reads. Soft hyphens go before every rule, NFC
included:
- `push_word`: the word, before the apostrophe is normalised, so edge apostrophes, digits, NFC,
  English's `n't`, Spanish's `al`/`del` and the single-letter rule read « could‧n't » as `couldn't`;
- `push_compound`: the compound's text and its parts;
- `french_lowercase`, through which every French comparison goes: the elided forms, `au`/`aux`, an
  elided word written alone, and the runs the pack lists (`listed_whole`);
- the word after a French elision (`push_french_elisions`'s `next`, which decides `s'` before `il`),
  and the pieces of a hyphenated inversion (`is_french_inversion`).

Every token's text, and every part, is the word without soft hyphens: « vi‧da » is `vida` [0, 6).
« lors‧qu’il » is `lorsque` [0, 11) + `il` [11, 13), and « jus‧qu’au » is `jusque` + `à` + `le`, as
without. The prototype is 35 lines in three files.

### D3 — Detection reads the block without its soft hyphens

`block_is_studied` and `detect_document_language` read each block through `without_soft_hyphens`
before trimming it, so all of these see the clean text:
- the minimum length (`MIN_BLOCK_BYTES`);
- a vote's weight;
- whichlang;
- the three guards: Catalan and Galician, and Occitan, for Spanish; Catalan, Occitan and Romanian
  for French.

Both callers of `detect` go through this, so the gate and the vote cannot disagree, and the guards'
own word readings (`iberian_neighbour`, `guard_words`) and their tables are untouched. A block that
only the soft hyphens took over 12 bytes is now too short, as its clean text is (« lors‧qu’il », 13
bytes as written, 11 without).

### D4 — The selection and the card follow

`gloss_phrase` and `word_grammar` read through the tokeniser, so D2 is enough: the selection
« ma‧ña‧na » is `mañana`, known and glossed. The card of « can‧tá‧ba‧mos » for `cantar` names its
reading, even while the extension still hands the word with its hyphens (D5). `word_grammar`'s
fallback for a written word that yields no token reads it without soft hyphens too.

### D5 — The extension stops cutting at the hyphen

In `reading/selection.ts` and `reading/session.ts`:
- `WORD_CHAR` gains U+00AD, so a selection that stops inside « vi‧da » snaps to the whole word.
- `collapse`, which reads the block, the text before the range and the range for
  `sentenceAndSelection`, drops U+00AD as it collapses whitespace. The sentence holds no soft hyphen,
  and the selection's offsets stay right in it, since all three texts are read the same way. That
  sentence is what a card stores, what translation marks and sends, what read-aloud speaks, and what
  `gluedWord` reads to speak a French elided piece with its word.
- The captured text (`captureFrom`) and the word as written (`pageHit`, `session.ts:897`) are read
  without soft hyphens. A loose-word or expression card is then headed and keyed by `vida`, and the
  word card's form is the word.
- Nothing else moves. Highlights and hit-testing read the core's spans against the text as written.
  `collectBlocks` hands the core the block as written, so a page and its spans keep one text.

How Bergamot reads a soft hyphen was not measured. Since no sentence sent to it holds one, the
question does not arise.

### D6 — Every analyser version is bumped

English `1.1.0` → `1.2.0`, Spanish `1.3.0` → `1.4.0` (`1.4.0` → `1.5.0` as implemented: change 42c
bumped it first), French `1.1.0` → `1.2.0`. On text holding a
soft hyphen, the change moves each language's tokens, dictionary forms, classes, counts, gate and
vote (Measurement). The core's rule, *An analyser version per studied language*, bumps the version of
every language whose output a change can alter, and every pack is compared with its language's
version by exact match. Change 41b bumped nothing because only glosses moved; here, how a page is
read moves. Text without soft hyphens does not move, so on the goldens only the version does (D7).
English and French stay equal, at `1.2.0`. Two tests already use Spanish's version as « another
language's » for that reason (`lingua-pack`'s `lib.rs`, `lingua-core`'s `pack.rs`), and their comments'
« English's `1.1.0` » is reworded.

### D7 — What moves, and what cannot

The prototype and the bump were applied to a scratch checkout of `main` (`eddaf712`), the goldens
re-blessed once, and every Lingua Rust test run. What moved is this, and nothing else:
- **The goldens**, on the version alone. Each moved line differs from its old one by the version
  string only, checked line by line:
  - `en-fr.golden`: 16 lines, the pack line and the 15 analyses;
  - `en-es.golden`: 16 lines, the same;
  - `es-fr.golden`: 19 lines — its pack line, the `beside en-fr` line and the 17 analyses;
  - `es-en.golden`: 18 lines, the pack line and the 17 analyses;
  - `fr-en.golden`: 19 lines — its pack line, the `beside es-en` line and the 17 analyses.
- **The packs.** The five committed pairs are re-reduced from their pinned sources. Each reducer reads
  its language's version from `analysis/mod.rs`, and no table holds a soft hyphen. Only `manifest.json`
  (`analyzer_version`) and `pin.json` (the pack's sha256) move: tables byte for byte, `pack_version`
  and sizes unchanged. The scratch, with the version written in, built:

  | Pair | sha256 | Bytes |
  |---|---|---|
  | en-fr | `582f7762…` | 1,835,638 |
  | en-es | `a95af250…` | 1,688,931 |
  | es-fr | `d617856d…` | 2,224,439 |
  | es-en | `507abaae…` | 2,608,413 |
  | fr-en | `d013a548…` | 2,527,222 |

  Should change 49 commit `tables/fr-es/` first, fr-es is re-reduced too.
- **The fixtures.**
  - `testdata/*/manifest.json` (five), with their built bytes recorded in
    `crates/lingua-pack/tests/pipeline_testdata.rs`: en-fr `ebb1f3c7…`, en-es `51066821…`, es-fr
    `652ab141…`, es-en `636e693e…`. Sizes are unchanged.
  - Two committed files are the en-fr fixture's build (`build.sh --testdata en-fr`):
    `crates/lingua-wasm/tests/fixtures/pack.lingua`, and `apps/lingua-extension/test/fixtures/
    en-fr.testdata.lingua` (`yarn gen:fixtures`), which `packs.spec.ts` checks against the core's
    English version.
  - The parity golden `crates/lingua-wasm/tests/fixtures/golden.json`, on its version alone (one
    line; `LINGUA_UPDATE_GOLDEN=1 cargo test -p lingua-wasm --test parity`).
  - The agent's `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` (the es-fr fixture's build) and
    its English `pack.lingua`, re-stamped at `1.2.0`. The 747-byte fixture predates the testdata
    builder; its metadata's version is the only change, made through `read_container` /
    `write_container`.
- **The tests that write a version out.**
  - In lingua-core: `english_keeps_its_analyser_version_and_spanish_has_its_own` (`language.rs`),
    `spec_scenario_the_page_analysis_does_not_move` (`engine.rs`) and
    `spec_scenario_loading_the_en_fr_pack_names_english` (`pack.rs`).
  - In lingua-wasm: `spec_scenario_a_second_language_is_served_by_its_own_pack`
    (`tests/languages.rs`, two literals), `analysis_matches_golden` (`tests/parity.rs`, through
    `golden.json`), and `a_fixture_left_behind_its_analyser_names_its_manifest`
    (`french_baseline.rs`, « this core is 1.1.0 »).

  Each reads the constant or the new number. The doc comments that state a version's history gain
  one line each (`analysis/mod.rs`).
- **With all of it**, the scratch ran:
  - the 726 tests of lingua-core, lingua-pack, lingua-wasm and lingua-agent, which pass — 721 at
    once, and the five above once their literal or golden followed;
  - the lingua-data Python suite (459 tests), which passes: nothing it reads moves.
- **Cannot move**: no line of any golden beyond its version string, no table, and no French
  analysis beyond its version. The extension's tests use fake engines and their own spans, and none
  holds a soft hyphen.

The programme's rule is that en-fr and es-fr output does not move. Here it moves on the version
alone, with the owner's approval in this change's pull request (task 6.1).

**As implemented**, on `main` `46450468`, after changes 42c (Spanish at `1.4.0`), 49 (fr-es's tables)
and 51 (`fr-es.golden`) landed: Spanish moves `1.4.0` → `1.5.0`; the goldens move as above, line for
line, and `fr-es.golden` on 19 lines (its pack line, its `beside en-es` line and the 17 analyses),
each by its version alone; the six committed pairs are re-reduced, manifest and pin alone moving and
sizes unchanged — en-fr `582f7762…`, en-es `a95af250…` and fr-en `d013a548…` as the scratch built them,
es-fr `6a7ef217…`, es-en `2044e767…` and fr-es `3b015cd2…` (1,973,407 bytes) at the new Spanish and
French versions —, each re-stamped back to its old version giving its old pin byte for byte; the
fixtures' digests are en-fr `ebb1f3c7…`, en-es `51066821…`, es-fr `a0f0db9b…`, es-en `6868b62b…`.

### D8 — Tests

In lingua-core, beside the tokeniser's and the guards' tests:
- the spec's scenarios: each sentence is asserted as written and with its soft hyphens. The Spanish
  and French lines are first read with each soft hyphen as a space. The detector still reads them
  as Spanish or French, and the guard refuses them (checked on the prototype), so the test fails if
  the syllables stop mattering for another reason;
- `without_soft_hyphens` borrows when there is nothing to strip;
- every span of a hyphenated sentence slices the source word with its hyphens;
- a hyphenated page analyses, votes and glosses exactly as its clean text, spans aside: one test per
  language over a page of the committed corpora, hyphenated in the test at hand-chosen breaks, so
  that no hyphenation dictionary enters the repository;
- a soft hyphen opening a block, ending a word, or doubled.

In the extension (`selection.spec.ts`, `reading-session.spec.ts`):
- the snap across U+00AD;
- `sentenceAndSelection` with soft hyphens before, inside and after the selection, its offsets
  still marking the word;
- a capture's text, and a page hit's `written`, without them;
- a highlight over « vi&shy;da » covering the word, through the existing fake engine.

The existing tests pass unchanged, except the version literals of D7.

### D9 — OpenSpec

- **ADDED.**
  - *A soft hyphen is not part of a word* (`lingua-analysis`), naming no version number;
  - *A soft hyphen does not cut the reader's word* (`lingua-browser-extension`).
- **MODIFIED**: five requirements whose text names English's `1.1.0`, each on that literal alone,
  every other word and every scenario name kept.
  - *An analyser version per studied language*. Its « English's analyser version SHALL remain
    `1.1.0` through this change » would be false; it becomes the rule this change follows: a rule
    every language shares bumps every version. Its scenario *Each analysis names its own language's
    version* names English's own version.
  - *Analysis by studied language*, scenario *English output does not move*.
  - *A word's grammar, from the pack*, scenario *The page analysis does not move*. Its test
    (`engine.rs`) pins `1.1.0`.

  None of the three is held by an open change. Two more are held by `add-lingua-french-analysis`,
  which ADDS the first and MODIFIES the second:
  - *French is a studied language served by its own analysis*, scenario *Each language reports its
    own version* (« they report `1.1.0`, … »);
  - *A pack names the language it studies* (`lingua-data-packs`), scenarios *Loading the EN→FR pack
    names English* and *Versions are compared within a language*.

  So `archiveAfter` names it. The MODIFIED blocks copy its text, which `main`'s spec will hold once
  it archives.
- **Left as they are.** Scenarios that state what an earlier change did not move:
  - change 42's *English and Spanish do not move* (« English's and Spanish's analyser versions are
    unchanged »);
  - change 42b's *English and French do not move*;
  - *A Spanish rule change leaves English alone*.

  They stay true of the change they describe. This one bumps all three versions under its own
  requirement.
- `archiveAfter` also names `add-lingua-french-detection-guard` and
  `add-lingua-spanish-occitan-guard`, whose guards *A soft hyphen is not part of a word* names.
  `openspec_archive_order.py` exits 10 naming the three.

## Risks / Trade-offs

- [Words saved before the change] → a status or card a reader set on a hyphenated word is keyed by
  a dictionary form holding U+00AD (`vi‧da`). It keeps that key. After the change the page reads
  `vida`, which that status does not match: a word the reader marked « Je connais » shows as unknown
  again, unless the reader's level already counts it known, until it is marked again. A card keeps
  reviewing, its front looking the same. Only words marked or added on a hyphenated page are
  concerned. Open question 1.
- [A hyphenated neighbour language] → Galician and Catalan read as Spanish go back to the guard's
  measured leak on their clean text (21.2 % and 13.7 % of the blocks, up from 8.9 % and 9.9 % on
  hyphenated text today). Today's lower figures are an accident: the detector takes the syllables for
  Portuguese, at the price of 37 % of Spanish. Catalan read as French falls (9.4 → 7.2 %).
- [A very short block] → one that only its hyphens took over 12 bytes is now too short to detect,
  as its clean text is.
- [Other invisible characters] → U+2060 and U+FEFF are *Format* too and would behave as U+00AD did.
  No e-book tool writes them between syllables; change 42b's corpora hold U+2060 five times and
  U+200B 6,726 times, the latter a word break already. Open question 2.
- [Translation] → Bergamot's reading of U+00AD is unmeasured. D5 sends it none.
- [A whichlang or unicode-segmentation update] → the core's tests assert the scenario sentences as
  written and hyphenated. An update that changed UAX #29's class for U+00AD would fail them.
- [An installed agent] → the agent (outside the programme, M17), rebuilt at the new versions, skips
  packs of the old ones until the new ones are copied beside it, as 42b's did.

## Migration Plan

Nothing is migrated. No stored format, wire field or table row moves, and statuses, counts, backups
and sync records do not name the analyser version. Every pack is rebuilt at the extension's next
release. Rollback is a revert, with the goldens, manifests, pins and fixtures.

## Open Questions

**Settled by the owner on 2026-10-10 (in session), each as its default:** words already saved with
the hidden hyphen are left as they are, and the word joiner and U+FEFF are left alone.

For the owner, none blocking. Each has a default, which the implementation follows unless the owner
answers otherwise.

1. **Words already saved with the hidden hyphen.** Today, a reader who marks « vi‧da » « Je
   connais » on a hyphenated page saves the word as `vi‧da`, with the invisible character inside.
   After the change, the page reads `vida`, which that mark does not match. The word shows as
   unknown again, unless the reader's level already counts it known, until marked again. A card
   added from such a page keeps reviewing as before, and looks the same. Leave such records as they
   are (the default: they arise only from a hyphenated page, and nothing is lost), or clean them
   once? Cleaning means: remove U+00AD from every saved word, merge two marks of one word (the most
   recent wins, as sync already decides) and two cards of one word, on the device and on the
   server — a change of its own, touching the backup, the sync and the server.
2. **Other invisible characters.** The word joiner (U+2060) and the zero-width no-break space
   (U+FEFF) hide in a word as the soft hyphen does, and the core would read them the way it read
   « vi‧da » before this change. No e-book tool puts them between syllables, and the corpora hold
   U+2060 five times. The zero-width space (U+200B) already ends a word. Leave them (the default),
   or read U+2060 and U+FEFF as this change reads U+00AD, which bumps no further version if done
   here?
