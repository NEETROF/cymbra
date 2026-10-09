# add-lingua-french-expression-keys — French expressions keyed as French is read

## Why

Change 44 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en, fr-es). A pack's expression table is keyed by the builder
(`crates/lingua-pack` `expression_key`): the headword is split at its spaces, each word lemmatised,
and the key is kept only when every word is a lemma of the lexicon. The phrase gloss
(`engine::match_expressions`) then joins the lemmas of a selection's tokens and looks the run up.
The two meet only while a word on the page is a token. French breaks that from change 40 on
(`add-lingua-french-tokenisation`, M21): « Au revoir » reads `À` + `le` + `revoir`, « coup d’œil »
reads `coup` + `de` + `œil`, « d’abord » reads `de` + `abord`, « Qu’est-ce que » reads `Que` + `est`
+ `ce` + `que`. A key written `au revoir` or `coup d'œil` is never met.

Measured on the English Wiktionary's French section (the file change 43 measured) against change
43's prototype tables, with change 40's implementation at analyser `0.2.0`: of 17,523 candidate
expressions, today's keying keeps 12,428, and **2,158 more** are reachable once the key is read the
way the page is — 1,564 holding an elision (`coup d'œil`, `jusqu'à ce que`), 457 holding `au` or
`aux` (`au revoir`, `au fur et à mesure`), 101 words without a space that the pre-pass splits
(`d'abord`, `c'est`, `allez-y`), 36 holding an inversion (`est-ce que`, `y a-t-il`). Change 40's
design (D8) gave this change the repair and added `au revoir` and `coup d'œil` to the fixture, so
that its re-bless shows both appear; change 43 (D4, D12) left `d'abord`, `c'est` and `l'on` out of
the forms table for this change to key as expressions.

Keying alone is not enough for French, measured:
- **The key is the card.** The phrase gloss reports the key as the expression's dictionary form,
  and the extension heads the card with it, keys the card and its status by it, and lists it as a
  row. A lemma chain is not French: `il y avoir`, `de bon heure` (`fr-en.golden` today),
  `à le revoir`. Spanish shows the same today (`tener en contar` for « tener en cuenta » in
  `es-fr.golden`; 3,936 of es-fr's 11,972 keys are not their headword).
- **« One form, one lemma » merges articles** (M8: `la`, `les` → `le`). The expression `à la`
  (« in the style of ») would answer every « au », and `haut la main` (« easily ») and
  `haut les mains` (« hands up ») would be one entry.
- **The pieces lengthen a key**: `au fur et à mesure` is five words and six tokens, beyond the
  five-token window.
- **`du` and `des` stay whole** (M21), so `à cause de` never meets « à cause des ».

Spanish is the precedent of leaving it: es-fr's 596 headwords holding `al` or `del` are keyed
`al …`, 526 of them sit in the pack, and no selection read `a` + `el` reaches them.

## What Changes

- **The builder keys a French headword through the core's French reading** — French's pre-pass
  and lemmatisation, the code a page goes through — not by splitting at spaces: `au revoir` →
  `à le revoir`, `coup d'œil` → `coup de œil`, `d'abord` → `de abord`, `il y a` → `il y avoir`.
  Neither the tables (a Python copy of change 40 in the reducers) nor the matcher (rebuilding the
  written words from the tokens) does it (design D1, the three measured).
- **Determiners stay as written in a French key**: `le`, `la`, `les`, `un`, `une` and the
  possessive and demonstrative determiners are written as the pre-pass gives them, every other
  token as its dictionary form: `à la` is `à la`, never `au`; `haut la main` and `haut les mains`
  stay two (D2).
- **A French expression is named by its headword.** The phrase gloss reports the dictionary's
  spelling (`au revoir`, `il y a`, `de bonne heure`) as the match's key, and the reader's status
  is read on it, so the card, the deck and the row say what the dictionary says. The pack carries
  the name wherever it differs from the key, in an optional section of its own (D3).
- **French's window is seven tokens**, holding 98.8 % of its keys as five holds 98.9 % of
  English's; a longer key is left out at build (D4).
- **`du`/`des` closing a run may stand for the `de` an expression ends on**: « à cause des »
  answers `à cause de` (D5).
- **A headword is an expression when French reads it as two tokens or more**: `d'abord` and
  `allez-y` are, `peut-être` and `aujourd'hui` are not; a headword part of which the analysis
  drops (`compte en t`) is left out (D6).
- **The fixture and the French golden**: the fr-en fixture gains `à la`, `d'abord` and
  `au fur et à mesure`, its pack version moves; four phrase probes are added; `fr-en.golden`
  moves on 6 of its 141 probes, gains 4, and no `analyse` probe moves (D7, D8).
- **Nothing else moves**: English and Spanish keys, the en-fr, es-fr, es-en and en-es packs and
  goldens, byte for byte; French's analyser version stays (D7).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A French pack keys its expressions as French is read* (the
  reading, the determiners, which headwords are expressions, the names, two headwords on one key,
  English and Spanish unmoved).
- `lingua-analysis`: ADDED — *French expressions are found on French's reading of a selection*
  (the pieces, the window, `du`/`des`, the name reported and the status read on it).

No requirement is modified. Both extend `add-lingua-expression-table`'s *Multi-word expression
table* and *Expression lookup in a phrase gloss*, which that change still holds, and read French as
changes 39, 40, 41 and 43 define it: the five are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: the reading of a headword (`analysis/pipeline.rs`), French's key
    pieces, window and `du`/`des` retry in `engine::match_expressions`, the names section and
    `Pack::expression_name`; *consumed*: the tokeniser and its French pre-pass, `resolve_lemmas`,
    the expression table, the knowledge model.
  - `crates/lingua-pack` — *new*: the French arm of `expression_key`, the names section, French's
    order among headwords of one key, the window cut; English and Spanish paths untouched.
  - `crates/lingua-wasm` — the French baseline re-blessed, four phrase probes added.
  - `scripts/lingua-data/testdata/fr-en/` — three expressions, three forms, the pack version.
  - `apps/lingua-extension` — *consumed*, unchanged: the card already shows the key the phrase
    gloss hands it. No French reaches it before change 52.

  ID, Music, Live, the back office, the site, the backend, the Apple host app, the agent plugin
  and every reducer are untouched.
- **Release.** Silent: no listed pair studies French.
- **Compatibility.** No stored format, wire field, table, pin or committed pack moves. A French
  pack gains an optional section an older core ignores; none is published before change 52.
- **Handed on.** Which headwords a French reducer offers as expressions — the single words the
  pre-pass splits among them — and which senses only point at another word (« que + elle »,
  « post-1990 spelling of … »): changes 48 and 49, with their glosses. A one-word selection over
  several pieces opening the whole-selection card, where `d'abord` answers, and a name without a
  space stored with its gloss: change 51.
- **Effort, against 1–2.5 ideal days**: 1.5–2.5 (design, *Effort*).
