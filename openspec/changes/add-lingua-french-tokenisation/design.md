# Design — add-lingua-french-tokenisation

## Context

See proposal.md (Why). Where French is cut into words today (`origin/main` 03bdb0d3, change 39
merged):

| Seam | Today |
|---|---|
| `analysis/tokenize.rs` `tokenize` | UAX #29 words (`unicode_word_indices`), then a run of pieces joined by exactly one `-` is a hyphenated compound (`push_compound`), every other word goes through `push_word` |
| `push_word` | `’` → `'`, edge apostrophes trimmed, the digit drop, `nfc_for` (Spanish alone), `split_contraction` (English `n't`, Spanish `al`/`del`, French `None`), the single-letter rule; the two halves of a contraction share the word's span, the base takes the written word's capital |
| `push_compound` | one token for the whole run, its pieces as `parts`; `resolve_lemmas` (`pipeline.rs`) keeps it whole when the pack lists the run, else lemmatises the parts and the classifier judges the run by its weakest part |
| `Token` | `text`, `start`, `end` (bytes into the block), `parts`; the doc says contraction halves share a span |
| `analysis/mod.rs` | `FRENCH_ANALYZER_VERSION = "0.1.0"` |
| `engine.rs` `word_grammar` | reads the written word through `tokenize`; reports `pieces` when the pre-pass made more than one token |
| `engine.rs` `gloss_phrase` | one row per token; expressions matched on the tokens' lemmas joined by spaces |
| `crates/lingua-pack` `expression_key` | an expression's key is its whitespace-separated words, each lemmatised, every one a lexicon lemma or the expression is dropped |
| `apps/lingua-extension/src/reading/scan.ts` | `rangeForToken` turns a token's byte span into a DOM range; `findTokenAt` returns the first range whose closed interval holds the caret |
| `crates/lingua-wasm/tests/french_baseline.rs` | the golden over 13 pages, a hand-written fr-en fixture pack, es-en beside; asserts `l'homme` one token, `au` whole, `0.1.0` |

The owner's decision M21 (2026-10-09): **au/aux split, du/des whole; one highlight span per
elision piece; « pas » a function word; moods merged on five-reading forms.**

All figures below were measured on a scratch copy of the workspace (never committed) carrying a
prototype of this design, against change 39's corpus and fixture: a dump of every token of the
corpus before and after, each rule switched on alone and each switched off alone, and the French
golden re-blessed.

## Goals / Non-Goals

**Goals:**
- French's own pre-pass, at `0.2.0`: the pieces a French word is made of read as words, each piece
  of an elision with its own span, a click landing on the piece under the pointer.
- The golden's diff says what moved, rule by rule; English and Spanish do not move.

**Non-Goals:**
- Lemmatisation: the cascade, NFC, the closed classes (« pas » among them), the names rule — change
  41. The pieces keep the baseline's lemmatisation until then.
- The moods merged on a five-reading form — the French word card, change 51 (D1).
- Expression keys built through the analyser — change 44 (D8).
- Which piece a one-word selection opens — change 51 (D7, open question 2).
- The demonstrative particles `-ci` and `-là`, the other apostrophe-like characters (U+02BC,
  U+2018, U+FF07), the 1990 spellings, the Catalan and Occitan guard (42), the forms tables (43).

## Decisions

### D1 — What M21 puts here, and what it leaves to changes 41 and 51

M21 has four parts. Two are tokenisation and land here: `au`/`aux` split with `du`/`des` whole
(D4), and one highlight span per elision piece (D3, D6, D7). The two others are not:

- **« pas » a function word** is a line of French's closed classes. The tables are checked by
  lemma, after lemmatisation, in `analysis/function_words.rs`, and change 39 gives « the cascade,
  the function words, the names rule and NFC » to change 41 (the study's row for 41 reads
  « cascade, mots-outils, noms »). M21 decides that `pas` is flagged as the negation although `pas`
  is also a noun (« a step ») — under « one form, one lemma » (M8) the noun's row is lost — and
  change 41 writes it with `ne`. This change writes no function-word table: change 39's scenario
  *No French word is a function word yet* stays true at `0.2.0`, and the pieces this change
  produces (`le`, `de`, `que`, `ne`) are rows of a word-by-word gloss until 41.
- **Moods merged on five-reading forms** is about the word card, not about cutting text or
  choosing a lemma. A first-group present such as « parle » has five readings: present indicative
  first and third person singular, present subjunctive first and third person singular, imperative
  second person singular. The core answers readings as tags, unmerged (`word_grammar.readings`,
  from the grammar tables of change 45); the card's description merges them
  (`apps/lingua-extension/src/reading/grammar-description.ts`: readings merged by tag, persons
  merged by tense). Spanish's card does not merge moods — « hable » reads « first- and
  third-person singular present subjunctive and third-person singular imperative of hablar »
  (`test/baseline/word-card-es-en.txt`). Saying « present indicative or subjunctive » once is a
  rule of the French card's description, change 51 (« formes françaises nommées en anglais et en
  espagnol »), on change 45's readings. Not change 41: the cascade picks one lemma per form and
  never sees a reading.

### D2 — The narrow no-break space separates French words

UAX #29 gives U+202F NARROW NO-BREAK SPACE the word-break class ExtendNumLet (rules WB13a/b, for
Mongolian), so `«\u{202F}C’est` and `fini\u{202F}!` come out of `unicode_word_indices` as one word
with the space inside. French typography sets U+202F before `?`, `!`, `;` and `»` and after `«`.
Measured on the corpus (the `fiction` and `proust` pages, set as French is printed):

- 12 tokens carry U+202F at an edge: `demain `, `pas `, ` C'est`, `fini `, ` Rentrez`, `tous `,
  ` Qu’est-ce`, `attend `, ` Je`, `m'éveillait `, `lumière `, `particulier `. None is a pack form;
  ` Je` and ` C'est` keep the space in their lemma, and the elision cannot see ` C'` or ` Qu’`.
- No token carries U+00A0, which the corpus sets before `:` and after the dialogue dash: it is not
  ExtendNumLet.

French's arm cuts a word at every U+202F before any other rule reads it; each part keeps its own
byte span, the space in none. Measured: the 12 tokens re-cut, the token count unchanged, and with
the elision two more words split (` C'est`, ` Qu’est-ce`).

English and Spanish keep the behaviour: their output must not move, and U+202F is French
typography. A page of theirs set with it has the same defect (open question 3).

*Rejected — trimming U+202F at a token's edges only.* A word with U+202F inside would stay glued;
cutting is the same code and covers both. *Rejected — NFKC.* It maps U+202F to a space but also
folds letters (ligatures, superscripts), which is a normalisation question for change 41, and
NFC, change 41's choice, leaves U+202F alone.

### D3 — Elision: a closed list, the word it stands for, a span per piece

**The rule.** In a French word (after D2), when the part before the first apostrophe — straight
`'` or typographic `’` — is one of the elided forms below, followed by a letter, the part up to and
including the apostrophe is a token of its own, and the rule runs again on what follows:

| Elided | Read as |
|---|---|
| `c'` | `ce` |
| `ç'` | `ça` |
| `d'` | `de` |
| `j'` | `je` |
| `l'` | `le` |
| `m'` | `me`; `moi` right after a hyphen (`donne-m'en`) |
| `n'` | `ne` |
| `qu'` | `que` |
| `s'` | `si` before `il` and `ils`; `se` otherwise |
| `t'` | `te`; `toi` right after a hyphen (`va-t'en`) |
| `jusqu'` | `jusque` |
| `lorsqu'` | `lorsque` |
| `puisqu'` | `puisque` |
| `quoiqu'` | `quoique` |

Matching is on the lowercased part; the read word takes the written first letter's capital on its
own first letter (`L'` → `Le`, `Qu’` → `Que`). After the last elision, what remains goes on through
`au`/`aux` (D4) and `push_word` as any word does: `jusqu'au` is `jusque` + `à` + `le`,
`qu'aujourd'hui` is `que` + `aujourd'hui`.

**What stays whole.** A word whose part before the apostrophe is not on the list: `aujourd'hui`
(`aujourd`), `presqu'île` (`presqu`), `quelqu'un` (`quelqu`), `prud'homme`, `entr'ouvert`,
`grand'mère`. `presque` and `quelque` elide only in these lexicalised words, so they are left off
the list on purpose. A plain word is split whatever the pack lists, as `al`, `del` and `don't`
are: `c'est` and `d'abord` are entries of the Wiktionary, and keeping them whole when the forms
tables list them would hide `être` in the most frequent sentence of the language. Inside a
hyphenated run the pack's listing decides first (D5).

**The read word is the token's text.** English writes `do` + `not` for `don't`, Spanish `de` + `el`
for `del`; French writes `le` for `l'`. The pack then resolves a dictionary word, the closed classes
of change 41 and the expression keys of change 44 read dictionary words, and the proper-noun rule
sees a word the pack lists. Measured at `0.1.0`: three sentence-initial elisions (`L'homme`,
`Lorsqu'il`, `D'abord`) were set aside as names, being capitalised and outside the lexicon; at
`0.2.0` `Le`, `Lorsque` and `De` are pack words.

*Rejected — keeping `l'` as the token's text and letting the pack resolve it.* It works only if
change 43's tables list every elided form; a capitalised `L'` the pack does not list is still a
« name »; and change 41's tables would need `l'`, `d'`, `qu'` beside `le`, `de`, `que`.
*Rejected — `la` for `l'` before a feminine noun.* It needs the next word's gender in the
tokeniser, and under « one form, one lemma » `la` is a form of `le` anyway.

**Known limits.** Informal `t'es` (« tu es ») reads `te` + `es`; `m'`/`t'` read `moi`/`toi` only
right after a hyphen. Both are measured in the corpus (`informel`, `inversions`) and shown in the
golden.

**Apostrophes.** `'` and `’`, the two the corpus holds (48 and 15 of its 63 elisions) and the ones
French keyboards and publishers produce; `push_word` already maps `’` to `'` for every language.
U+02BC MODIFIER LETTER APOSTROPHE (a letter for UAX #29), U+2018 and U+FF07 are not read as
apostrophes — none occurs in the corpus — so `lʼhomme` stays one token; a later rule would bump
French's version alone.

**An elided word on its own.** An elided form directly followed by an apostrophe and then by
neither a letter nor a digit — the end of the text, a space — is the elided piece, its span holding
the apostrophe. It moves no token of the corpus. It exists for the word card: the extension hands
`word_grammar` the text of the clicked range (`session.ts`, `written: hit.range.toString()`),
which for an elided piece is `l’` alone; UAX #29 leaves an apostrophe at a word's edge out, and the
`l` that remains would fall to the single-letter rule. With it, the card on `l’` reads `le`; the
golden's new probe `word-grammar l’ le` records it. It also reads `l’ homme`, typed with a space.

### D4 — `au` and `aux` split, `du` and `des` whole (M21)

`au` → `à` + `le`, `aux` → `à` + `les`, the two tokens sharing the word's span (two letters cannot
be shared out), as Spanish's `al`/`del`. `du` (de + le, or the partitive article) and `des` (de +
les, or the plural of `un`) stay whole: the tokeniser cannot tell their two readings apart, and
« one form, one lemma » keeps the pack's lemma for each. `auquel`, `auxquels`, `duquel`,
`desquels` are other words and stay whole; nothing is split inside a hyphenated compound
(`au-dessus`, `au-delà`), but a remainder after an elision is (`jusqu'au`).

**The capital.** `Au` → `À`, `AUX` → `À` + `les`. The casing code English and Spanish share keeps
the written first letter's case by slicing the base at that letter's byte length — `D`/`d`,
`A`/`a`, always one byte. `à` is two bytes where `A` is one: French uppercases the read word's own
first letter instead, and a test holds `Au` and `AU`.

Measured: 9 words (`au` ×7, one of them in `jusqu'au`; `aux` ×2) become 18 tokens.

### D5 — Hyphenated inversions are read as words

A hyphenated run (exactly one `-` between pieces, the language-neutral rule) is read, for French,
in this order:

1. a digit anywhere → each piece on its own, as today;
2. the whole run listed by the pack → one token, as today (`peut-être`, `rendez-vous`, and
   `c'est-à-dire` if the tables list it);
3. an elision on the first piece is split off (D3), and the rest is read again from step 2:
   `l'arc-en-ciel` → `le` + `arc-en-ciel` (listed), `Qu’est-ce` → `Que` + `est-ce`;
4. when every piece after the first is a pronoun written in lowercase — `je`, `tu`, `il`, `elle`,
   `on`, `nous`, `vous`, `ils`, `elles`, `ce`, `le`, `la`, `les`, `lui`, `leur`, `moi`, `toi`, `y`,
   `en`, or an elided `m'`, `t'`, `l'` before `en` or `y` — with the euphonic `t` allowed right
   before `il`, `elle`, `on`, `ils` or `elles`: each piece is a token with its own span; the `t`
   and the hyphens belong to no token;
5. otherwise the compound rule, as today.

The first piece of an inversion goes through D3 and D4 like any word (`Qu’est` was split at step 3);
an elided pronoun of the tail reads `moi`/`toi`/`le` (D3).

Measured on the corpus: 12 runs become 25 tokens — `demanda-t-elle`, `Qu’est-ce`, `dit-il`,
`a-t-il`, `Est-ce`, `Va-t'en`, `Donne-m'en`, `allez-vous-en`, `pense-t-elle`, `Viendront-ils`,
`coupez-les`, `versez-les` — the euphonic `t` dropped three times. Kept: the four runs the fixture
lists whole (`peut-être`, `rendez-vous`, `arc-en-ciel`, `porte-monnaie`) and the compounds whose
tail is no pronoun (`demi-heure`, `moi-même`, `Celui-ci`, `celle-là`, `jour-là`, the names
`Charles-François-Bienvenu`, `Jean-Pierre`, `Saint-Étienne`, `Marie-Claire`, and the Catalan
`penya-segat` of the mixed page).

**Lowercase tails.** A name can end in a word that is also a pronoun (`Saint-Y`); an inversion's
pronouns are lowercase except in text set in capitals, which falls back to the compound rule.

*Rejected — splitting every hyphenated run.* Names and compounds are what the compound rule is for.
*Rejected — a table of inverted forms.* Every verb form times every pronoun sequence. *Rejected —
splitting `-ci` and `-là`.* `celui-ci` and `celle-là` are pronouns of their own, which change 43's
tables list (« composés à trait d'union »); `jour-là` stays a compound judged by its parts.

**The risk is a compound with a pronoun tail that the pack does not list** (`rendez-vous`,
`chez-moi`, `m'as-tu-vu`, `qu'en-dira-t-on`): it would be split. Change 43 carries the check — every
hyphenated entry of the French section whose last piece is one of the pronouns above is in its
forms table — measured on the real tables there.

### D6 — Spans: the `Token` model does not change

The study expected the model to change for elision. It does not need to: every `Token` already
carries its own `start`/`end`, and nothing downstream assumes that the tokens of one written word
share a span. Only `Token`'s doc comment moves: the halves of a contraction (`don't`, `del`, `au`)
share the word's span; an elision's pieces and an inversion's words each have their own.

| Written | Tokens and spans (bytes) |
|---|---|
| `L'homme` | `Le` [0, 2) `L'`, `homme` [2, 7) |
| `l’horizon` | `le` [0, 4) `l’` (the typographic apostrophe is three bytes), `horizon` [4, 11) |
| `jusqu'au` | `jusque` [0, 7) `jusqu'`, `à` and `le` [7, 9) `au` |
| `a-t-il` | `a` [0, 1), `il` [4, 6) |
| `«\u{202F}C’est` | `Ce` [5, 9) `C’`, `est` [9, 12) — `«` and U+202F in no token |

Consumers, all unchanged: the extension converts each span to a DOM range (`rangeForToken`, byte
offsets to characters — a span inside a word converts like any other) and paints it; `word_grammar`
reports pieces when the written word holds more than one token (`l'homme` → `le`, `homme`;
`au` → `à`, `le`); `gloss_phrase` gives one row per token. Measured: 63 elided pieces in the
corpus, 15 of them with the typographic apostrophe.

Determinism: the pre-pass is pure Rust over the text, so native and wasm agree by construction; a
French page under `wasm-pack test --node` is held to the host's tokens and spans (task 2.6), the
first French check on the wasm target with multi-byte spans inside a word.

### D7 — The extension: a click between two pieces opens the piece that starts there

Separate spans meet at one offset (`l’|homme`). `findTokenAt` (`reading/scan.ts`) returns the first
resolved range whose closed interval holds the caret, so a click on the left half of `homme`'s
first letter — the caret lands before it — opens `le`. It becomes half-open: a range that starts at
the caret, or holds it strictly, wins over one that merely ends there, which is kept as a fallback
(a click at the very end of a word still opens it). Identical spans (`don't`, `del`, `au`) give the
first token as today, and a single range answers as today, so English and Spanish clicks do not
move. The test feeds synthetic tokens; no French pack reaches the extension before change 52.

Painting is unchanged: two adjacent ranges of one class are painted in one colour and read as one
highlight, each clickable apart. Whether a hairline should separate them is a dogfood question for
change 52.

**Left to change 51 — a one-word selection.** A selection without whitespace is a « word »
(`selection.ts`), resolved at the selection's start (`session.ts` `onCapture`): a double-click on
`l’homme`, which browsers select whole, opens `le`. Which piece a selection over several pieces
opens is a choice of the card (open question 2).

### D8 — What the interval between 40 and 44 costs: expression keys holding `au` or `aux`

`expression_key` (`crates/lingua-pack`) keys an expression by its whitespace-separated words, each
lemmatised, and drops it when one is not a lexicon lemma. At `0.1.0` the selection « Au revoir »
read `au` + `revoir` and matched the key `au revoir`; at `0.2.0` it reads `À` + `le` + `revoir`
and the key is out of reach until change 44 builds keys through the analyser. Keys holding an
elision (`coup d'œil`: `d'œil` is no lemma) are already dropped at build — the study's 9.6 % of
French expressions — so this change adds the keys holding `au`/`aux` to those 44 repairs, as the
596 es-fr keys holding `al`/`del` were for Spanish. Nothing ships in between.

To make 44's effect visible, the fixture gains the expressions `au revoir` (« goodbye ») and
`coup d'œil` (« glance ») and the scenario two phrase probes, « Au revoir » and « un coup d’œil »:
both answer no expression at `0.2.0` (measured; « Au revoir » matched with the `0.1.0`
tokeniser), and change 44's re-bless shows both appear.

### D9 — `0.2.0`, the fixture, the probes

`FRENCH_ANALYZER_VERSION = "0.2.0"`, the version change 39's D4 reserved; `1.0.0` stays change 41's.
The fixture's manifest is re-stamped. Its forms gain the words the pre-pass writes that it lacked —
`te`, `moi`, `toi`, `ça`, `jusque`, `lorsque`, `puisque`, `quoique` — and `revoir`, `coup` for D8's
expressions: measured, without them `Lorsque` at the head of a sentence was set aside as a name. A
test holds that the fixture lists every word the pre-pass can write; change 43 runs the same check
on the real tables.

The scenario (`support/french.rs`) gains five probes: the phrases « S’il pleut, viendras-tu ? »
(set with U+202F: `si`, an inversion with `tu`, D2), « Au revoir » and « un coup d’œil » (D8), and
the word grammars `l’` for `le` (D3's elided word on its own) and `au` for `à` (D4's pieces).

### D10 — What the golden shows, and what does not move

Per rule, on the 917 tokens of the corpus's French blocks (each rule on alone; then each switched off
from the whole, which counts what it adds beside the others):

| Rule | Alone: tokens replaced → new | Its share of the whole |
|---|---|---|
| U+202F a space (D2) | 12 → 12 | +2 |
| Elision (D3) | 59 → 118 | +64 |
| `au`/`aux` (D4) | 8 → 16 | +9 |
| Inversions (D5) | 12 → 25 | +15 |
| An elided word on its own (D3) | 0 | 0 |
| All | 89 → 174 | **917 → 1,002 tokens** |

Page by page, for the new reader:

| Page | Tokens | Counted | Glossed | Set aside as names |
|---|---|---|---|---|
| actualites | 66 → 72 | 64 → 69 | 23 → 30 | 2 → 3 |
| fiction | 80 → 91 | 79 → 89 | 29 → 39 | 1 → 2 |
| proust | 112 → 119 | 106 → 113 | 27 → 32 | 6 → 6 |
| homographes | 116 → 120 | 116 → 120 | 56 → 62 | 0 → 0 |
| elisions | 79 → 101 | 76 → 101 | 27 → 44 | 3 → 0 |
| contractions | 55 → 60 | 55 → 60 | 27 → 34 | 0 → 0 |
| inversions | 60 → 73 | 58 → 71 | 17 → 25 | 2 → 2 |
| noms | 68 → 71 | 45 → 47 | 17 → 21 | 23 → 24 |
| informel | 35 → 37 | 35 → 37 | 12 → 13 | 0 → 0 |
| recette | 55 → 59 | 53 → 57 | 16 → 20 | 2 → 2 |
| technique | 60 → 62 | 58 → 59 | 20 → 22 | 2 → 3 |
| mixte | 47 → 49 | 46 → 48 | 17 → 19 | 1 → 1 |
| grammaire | 84 → 88 | 83 → 87 | 25 → 26 | 1 → 1 |
| **total** | **917 → 1,002** | **874 → 958** | **313 → 387** | **43 → 44** |

Names: `L'homme`, `Lorsqu'il`, `D'abord` and `Donne-m'en` are no longer set aside; `INSEE`, `API`
and `Aube` are, once their `l'` is split off (`l'INSEE`, `l'API`, `l'Aube`, the river); so are
`Rentrez`, no longer glued to U+202F, and `Donne`, both capitalised verbs the fixture does not list
(the rule English has too; the real tables list them). For the reader with a history, the known
share of the four reader pages: actualites 31 → 41 %, homographes 38 → 42 %, elisions 37 → 49 %,
mixte 37 → 40 % — the common words the reader knows (`le`, `que`, `ne`) now read as such.

**The golden's diff**: 26 of its 136 probes move, 5 are added, 110 are byte for byte.
- `pack`: `analyzer_version "0.2.0"`, 4,948 → 5,139 bytes (D9).
- The 13 `analyse new-reader` pages and the 4 `analyse reader` pages.
- Six phrase glosses: « l'homme qu'il attendait », « jusqu'au soir », « au marché », « dit-il »,
  « Y a-t-il encore du café », and « El far s'alçava » — the Catalan selection, glossed word by
  word as French (`s'` → `se`); a selection has no language detection.
- Two word grammars: `au` for `au` (pieces `à`, `le`) and `l'homme` for `homme` (pieces `le`,
  `homme`).
- Added: the five probes of D9.
- Unmoved: every `gloss` probe, « Aujourd'hui », « du pain et des œufs », the Proust sentence,
  every other phrase and word grammar, the levels, the ladder, the review, the exports and the
  backup.

**What cannot move.** The pre-pass is French's arm of `tokenize`; English, Spanish and every pair
studying them never reach it. Measured on the prototype, without re-blessing: `english_baseline`
(en-fr), `spanish_baseline` (es-fr), `es_en_baseline`, `en_es_baseline`, `cross_native` and
`parity` pass, 25 tests in 6 suites. The pack builder's French path calls `lemmatize` only, so no
committed table or pin moves.

**Tests that flip at `0.2.0`**, by design, each rewritten with the rule it now states:
`analysis::language::tests::english_keeps_its_analyser_version_and_spanish_has_its_own`,
`analysis::tokenize::tests::{spec_scenario_an_elided_french_word_is_one_token,
spec_scenario_french_contracted_articles_are_whole,
french_inversions_and_compounds_follow_the_compound_rule}`,
`engine::tests::spec_scenario_a_french_page_is_read_by_the_baseline`,
`packs::pack::tests::spec_scenario_a_french_pack_at_french_s_analyser_version` (lingua-core);
`tests::a_french_pack_builds_at_french_s_analyser_version_and_loads` (lingua-pack, which stamps a
literal `0.1.0`); `spec_scenario_a_french_reader_s_backup_is_version_3` (`languages.rs`), and
`french_is_the_baseline_until_its_rules_are_written` and
`a_fixture_left_behind_its_analyser_names_its_manifest` (`french_baseline.rs`) (lingua-wasm);
`test/packs.spec.ts`, which reads `0.1.0` for French (the extension).

OpenSpec: two ADDED requirements in `lingua-analysis` and one in `lingua-browser-extension`, held by
no open change; two MODIFIED requirements in `lingua-analysis`, both added by
`add-lingua-french-baseline`, which is merged and not archived — hence `archiveAfter`, and
`openspec_archive_order.py` exits 10 naming it alone. Each MODIFIED block is change 39's text with
what moves at `0.2.0` rewritten, every requirement and scenario name kept:
- *French is a studied language served by the baseline analysis*: French's own pre-pass is named
  beside the rules that belong to no language, « no contraction or elision split » goes, the
  version is `0.2.0`; the scenario *An elided word is one token* now holds the words whose elision
  is part of them, *Contracted articles are whole* holds `du` and `des`, *Each language reports its
  own version* reads `0.2.0`; the four other scenarios are unchanged.
- *A French invariance baseline runs beside the English and Spanish ones*: the scenario *What the
  baseline shows today* says what it shows at `0.2.0`; the requirement and its five other scenarios
  are unchanged.

A later change that rewrites either requirement (change 41's cascade, and change 42 if its guard
touches it) lists this one in its `archiveAfter` and carries this wording.

## Risks / Trade-offs

- [A compound with a pronoun tail that the pack does not list is split — `rendez-vous`] → The
  pack's listing is read first (D5 step 2); change 43 checks every such entry is listed.
- [A name ending in a pronoun] → Pronouns must be lowercase (`Saint-Y` stays whole).
- [`t'es` reads `te` + `es`, `l'` always `le`] → Measured and shown in the golden; `la` would need
  the next word's gender, and its lemma is `le` under M8.
- [Expression keys holding `au`/`aux` are out of reach until change 44] → Nothing ships before;
  the fixture's `au revoir` shows the repair in 44's diff (D8).
- [Two adjacent pieces of one class read as one highlight] → Each is clickable apart (D7); a
  visual separation is a dogfood question for change 52.
- [A double-click on `l’homme` opens `le`] → Left to change 51 with the card (open question 2);
  no French reaches the extension before 52.
- [`Au` → `À` slices a two-byte letter] → French uppercases the read word's own first letter (D4);
  a test holds `Au`, `AU`, `AUX`.
- [A rule of French's runs on English or Spanish] → The pre-pass is French's arm; the four other
  goldens pass without re-blessing (D10).

## Migration Plan

Nothing to migrate: no stored format, wire field, pack byte or pin moves, and no reader studies
French. A French pack must carry `0.2.0`; only the fixture exists. Rollback is a revert.

## Open Questions

For the owner, none blocking:
1. **Placements of M21's last two parts** (D1): « pas » as a function word in change 41 with the
   closed classes, the moods merged on « parle » in change 51 with the card. Either can be pulled
   forward, at the cost of a partial table or a card rule written before its readings exist.
2. **A one-word selection over several pieces** (D7): a double-click on « l’homme » selects the
   whole word, and the card opens on its first piece, `le`. Recommended for change 51: such a
   selection opens the whole-selection card, whose rows list the pieces — with change 41's closed
   classes left out, « l’homme » shows `homme` and « dit-il » shows `dit`. The other choice, a
   word card on the first piece that is neither elided nor a pronoun, needs the pieces' kinds in
   the extension.
3. **U+202F in English and Spanish text** (D2): the same glue happens there; fixing it would bump
   their analyser versions and re-bless their goldens, in a change of its own if wanted.
