# Design — add-lingua-french-analysis

## Context

See proposal.md (Why). Where French is analysed on `main` (35faf774, change 40 merged as #821):

| Seam | Today |
|---|---|
| `analysis/tokenize.rs` | French's own arm (change 40): U+202F a space, `FRENCH_ELISIONS` matched on the lowercased written piece (`french_elided_word`), `au`/`aux` split, `listed_whole` before an inversion; `nfc_for` returns French text as it came, « NFC being a rule of its cascade » (its doc) |
| `analysis/lemmatize.rs` | `lemmatize` dispatches French to `lemmatize_baseline`: the pack's lemma for the lowercased form, else the form |
| `analysis/function_words.rs` | `is_function_word`: English's and Spanish's six tables; French `&[]` |
| `engine.rs` | `analyse_page` computes `document_names` for Spanish alone; `mid_sentence` (after a letter, a digit, a comma or a semicolon); the override applies to a token without parts that the knowledge model reads `Unknown` |
| `analysis/percent.rs` | the shared proper-noun rules: a capitalised surface whose form and lemma resolve nowhere; a capitalised compound none of whose parts resolves |
| `packs/pack.rs` | `is_dictionary_word`: the pack's lexical section (`tables/<studied>/lexical.tsv`, change 5), else « has a gloss » for a pack without one (the fixture) |
| `analysis/mod.rs` | `FRENCH_ANALYZER_VERSION = "0.2.0"` |
| `crates/lingua-pack` `measure.rs` | `lingua-pack-measure` lemmatises each treebank word through `lemmatize` with the pack's studied language: it measures this cascade, not the tokeniser |
| `crates/lingua-wasm/tests/french_baseline.rs` | the golden over change 39's 13 pages and the hand-written fr-en fixture (296 forms), es-en beside; `the_nfd_block_s_memoire_is_glossed_once_french_composes_it` waits for this change |

The owner's decisions that bind it (2026-10-09): **M8**, one form, one lemma, in every language —
the tables decide, lemma alternatives stay the optional `add-lingua-lemma-alternatives`; **M21**,
« pas » a function word, as change 40's D1 hands it here.

**How it was measured** (scratch, never committed): a prototype of this design on a copy of change
40's implementation, whose tokeniser, golden and fixture are byte for byte `main`'s; each rule
switchable, so that each was measured alone and switched off from the whole. Three inputs:
- change 39's corpus and fixture, through the French golden (D7);
- UD French-PUD (20,232 measured words) and GSD's test section (8,049), through
  `lingua-pack-measure`, with a pack built from change 43's prototype tables (124,040 forms,
  60,000 lemmas; its D8's final cut), stamped at the prototype's version;
- **a raw-text corpus** of 43,494 French paragraphs and sentences, 864,079 tokens once French
  blocks are kept: four Gutenberg novels (Swiss, Belgian and two Québec authors; 225,774 tokens),
  French Wikipedia articles (364,775), 20,000 Tatoeba sentences (156,533), and the raw sentences
  of UD's seven French treebanks (116,997) — the text change 42 gathered for its guard. It runs
  through the page analysis (tokeniser, cascade, names), with change 43's tables and, for the
  names rule, a stand-in for the dictionary words change 48 will commit: the 25,872 ranked lemmas
  the English Wiktionary's French section gives a sense that is neither a form nor a spelling of
  another word, names left out (D4).

## Goals / Non-Goals

**Goals:**
- French text read as the pack writes it, whatever its Unicode form.
- A lemma for every French token, the tables deciding each one (M8), and unlisted plurals counted
  once.
- A word-by-word gloss in French without its closed-class rows, « pas » among them (M21).
- A French document's names out of the count, as Spanish's are.
- French leaves the baseline's `0.x` versions; English and Spanish do not move.

**Non-Goals:**
- Choosing between a form's readings: the tables do (change 43, M8).
- Expression keys built through the analyser (44), the readings and the moods merged on a
  five-reading form (45, 51), the Catalan and Occitan guard (42).
- Lowercase text written without its accents (`apres`, `ca`, `deja`), pre-1835 spellings
  (`étoit`, `connoître`), the Gutenberg underscore markup (`_il_`, which UAX #29 glues, as for
  every language).

## Decisions

### D1 — NFC is a rule of French's pre-pass, not of its cascade

French's arm composes every word it reads, before anything compares it: the elided piece looked up
in `FRENCH_ELISIONS`, the `au`/`aux` check, the run `listed_whole` looks up, and each token's text
(`nfc_for` answers NFC for French, as for Spanish). Spans stay the source's: a token's `start` and
`end` cover the decomposed letters and their combining marks, so the extension paints and clicks
what the page holds.

*Rejected — NFC in the cascade only*, as change 40's doc comment had it pending. The pre-pass
matches written pieces against its tables before any lemma is sought: a decomposed `ç'`
(`c` + U+0327) is not `ç` and is never split, a decomposed `peut-être` is not the run the pack
lists and goes to the inversion and compound rules. Checked on the prototype: with the pre-pass
composing, `ç'a été peut-être` gives `ça` [0, 4) — the apostrophe and the
combining cedilla in its span —, `a`, `été` and the listed `peut-être`. *Rejected — NFKC*: it
folds ligatures, superscripts and U+202F, which change 40 reads already.

Measured: none of the 43,494 raw paragraphs and none of PUD's or GSD's test words is decomposed, so NFC moves no
token there; it moves the golden's NFD block (D7), which change 39 committed for this change. It is
there for what is: text typed through a dead-key layout that composes late, pasted from a terminal,
an OCR or a macOS file name.

### D2 — The cascade: the pack's forms, an unlisted plural, the form

For each French token, lowercased (`’` read as `'`) and in NFC:
1. **The pack's forms** (`lemma_of`): the lemma change 43's tables chose — one per form, by its
   override list, GSD's counts, the form's own entry, frequency, order (its D5, M8).
2. **An unlisted plural**, only for a form written in lowercase that the pack does not hold at
   all, and only when its singular is not in the pack either: `-eaux` → `-eau`, any other `-aux` →
   `-al`, `-s` after a letter other than `s` → without it. Left alone: forms of four letters or fewer,
   singulars in `-us`, `-is`, `-ès`, `-os`, a hyphenated or elided word, and the passé simple's
   `-âmes`, `-îmes`, `-ûmes`, `-âtes`, `-îtes`, `-ûtes`.
3. **The form itself.**

The order is the determinism contract; changing it bumps French's version.

**The tables decide (M8).** Step 2 never reads a form as a word the pack lists: its singular must be
unknown too. So change 43's own choice stands — `étés`, plural of the noun *été*, whose singular
reads as *être*, stays out (its D3, « an unknown word rather than the verb ») —, and so do
`vivants` (*vivant* reads as *vivre*) and `sorts` (*sort* reads as *sortir*): each stays itself, an
unknown word. The cascade never picks between a form's readings; lemma alternatives stay the
optional `add-lingua-lemma-alternatives`.

**Why lowercase only.** On the raw corpus 913 of the rule's 2,313 hits were capitalised, every one
a name or a people already set aside as a proper noun (`Lluís` → `lluí`, `Niaux` → `nial`,
`Wisigoths`): the rule only renamed them. Lowercase, it moves 1,400 tokens (0.16 %), 965 distinct
words: `belgicismes`, `félibres`, `patoisants`, `comarques`, `alluvions`, `ramures` — and leaves 63
passé simple forms (`dormîmes`, `cessâmes`) alone, which `-s` alone would have cut to `dormîme`.
`-eaux` is read before `-aux` (`perdreaux` → `perdreau`, not `perdreal`: 8 raw tokens). `-eux` and
`-oux` are not stripped: an unlisted `-eux` is far likelier an adjective
(`sablonneux`) than the plural of an `-eu` noun, all of which the tables list.

**On UD**, step 2 changes 18 PUD words and 8 GSD test words; their content lemmas go from 96.38 %
to 96.48 % (PUD) and from 95.65 % to 95.81 % (GSD test), ten and six words taking the treebank's
lemma (`mégalithes`, `vicissitudes`, `auspices`), none losing it. Resolution cannot move, by
construction (the singular is outside the lexicon too): 99.12 % and 98.91 %. Auxiliaries: 99.90 %
and 99.72 %, unchanged.

**Measured and rejected:**

| Rule | Raw corpus | UD | Why not |
|---|---|---|---|
| A capital without its accent (`Ecole` → `école`, `Etat` → `état`) | 3 tokens (13 more, `Etat`, `Etats`, `Ecole`, are then set aside by the names rule, D4) | none in PUD; 5 words in GSD's 400,000 | edited French sets its accents on capitals |
| `oe` read as `œ` (`manoeuvre`) | 3 tokens | 0 | change 43 maps the dictionary's ASCII spellings (`coeur`, its D7) |
| A plural or feminine read through its listed singular (`vivantes` → *vivre*) | 1,102 tokens: `servante` → *servir*, `subite` → *subir*, `étés` → *être*, `surplombe` → *surplomb* | PUD 99.12 → 99.23 % resolved, content 96.48 → 96.51 % | it undoes change 43's D3 (*A noun's plural is not its homograph's verb*) and M8's choice, word by word |
| Verb endings checked against the lexicon (`promenèrent` → *promener*) | 458 tokens (0.056 %), with `silve` → *silver*, `enfe` → *enfer*, `meurtrie` → *meurtrier* | — | guesswork where the tables are exact; the literary forms belong in them (change 43's « every form » cut, its D8) |
| A capital `A` read as `à` | 169 standalone `A`, 110 of them before a lowercase word (`A présent`, `A la`); 610 `À` | GSD and PUD read 96 standalone `A` as `à`, 2 as a verb | `a` is listed (*avoir*), and an inversion's `A` (`A-t-il`) is *avoir*: the tokeniser's knowledge, not the cascade's; both readings are among the commonest words and both are function words (D3) |

*Rejected — Spanish's cascade.* Its accent retry reads the spellings the 2010 rules retired, which
French has no equivalent of; enclitics are hyphenated in French and split by change 40.

`lemmatize_baseline`, which change 39 brought back for French, serves no language any more and
goes, as it went when Spanish got its cascade (#655).

### D3 — French's closed classes

Six sorted tables, as English's and Spanish's, checked on the lemma the cascade produced; they set
only the phrase gloss's `function_word` flag, which the selection card reads to leave a row out
(`selection-card.ts`). Counts, statuses and the page analysis do not read them.

- **Determiners**: `le`, `un`, `du`, `des`, `ce`, `cet`, the possessives (`mon`, `ma`, `mes`,
  `ton`, `ta`, `tes`, `son`, `notre`, `votre`, `leur`), `quel`, `aucun`, `nul`, `chaque`,
  `plusieurs`, `quelque`, `tout`, `même`, `autre`, `tel`, and the quantifiers `beaucoup`, `peu`,
  `trop`, `tant` (English keeps `many`, `much`, `few`; Spanish `mucho`, `poco`, `tanto`);
- **pronouns**: personal (`je` … `elles`, `me`, `te`, `se`, `moi`, `toi`, `soi`, `lui`, `eux`, `y`,
  `en`), demonstrative (`ce`, `ceci`, `cela`, `ça`, `celui`, `celui-ci`, `celui-là`), relative and
  interrogative (`qui`, `que`, `quoi`, `dont`, `lequel`, `duquel`, `auquel`), indefinite
  (`quelqu'un`, `chacun`, `rien`, `autrui`, `quiconque`);
- **prepositions**: `à`, `de`, `en`, `dans`, `par`, `pour`, `sur`, `sous`, `avec`, `sans`, `chez`,
  `entre`, `vers`, `contre`, `depuis`, `pendant`, `durant`, `avant`, `après`, `devant`,
  `derrière`, `parmi`, `selon`, `malgré`, `envers`, `hors`, `hormis`, `dès`, `jusque`, `outre`,
  `sauf`, `via`, `près`, `afin`;
- **conjunctions**: `et`, `ou`, `mais`, `donc`, `ni`, `car`, `que`, `si`, `quand`, `comme`, `où`,
  `lorsque`, `puisque`, `quoique`, `parce`, `tandis`;
- **auxiliaries and modals**: `être`, `avoir`, `pouvoir`, `devoir` (Spanish's `ser`, `haber`,
  `poder`, `deber`);
- **negation**: `ne`, `pas`, `non` — `pas` by M21.

As Spanish's, the tables hold the dictionary forms and the inflected forms a pack may keep as lemmas
of their own: change 43's tables keep `cet`, `ma`, `mes`, `ta`, `tes`, `ton` apart from `ce`,
`mon`, `son`, so the tables name them, and `la`, `les`, `une`, `cette`, `ces`, `toute`, `tous`… too,
so that a later reduction moving a form to its own lemma does not give a gloss a row for it.

**What a table hides**, measured on GSD (train, dev and test; the words whose form change 43's
tables read as the lemma, and those the treebank reads as another word or tags NOUN):

| Lemma | GSD words | Read otherwise | Decision |
|---|---|---|---|
| `pas` | 1,047 | 10 nouns « step » (1.0 %); the other six French treebanks: 9 of 576 | kept (M21); the noun's row is lost with it, as M8 already merges the two |
| `son` | 3,287 | 21 nouns « sound » (0.6 %) | kept |
| `ton` | 18 | 10 nouns « tone »; the other treebanks 4 of 8 | kept with the possessives — open question 2 |
| `car` | 132 | 4 nouns « coach » | kept |
| `entre` | 493 | 27 forms of *entrer*, already read as *entre* by the tables (M8) | kept |
| `contre`, `avant`, `vers` | 300, 258, 225 | 24, 14, 8 nouns | kept |
| `pouvoir`, `devoir` | 773, 355 | 89 and 9 nouns (« power », « duty ») | kept, as Spanish's `poder` |
| `peu` | 248 | 74 tagged NOUN, all `un peu` | kept |
| `personne` | 171 | 153 nouns (89.5 %; the other treebanks 24 of 32) | **left out** |
| `point` | 185 | 173 nouns (93.5 %) | **left out** — `ne … point` is literary |
| `or` | 69 | 51 nouns « gold » (73.9 %) | **left out** |
| `certain` | 241 | 61 adjectives « sure » | left out |
| `plus`, `jamais`, `guère` | 1,346, 90, 7 | — | left out: adverbs with meaning, as English's `never`, Spanish's `nunca`, `más` |
| `falloir`, `vouloir`, `aller`, `faire` | 120, 117, 267, 1,149 | — | left out: content verbs, as Spanish's `querer`, `ir`, `hacer` |
| `voici`, `voilà` | 9, 20 | — | left out: presentatives |

UD tags `tout`, `même`, `autre`, `tel` and `quel` ADJ on half their uses or more; they are the
indefinite determiners English's `all`, `other`, `such` and Spanish's `todo`, `otro`, `mismo`
stand for, and are kept.

*Rejected — a frequency cut.* The hundred commonest lemmas hold `faire`, `dire`, `temps`, `monde`,
the words worth a row.

### D4 — A French document's names: Spanish's rule, after an elision, and over a hyphenated run

`document_names` gains French's arm. A form is set aside, like an out-of-lexicon proper noun, when
the document never writes it in lowercase, capitalises it at least once in mid-sentence, and its
dictionary form is not a dictionary word of the pack (`is_dictionary_word`) — Spanish's rule — with
two French readings:
- **a capital right after an elided piece is mid-sentence** (`l'Europe`, `d'Espagne`,
  `qu'Augusto`): the apostrophe follows a letter, so the capital is the word's own, even when the
  elided piece opens the sentence. Spanish's `mid_sentence` reads `'` as punctuation and finds no
  evidence there;
- **a hyphenated run the pack does not list is one form** (`Saint-Étienne`, `Jean-Pierre`,
  `Haute-Garonne`): Spanish's rule considers tokens without parts only, and the compound rule sets
  a capitalised run aside only when none of its parts resolves — `saint`, `pierre`, `haute` all do.

Spanish's arm is unchanged (no elision evidence, no runs); English has no names rule.

**Measured on the raw corpus** (tokens set aside beyond the out-of-lexicon rule's 11,579):

| Rule | With the stand-in dictionary words | With none (the state until change 48) |
|---|---|---|
| Spanish's rule | 27,548 | — |
| + a capital after an elided piece | +1,684 | — |
| + a hyphenated run as one form | +2,350 | — |
| **French's rule** | **31,582 (3.65 %)** | **44,939 (5.20 %)** |

By source, with the stand-in: novels 2,438 tokens (1.1 %), Tatoeba 1,925 (1.2 %), UD's raw
sentences 5,187 (4.4 %), Wikipedia 22,032 (6.0 %). The commonest forms set aside: `France` (821),
`Tom` (752), `Toulouse`, `Montpellier`, `Paris`, `Mme` (398), `Montréal`, `Europe`; after an elided
piece `Espagne` (142), `Europe` (120), `Italie`, `Algérie`, `UNESCO`; as runs `Haute-Garonne` (65),
`Saint-Laurent`, `Saint-Jean`, `Michel-Ange`, `Radio-Canada`. A random sample of 80 runs and of 60
words set aside reads as names, acronyms, roman numerals and English words; of the 6,867 distinct
forms, 287 have a lemma among the 3,000 commonest — names, titles, letters, acronyms and English
words (`France`, `Paris`, `Mme`, `The`, `New`, `II`), and a few French words: `Etat`, `Etats`,
`Ecole` written without their accent (13 tokens), `Orient` and `Info` in titles (11).

**Until change 48** the French pack has no dictionary word: change 43 commits `tables/fr/lexical.tsv`
empty, so every capitalised form never written in lowercase is set aside — `État`, `Institut`,
`Conseil` with the names (the second column). Nothing ships before change 52, and change 48 fills the
section from fr-en's glosses. The fixture pack has no lexical section, so its dictionary words are
its glossed lemmas, as Spanish's es-fr has been read.

**What it does not do.** A dictionary word stays a word: `Orange` (the town, glossed « orange »)
and `Vienne`, whose form the tables read as *venir* (M8), keep their cards. A name met only at the
head of sentences, or after `M.` — a full stop, as Spanish's rule reads it —, stays a word: `M.
Myriel`'s capital proves nothing. *Rejected — every capitalised word outside the lexicon's
dictionary words*: `Dieu`, `Seigneur` and a reader's `État` would go.

### D5 — French's analyser version leaves the baseline's

`FRENCH_ANALYZER_VERSION` becomes `1.0.0`, the version change 39's D4 reserved for the cascade: a
French pack built before carries `0.2.0` and is refused by its own version check. Its doc line says
what `1.0.0` adds. Change 42's guard bumps it after this change (`1.1.0`, its D6), as Spanish's
guard followed Spanish's analysis.

**The spec states the version without pinning it.** Spanish's requirements still name `1.0.0` and
`1.1.0` beside a `1.2.0` core. This change's requirement says what holds after any later bump:
French carries a version of its own that is no longer a `0.x` one — `1.0.0` or later —, every page
analysed as French reports it and a French pack must carry it. Change 42's requirement states its
bump without a number either.

What moves with it, as every French bump: the fixture's `manifest.json`; the tests that name
French's version (`language.rs`, `engine.rs`, `packs/pack.rs`, `french_baseline.rs`,
`languages.rs`, the extension's `test/packs.spec.ts`; `crates/lingua-pack` reads the constant). If
change 43 has merged, `tables/fr-en/` is re-reduced (its D12: `manifest.json` alone moves, and the
pack's bytes and its pin with it); if change 43 merges after, its reduction reads `1.0.0` from
`analysis/mod.rs`.

### D6 — The measurement at implementation

When change 43's tables are on `main`, the pull request runs `scripts/lingua-data/measure/fr-ud.sh`
(change 43's) and records the figures it prints in the programme. With change 43's prototype tables,
through this cascade:

| | Words | Resolved | Content words | Auxiliaries |
|---|---|---|---|---|
| UD French-PUD (gated) | 20,232 | 99.12 % (99.12) | **96.48 %** (96.38) of 9,573 | 99.90 % (99.90) of 1,030 |
| GSD test (reported) | 8,049 | 98.91 % (98.91) | **95.81 %** (95.65) of 3,791 | 99.72 % (99.72) of 359 |

In brackets, `main`'s baseline lemmatisation over the same pack. The harness lemmatises UD's words as
written: it measures the cascade (D1 does not move UD, which is NFC; D3 and D4 are not lemmas). The
gates are met with a margin either way (98.5 / 93.5 / 97). If change 43 merges after this change,
its own measurement runs through this cascade and records these figures.

### D7 — What the French golden shows, and what cannot move

The fixture gains the 17 forms the real tables hold where the fixture's gaps would mislead:
`printemps`, `moins`, `longtemps`, `travaux` (→ *travail*), `endors` (→ *endormir*) and `apres`
(the real tables rank the informal spelling), without which step 2 writes `printemp`, `moin`,
`longtemp`, `traval`, `endor` and `apre`; `paris`, `lot`, `aube`, `saint`, `pierre`, `orange`,
`vienne` (→ *venir*), `mme` and `personne`, which change 43's tables rank — so that the names rule
meets lexicon words, as it will on the real tables —, `travail` and `endormir` with them; and two
glosses, `orange` and `pierre` (« stone »), so that a dictionary word is seen staying a word. The
scenario gains two phrase probes from the `homographes` and `fiction` pages: « Il ne fait pas un pas
sans son chien, et le son de sa voix le rassure. » (both `pas` and both `son` flagged — M21's and
M8's cost) and « Personne au village ne se souvenait » (`personne` not flagged).

**Against `main`'s golden** (141 probes): 39 move, 2 are added, 102 are byte for byte.
- `pack`: `analyzer_version "1.0.0"`, 5,139 → 5,439 bytes.
- The 13 `analyse new-reader` pages and the 4 `analyse reader` pages (the version on each).
- 20 of the 22 phrase glosses (their function-word flags; « Aujourd'hui » and « The lighthouse
  stood » hold none) and the reader's phrase gloss.
- Unmoved: every `gloss` and `word-grammar` probe, the expressions of every phrase gloss, the
  levels, the ladder, the estimates, the review, the exports and the backup.

Token by token (1,002 tokens before and after; counted 958 → 963, glossed 387 → 390, set aside
44 → 39), each rule switched off from the whole:

| Rule | Tokens it moves | Probes |
|---|---|---|
| NFC (D1) | 4 of the `technique` page's NFD block composed — `Vérifiez`, `système`, `télécharger`, and `mémoire`, whose gloss « memory » appears | 1 |
| The unlisted plural (D2) | 18: `bâtiments`, `publics`, `syndicats`, `terminés` (`actualites`), `cerises`, `clients`, `levées`, `prudents`, `précautions`, `écrites`, `poules`, `volets`, `quelques`, `réflexions` ×2, `cuillères`, `fines`, `rondelles` — each read as its singular, where the fixture lacks a form the real tables hold | 9 |
| The closed classes (D3) | no page token; 46 of the 91 tokens of the 23 phrase glosses `main` has flagged (`il y a` all three, `du pain et des œufs` `du`, `et`, `des`; the Italian `Il` and `si` too: a selection has no language detection), and 17 of the two new probes' 24 | 23, 2 of them added |
| The names rule (D4) | 5 set aside on `noms`: `Paris`, `Lot` (Spanish's rule), `Aube` (only after `l'`), `Jean-Pierre`, `Saint-Étienne` (as runs) | 1 |
| The fixture's forms alone | `travaux` → *travail*, `endors` → *endormir*; `Orange` and `Vienne` (dictionary words), `Mme` (only at a block's head), `Longtemps`, `Personne` become words; `Paris`, `Lot`, `Aube`, `Jean-Pierre`, `Saint-Étienne` lexicon words, set aside again by the names rule | — |

`french_baseline.rs` asserts what each rule shows: the NFD block's `mémoire` composed and glossed
(`the_nfd_block_s_memoire_is_glossed_once_french_composes_it` flips to its title), `pas` and `ne`
flagged and `personne` not, the five names set aside and `Orange`, `Vienne`, `Mme` kept, French's
version no longer `0.x`.

**What cannot move.** Every rule is French's arm of a `match` on the studied language (`nfc_for`,
`lemmatize`, `is_function_word`, `document_names`); Spanish's names rule is called as before.
Measured on the prototype, without re-blessing: `english_baseline` (en-fr), `spanish_baseline`
(es-fr), `es_en_baseline`, `en_es_baseline`, `cross_native` and `parity` pass, 25 tests in 6 suites,
and no committed golden, table or pin differs from `main`. The pack builder's French path calls
`lemmatize` for expression keys: step 2 only writes words outside the lexicon, which drop a key
anyway, so no key moves (the golden's expressions are byte for byte).

**Tests that flip at `1.0.0`**, each rewritten with the rule it now states (measured on the
prototype: these nine Rust tests and no others fail before they are rewritten): in lingua-core
`analysis::language::tests::english_keeps_its_analyser_version_and_spanish_has_its_own`,
`analysis::function_words::tests::spec_scenario_no_french_word_is_a_function_word_yet`,
`analysis::tokenize::tests::french_text_is_read_as_it_came`,
`engine::tests::spec_scenario_a_french_page_is_read_by_the_baseline`,
`packs::pack::tests::spec_scenario_a_french_pack_at_french_s_analyser_version`; in lingua-wasm
`french_has_its_pre_pass_and_the_baseline_s_lemmas`,
`the_nfd_block_s_memoire_is_glossed_once_french_composes_it`,
`a_fixture_left_behind_its_analyser_names_its_manifest` (`french_baseline.rs`) and
`spec_scenario_a_french_reader_s_backup_is_version_3` (`languages.rs`); and the extension's
`test/packs.spec.ts`, which reads French's version.

### D8 — OpenSpec: ADDED only

Four ADDED requirements in `lingua-analysis`, held by no other change: *French text is read in
NFC*, *French lemmatisation cascade*, *French closed classes*, *A French document's names are set
aside*. None is MODIFIED.

The two requirements change 39 added and change 40 MODIFIED are left to them. Their wording was
written to expire: French is served « until its lemmatisation rules are written » by the rest of the
baseline, at « `0.2.0` while its lemmatisation is the baseline's ». *French lemmatisation cascade*
says that these are French's lemmatisation rules, so those clauses no longer bind. Three scenarios
name what this change ends and will read false once it is implemented: *No French word is a
function word yet* and *Each language reports its own version* (`0.2.0`) in *French is a studied
language served by the baseline analysis*, and *What the baseline shows today* (`0.2.0`, no function
word) in *A French invariance baseline runs beside the English and Spanish ones*. They are handed on
(open question 1).

`archiveAfter`: change 39 and change 40, whose requirements this change's build on (the French
variant, the pre-pass whose pieces it reads), and change 43, whose tables decide every lemma it
returns (*French's forms and frequencies*). `openspec_archive_order.py` exits 10 naming the three.
The phrase gloss's function-word flag is `add-lingua-phrase-gloss`'s (open, implemented, one owner
task left); an ADDED requirement needs no order on it, as *Spanish closed classes* did not.

## Risks / Trade-offs

- [The names rule sets aside dictionary words until change 48] → `tables/fr/lexical.tsv` is empty
  until fr-en's glosses (measured: 44,939 tokens instead of 31,582); nothing ships before change 52,
  and change 48's re-bless shows them come back.
- [A name after `M.` stays a word] → as Spanish's rule reads a full stop; another occurrence in the
  document usually carries the evidence.
- [`Vienne` reads as *venir*] → M8: the tables chose; the names rule cannot set aside a dictionary
  word's form. Lemma alternatives are the optional change.
- [« pas » and `son` hide their noun's row] → M21 and M8 accept it; measured at 1.0 % and 0.6 % of
  GSD's uses.
- [The unlisted-plural rule misreads a rare singular in `-s`] → only outside the lexicon, lowercase,
  with its singular unknown too; both forms were unknown words before.
- [Change 43's prototype tables map `m` to *paris*] → the English Wiktionary's « Paris » entry lists
  its gender `m` as a form, so `M.` (Monsieur) reads *Paris* (290 raw tokens, set aside by the
  names rule while `paris` is no dictionary word). A defect of change 43's reduction, reported to
  it; nothing here depends on it.
- [A French rule runs on English or Spanish] → French's arms only; the four other goldens pass
  without re-blessing (D7).

## Migration Plan

Nothing to migrate: no stored format, wire field or other pair's byte moves, and no reader studies
French. A French pack must carry French's new version; the fixture is re-stamped, and fr-en's
tables re-reduced if change 43 has merged. Rollback is a revert.

## Open Questions

For the owner, none blocking:
1. **The baseline's scenarios after the cascade** (D8). This change only adds requirements, so
   once changes 39, 40 and 41 are archived the spec still holds *No French word is a function word
   yet*, *Each language reports its own version* (`0.2.0`) and *What the baseline shows today*
   beside this change's. Recommended: the first change archived after this one that touches *French
   is a studied language served by the baseline analysis* or *A French invariance baseline…* — at
   the latest `enable-lingua-french` (change 52) — MODIFIES both to drop the baseline's tense (the
   first one's name too), as `add-lingua-spanish-analysis` rewrote *Analysis by studied language*.
   The alternative is this change MODIFYING them with change 40 in `archiveAfter`, which the
   programme's rules allow.
2. **`ton`** (D3): flagged with the possessives, it hides « tone » (10 of GSD's 18 uses, 4 of 8 in
   the other treebanks). Leaving it out is one line.
3. **The names rule's two French readings** (D4) — after an elided piece, over a hyphenated run —
   go beyond Spanish's rule as the owner chose it (2026-10-04); measured at 1,684 and 2,350 raw
   tokens, names in every sample.
