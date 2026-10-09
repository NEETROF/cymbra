# add-lingua-spanish-expression-keys — Spanish expressions keyed as Spanish is read

## Why

Row 44b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. Change 44 (`add-lingua-french-expression-keys`, merged as #835) keys a French
pack's expressions through the core's own French reading and names each by its headword; its open
question 4 asked whether Spanish gets the same rules. The owner answered on 2026-10-09: yes, in a
change of its own. This is that change. It moves es-fr's output, which readers have, and es-en's.

Spanish keys are still built the way change 44 replaced for French: the headword split at its
spaces, each word lemmatised. The page is not read that way. Measured on the committed tables
(`tables/es`, `tables/es-fr`, `tables/es-en`) and the Spanish corpus (`baseline/pages-es.txt`):
- **`al` and `del` are never met.** The Spanish pre-pass splits them into `a` + `el` and
  `de` + `el` (*Spanish tokenisation pre-pass*), but `al` and `del` are lemmas of `tables/es`, so
  526 of es-fr's keys and 607 of es-en's hold `al` or `del` as a word and no selection reaches
  them: « al menos », « al fin y al cabo », « del todo ».
- **The article entries answer every article.** « One form, one lemma » (M8) files `la`, `los` and
  `las` under `el`. es-fr's `a la`, `a las`, `a los` (« À la », « Aux ») are all keyed `a el`, and
  its `de las`, `de los` (« Des ») `de el`: « al menos » answers `a el` « À la », and 17 of es-fr's
  45 matches on the corpus are those entries, 13 of them on a contraction or another article
  (« del », « al », « de la »). Distinct entries share a key: « a los » answers `a la`'s « À la »,
  not its own « Aux ». And once `al` is read `a` + `el`, `a la par de` and `al par de`, or
  `a las armas` and `al arma`, would be one key if the articles were lemmatised.
- **The card shows a lemma chain.** The phrase gloss reports the key as the expression's dictionary
  form, so the card is headed, keyed and listed by it: `tener en contar` for « tener en cuenta »,
  `dar contar` for « darse cuenta », `a el` for « a la casa » (`es-fr.golden` today). 3,815 of
  es-fr's 11,851 keys are not their headword, 4,665 of es-en's 13,739.
- **Five tokens are not enough once `al` and `del` are read**: « al fin y al cabo » is five words and
  seven tokens.

Unlike French, Spanish is read today: es-fr has shipped. A reader may hold a status or a deck card
on an expression under the lemma chain it was shown under. 3,538 of the 11,199 expressions es-fr
reaches today would be reported under another string.

## What Changes

- **The builder keys a Spanish headword through the core's Spanish reading**, the function change
  44 wrote for French made one function for both: `al menos` → `a el menos`, `del todo` →
  `de el todo`, `tener en cuenta` → `tener en contar` (design D1).
- **Spanish's determiners are written as the pre-pass gives them** — the articles (`el`, `la`,
  `los`, `las`, `lo`, `un`, `una`, `unos`, `unas`) and the possessive and demonstrative
  determiners, 35 words —, every other token as its dictionary form: `a la vez` is `a la vez`, and
  `a la` no longer answers « al » (D2).
- **A Spanish expression is named by its headword**: es-fr's and es-en's packs carry the
  `expr.names.zst` section change 44 added, the match reports the name and the status is read on
  it; the order among headwords of one key is French's (D3).
- **Spanish's window is seven tokens**, as French's; a longer key is left out at build (D4).
- **A Spanish match never ends inside a written word**: when it ends on the `a` or `de` of « al » or
  « del », it covers the article too, so « después del » is answered whole (D5).
- **An expression the reader settled before keeps its key**: where the reader holds a status (or a
  withdrawn one) on a run's lemma chain and none on its name, the match reports the lemma chain, as
  before, and the reader's deck card and status are still read (D6).
- **Review finds a Spanish expression card by its name**, and a card made before by its lemma
  chain (D7).
- **What moves** (D8): `es-fr.golden` and `es-en.golden`, each on its `pack` line and 7 phrase
  probes, 4 probes added to both; `fr-en.golden` on its `beside es-en` line alone; es-fr's and
  es-en's pins (the packs' sha256 and size). Spanish's analyser version, every `analyse`, `gloss`
  and `word-grammar` probe, the tables, `pack_version`, en-fr, en-es, fr-en's own output and every
  extension snapshot do not.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A Spanish pack keys its expressions as Spanish is read* (the
  reading, the determiners, the window, the names, two headwords on one key).
- `lingua-analysis`: ADDED — *Spanish expressions are found on Spanish's reading of a selection*
  (the pieces, the window, the article of a contraction covered, the name reported) and *A Spanish
  expression settled under its lemmas keeps that key*.
- `lingua-decks-review`: ADDED — *Review finds a Spanish expression card by its name*.
- `lingua-data-packs`: MODIFIED — *Versioned pack container, keyed by language pair*: a section the
  builder derives from committed tables (the names) leaves `pack_version` alone, the pin recording
  the bytes; a table added with new tables still bumps it (design D9). No open change holds it.

For Spanish, the ADDED requirements take the place of the key and match rules of
`add-lingua-expression-table`'s *Multi-word expression table* and *Expression lookup in a phrase
gloss*, and of the sentences of change 44's *A French pack keys its expressions as French is read*,
*French expressions are found on French's reading of a selection* and *Review finds a French
expression card by its name* that keep Spanish as it was; both changes are open, so each
requirement here says so rather than MODIFY them (design D9). The four changes whose requirements
this one builds on are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: `SPANISH_KEY_WRITTEN`, Spanish's window, the article of a
    contraction covered and the settled lemma chain kept in `engine::match_expressions`;
    *generalised*: change 44's `french_expression_key` serves Spanish too, French byte for byte;
    *consumed*: `headword_reading`, the Spanish pre-pass and cascade, `Pack::expression_name`, the
    knowledge model.
  - `crates/lingua-pack` — the Spanish arm of `expression_key`, the names section and French's
    order for Spanish packs; English untouched.
  - `crates/lingua-wasm` — review's `readable_gloss` for a Spanish expression card; the Spanish
    baselines re-blessed, four phrase probes added; the French baseline's `beside es-en` line.
  - `scripts/lingua-data/tables/es-fr/pin.json`, `es-en/pin.json` — the pack's sha256 and size.
  - `apps/lingua-extension` — *consumed*, unchanged: the card already shows, keys and acts on the
    key the phrase gloss hands it.

  ID, Music, Live, the back office, the site, the backend, the Apple host app, the agent plugin
  (which reads no expression) and every reducer are untouched.
- **Release.** es-fr has shipped: merging changes no reader's extension. The change reaches es-fr's
  readers only with an extension release — Chrome, Firefox and the Safari host app, each carrying
  the new pack — which the owner decides (M18, task 6.4), after approving the re-bless (6.1). es-en
  is not shipped yet.
- **Compatibility.** No table, stored format, wire field or `pack_version` moves (the container's
  `pack_version` sentence modified to say so, D9). The packs gain an optional section an older core
  ignores; a pack and its core ship together. A reader's statuses and cards are not rewritten (D6).
- **Effort, against no programme estimate (outside the 57)**: 1.5–2.5 ideal days (design,
  *Effort*).
