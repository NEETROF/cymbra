# pick-lingua-expression-written-as-the-page — « ça va » answers `ça va`, not `ça ira`

## Why

Row 44e of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. Change 44 (`add-lingua-french-expression-keys`, merged) keys a French
expression by what French's analysis reads in its headword, so an expression is found in any of its
forms (« il y avait » answers `il y a`). The same reading makes two headwords one key when their
words share their dictionary forms: `ça va` (« how are you? ») and `ça ira` (« it will be all
right ») are both `ça aller`. The pack keeps one headword per key (change 44's D6: the one written as
the key, then the most pieces written as their key piece, then byte order), so `ça ira` wins and
**« Ça va ? » is answered `ça ira`, « it will be all right »**, in fr-en. fr-es holds `ça va` alone.
The owner decided on 2026-10-10, in a change of its own: **when several expressions share one key,
the one written as on the page is chosen.**

Measured on fr-en's and fr-es's committed tables and change 44c's 51,790 French selections (UD
French-GSD, four Gutenberg novels, French Wikipedia, Tatoeba, change 39's pages: 1,085,056 tokens):
- **fr-en: 60 keys several headwords reach** (121 headwords, `allez-y`, `allons-y` and `vas-y` on
  one), **fr-es 64** (128). The other headwords are not in the pack today.
- **1,398 of fr-en's 60,359 matches fall on such a key, and 128 are named by a headword the page
  does not write**: `droit de l'homme` on « droits de l'homme » 42, `ça ira` on « ça va » 19,
  `peu importer` on « peu importe » 12, `allez-y` on « vas-y » 9 and « allons-y » 3, `quoi qu'il en
  fût` on « quoi qu'il en soit » 7, `grand air` (« open air ») on « grands airs » (« supercilious
  attitude ») 4. fr-es: 220 and 46 (`les siennes` on « les siens » 13, `tous les deux` on « toutes
  les deux » 10).
- Several of those are other expressions, not spellings: `fruit de mer` « a piece of seafood » and
  `fruits de mer` « seafood », `pied de mouche` « paragraph mark » and `pieds de mouche` « chicken
  scratch », `petit mal` and `moindre mal` (« lesser evil »), `demander en mariage` and `demande en
  mariage`.

## What Changes

- **A French pack keeps every headword of a key** (design D1): the key, its winning headword, gloss
  and name stay as change 44 makes them; each other headword becomes an expression of its own in
  `expr.zst` and `expr.names.zst`, under an id after the keys' that no key maps to, and a new
  optional section, `expr.others`, lists each shared key's other headwords in change 44's order. A
  core that predates it reads the pack as today. fr-en 2,537,386 → 2,538,483 B (+1,097), fr-es
  1,971,289 → 1,972,957 B (+1,668); `pack_version` and `FORMAT_VERSION` stay, the two pins move.
- **A match is named by the headword written most like the page** (D2): among the key's headwords
  whose pieces meet the run as changes 44c and 44d require (D3), the one whose pieces are written as
  the run's tokens at the most places wins, change 44's order between equals; its name and gloss are
  the match's, and the status is read on its name. « Ça va ? » answers `ça va`, « Ça ira. » `ça ira`,
  « Allons-y ! » `allons-y`; « quelle que soit » `quel que soit` (two pieces of three written as the
  page, `quel que fût` one); « ça allait » `ça ira` as today (one each, change 44's order).
  On the corpus: fr-en 143 matches renamed, fr-es 47, none lost; fr-en gains « c'qui » `c'qui`,
  whose winner `ce qui` does not meet an elided « c' » (D3). Gloss time +0.3 %.
- **Review reads a French card's gloss under its own headword** (D4): a card `ça va` reviewed in
  another language than its gloss shows `ça va`'s gloss, not the key's.
- **What moves** (D5): `fr-en.golden` and `fr-es.golden` move their `pack` line and gain five
  phrase probes — « Ça va ? », « Ça ira. », « Allons-y ! », « les miens », « Quelle que soit la
  distance » —, 10 lines each, every probe before byte for byte; `word-card-fr-en.txt` +19 lines and
  `word-card-fr-es.txt` +17, their specs' count 38 → 43; `tables/fr-en/pin.json` and
  `tables/fr-es/pin.json`, the pack's sha256 and size. French's analyser version, `pack_version`,
  the tables and every other snapshot do not move.
- **English and Spanish do not move** (D6): no English or Spanish pack carries the section, so
  en-fr, es-fr, es-en and en-es are byte for byte — measured, 62 tests in 10 suites without
  re-blessing, the four packs at their pins. **Spanish has the same defect, larger** — es-fr 98 keys,
  584 of 205,743 matches over 464,574 Spanish selections renamed (`hacer tiempo` « Atermoyer » on
  « hace tiempo » 85), es-en 138 and 1,604 of 236,765 —, and es-fr is read: moving it is the owner's
  decision (open question 2).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A French pack keeps every headword of a key several reach* (the other
  headwords with their glosses and names, an optional section listing them, a core that predates it
  unchanged, `pack_version` kept and the pin moved, no English or Spanish pack carrying it).
- `lingua-analysis`: ADDED — *A French match is named by the headword written as the page* (the
  headword written most like the run among those whose pieces meet it, change 44's order between
  equals, its name and gloss reported).
- `lingua-decks-review`: ADDED — *Review reads a French expression card's gloss under its headword*.

No requirement is modified. The new ones narrow *A French pack keys its expressions as French is
read*, *French expressions are found on French's reading of a selection* and *Review finds a French
expression card by its name* (change 44), *Multi-word expression table* and *Expression lookup in a
phrase gloss* (`add-lingua-expression-table`), beside changes 44c's and 44d's *A French expression's
pieces meet the words as its headword writes them* and *A French expression's contractions meet the
words as its headword writes them*; those changes are open, so each new requirement says what it
takes the place of rather than MODIFY them. The section is additive and derived from the committed
tables, as change 44b's *Versioned pack container, keyed by language pair* allows without a new
`pack_version`. The changes whose requirements it builds on are in `archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-pack` — *new*: the builder keeps a French key's other headwords (`build_pack`,
    the `expr.others` section); `committed_tables.rs` checks them on the committed tables.
  - `crates/lingua-core` — *new*: `section::EXPR_OTHERS`, `Pack::expression_headwords` and the
    gloss of a named headword; the choice in `engine::match_expressions`; *consumed*:
    `headword_reading`, `meets_as_written`, `expression_for_run`.
  - `crates/lingua-wasm` — `readable_gloss` reads a French card under its headword; five phrase
    probes in the French scenario; `fr-en.golden` and `fr-es.golden` re-blessed.
  - `scripts/lingua-data/tables/fr-en/pin.json`, `tables/fr-es/pin.json` — the pack's sha256 and
    size.
  - `apps/lingua-extension` — *consumed*, its code unchanged; `word-card-fr-en.txt` and
    `word-card-fr-es.txt` gain the five cards, their specs' count 38 → 43.

  ID, Music, Live, the back office, the site, the backend, the Apple host app, the agent plugin
  (which studies no French) and every reducer and table are untouched.
- **Release.** Silent: no listed pair studies French. It lands before change 52 lists fr-en, so no
  reader is shown `ça ira` on « ça va ».
- **Compatibility.** The pack gains an optional section, read by no older core; no stored format,
  wire field or table moves. No reader studies French, so no status or card is under a French name.
- **Effort, against no programme estimate (outside the 57)**: 0.75–1.25 ideal day (design,
  *Effort*).
