# add-lingua-french-word-card — the word card of French, in English and in Spanish

## Why

Change 51 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en for English speakers, fr-es for Spanish speakers). The programme's
architecture for the card: « a language-neutral description of readings, then one renderer per
native language. Only tense names are keyed by pair (`Tense=Past` is … "past historic" in fr-en, …
« pretérito perfecto simple » in fr-es). Goldens lock the order of tenses. » Change 18
(`generalise-lingua-card-wording`) built the description and the three renderers, for English and
Spanish studied only: `StudiedLanguageCode` is `en | es`, and its D3 left « fr studied » to « stage
3's type ». A French-native reader cannot study French (change 39), so French's card is read in
English (fr-en) and in Spanish (fr-es).

Change 45 (`add-lingua-french-grammar-tables`) gives French's forms their readings: 125,193
readings of 88,579 forms in 79 tags, the passé simple as `Tense=Past`, the conditional and the
imperative without a tense, a present participle (`VerbForm=Part|Tense=Pres`), and « parle »'s
five readings stored unmerged for this change to say once (M21). The card cannot say any of it
yet, and its rules, applied to French as they stand, would say it badly — measured on change 43's
implemented tables with change 45's readings (all 124,050 forms, design *How it was measured*):
- no renderer has a French table, and `finiteKey` keys any language but Spanish as English's
  indicative, so no French finite form would be named;
- the 2,836 forms whose reading is the present participle would be unnamed (`formKind` names a
  participle only when its tense is `Past`);
- « parle », once named, would read « first- and third-person singular present indicative,
  first- and third-person singular present subjunctive and second-person singular imperative of
  parler »: M21 (2026-10-09) merges the moods, which no card does yet (Spanish's « hable » reads
  its subjunctive and its imperative apart);
- 1,072 forms spelled like their dictionary form would say « may also be the masculine plural
  of … » of themselves — 71 of them among the 1,000 commonest lemmas: `temps`, `pays`, and `un`,
  `pas`, `plus`, `si`, `nous`, read through their homograph nouns (M8).

Three hand-overs land here too:
- **A selection inside a word written as pieces** (change 40's D7): change 40 gave each elision
  piece its own span (M21), but a selection is widened to whole words before it is read, so a drag
  over « homme » in « l’homme » opens `le`, and a drag over « il » in « dit-il » opens `dit`
  (`test/selection.spec.ts` pins it). 72 of the 992 written words of the French corpus are such
  words; none of the English corpus's 867 nor of the Spanish corpus's 585 is, through any of the
  four other pairs.
- **An expression named without a space** (change 44's D9): `d'abord` is an expression keyed
  `de abord` and named `d'abord`, answered by the whole-selection card; « + Deck » asks its gloss
  of the single-lemma port, which knows no `d'abord`, and review reads a card as an expression only
  when its lemma holds a space.
- **The French card's goldens** (changes 48 and 49): fr-en's card renders the French invariance
  baseline once change 48 has put it on the committed tables; fr-es's golden is this change's, as
  en-es's was change 24's.

## What Changes

- **French is a studied language the card names** (design D1): the renderers' key,
  `StudiedLanguageCode`, gains `fr`; the extension's `StudiedLanguage` stays `en | es` for change
  52, as change 47 left it, so no French reaches a card before then — the renderers are driven by
  tests.
- **French's forms, named by each renderer** (D2): in English, the English Wiktionary's French
  form-of wording — present and imperfect indicative, "past historic (passé simple)" (M10), simple future,
  conditional, present and imperfect subjunctive, imperative —; in Spanish, the RAE's terms (M10) — « pretérito
  perfecto simple de indicativo », « futuro simple de indicativo », « condicional simple »… —; in
  French, for the override M2 reserves, French school terms. Tenses come in the grammars' order:
  indicative, conditional, subjunctive, imperative; the snapshots lock it.
- **The indicative and the subjunctive said once (M21)** (D3): where a form reads both moods in one
  tense, persons and number, the card names them once — « first- and third-person singular present
  indicative or subjunctive and second-person singular imperative of parler », « primera y tercera
  persona del singular del presente de indicativo o de subjuntivo y segunda persona del singular del
  imperativo de parler ». 6,147 forms, the 2,142 five-reading forms M21 names among them — wider than
  its words, as change 45's D7 leaves to this change (open question 1).
- **The present participle named** (D4): "present participle of parler", « participio presente de
  parler »; the past participle « participio pasado » in Spanish beside it. French has no gerund.
- **What stays unnamed on a French card** (D5): a plural spelled like the card's own dictionary
  form (« temps », « un »); a numeral's form and a plural determiner or pronoun without a gender, as
  a Spanish card's are. A comparative is named as such, its agreement said by the form's own line.
- **The Spanish card names two genders of one number once** (D6), as the English card does since
  change 23: « el masculino y femenino plural de somme », 951 French forms; no English reading has
  a gender, so en-es does not move.
- **A selection inside a word written as pieces opens the piece selected** (D7): the capture keeps
  the selection as the reader made it beside the widened one; within an analysed block, a selection
  inside one piece opens that piece, and one covering several opens the whole-selection card, where
  an expression covering them is the answer (« D’abord » → `d'abord`). Pieces sharing a span
  (`don't`, `del`, `au`) open as before.
- **An expression named without a space** (D8): a card the expression table answered stores the
  gloss it showed, whatever its name's spelling; review reads a French card whose lemma French reads
  as two tokens or more at that key (`crates/lingua-wasm`).
- **The French card pinned on the real packs** (D9): the French invariance baseline gains the card's
  probes — 21 forms, one per name the card says, and 40 lemmas with two sense runs or more — and a
  phrase probe; `fr_es_baseline.rs` writes `fr-es.golden`, the French scenario glossed in Spanish
  with en-es beside it, equal to `fr-en.golden` on the studied side; `test/word-card-fr-en.spec.ts`
  and `-fr-es.spec.ts` render every grammar and phrase probe of each golden and pin the lines in
  `test/baseline/word-card-fr-{en,es}.txt`, run and re-blessed where the others are. What the
  snapshots show wrong in the data is listed for the pairs' refinements.
- **Nothing else moves** (D10): every form of the en-fr, es-fr, es-en and en-es packs renders byte
  for byte (measured: 440,534 forms), `test/word-grammar.spec.ts` is unchanged, the four other
  goldens and the three committed snapshots pass as committed; `fr-en.golden` gains 62 probes and
  moves none.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`:
  - ADDED — *A French card names its forms in the interface language's grammar* and *The French
    card is pinned on the real packs*. They read *The word card describes a form once, and says it
    in the interface language* (change 18) as written: for French, a studied language its « today »
    did not have, what is named is the description's, the same in every renderer.
  - MODIFIED — *Selection capture on any pointer*, held by no open change: a one-word selection
    over a word written as pieces with spans of their own; its text and scenarios kept.
- `lingua-decks-review`: ADDED — *Review finds a French expression card named without a space*,
  beside change 44's *Review finds a French expression card by its name*.

## Impact

- **Products.** Cymbra Lingua only:
  - `apps/lingua-extension` — *changed*: `src/i18n/index.ts` (`StudiedLanguageCode`),
    `src/reading/grammar-description.ts` (what French names, the moods' merge, the order),
    `src/i18n/{en,es,fr}/grammar.ts` (French's tables, the present participle, Spanish's genders),
    `src/i18n/README.md`, `src/reading/selection.ts` (the reader's range), `src/reading/session.ts`
    and `selection-card.ts` (the routing, `cardGloss`); *new*: `test/word-card-fr-en.spec.ts`,
    `test/word-card-fr-es.spec.ts` and their snapshots; *moving*: `test/word-grammar-en.spec.ts`,
    `-es.spec.ts`, `grammar-description.spec.ts`, `selection.spec.ts` (change 40's pinned case),
    `session.spec.ts`/`reading-session.spec.ts`; `test/word-grammar.spec.ts` unchanged; *consumed*:
    the description and the renderers' API (change 18), the click's hit test (40), the closed classes
    the rows leave out (41), the expressions' names (44), the readings (45), `rowGloss` and the
    snapshots' harness (23, 24).
  - `crates/lingua-wasm` — `src/lib.rs` `readable_gloss` (French's arm, over change 44's
    `french_expression_key`, *consumed*); `tests/support/french.rs`
    (the card's probes), `tests/fr_es_baseline.rs` and `baseline/fr-es.golden` (new),
    `baseline/fr-en.golden` (probes added).
  - `.github/workflows/lingua-extension-check.yml` and `lingua-pack-update.yml` — `fr_es_baseline`
    and the two snapshots beside the others.

  ID, Music, Live, the back office, the site, the backend, the packs, the reducers, lingua-core,
  the Apple host app and the agent plugin are untouched.
- **Release.** Silent: no listed pair studies French, `StudiedLanguage` has no `fr` until change 52.
  English and Spanish selections move only over a hyphenated run holding a digit, of which the
  English and Spanish corpora hold none (D7).
- **Compatibility.** No stored format, wire field, table, pin or pack moves.
- **Order.** Implemented after the implementations of changes 44 (the names), 45 (the readings), 48
  (the French baseline on the committed tables) and 49 (fr-es's tables) — change 41's, whose closed
  classes the rows leave out, is on `main`, and `fix-lingua-lemma-lookup` (41b) comes before 45, so the
  snapshots never pin `été été` or `porte porte` read off *être* or *porter*; before 52. Until 48, the
  French golden answers `"readings":[]` and a snapshot would pin nothing; if 49's committed measurement
  falls below its floor (M6: 81.4 / 68.8 / 54.5 %), no fr-es table is committed, fr-es's golden waits
  for the change that commits them, and the Spanish snapshot pins the Spanish grammar lines of
  fr-en's probes, with no gloss (D9).
- **Not here.** The pairs' glosses and their refinements (48, 49, `refine-lingua-fr-en-glosses`);
  `StudiedLanguage`, the labels, `packs.json` and the dogfood on devices (52); the shared wording
  questions the snapshots raise for every pair (56, `refine-lingua-matrix-wording`); `Pack::readings`
  for a lemma that is another word's form (change 45's open question 4: `fix-lingua-lemma-lookup`, 41b).
- **Effort, against 3.5–6 ideal days**: 4–5.75 (design, *Effort*).
