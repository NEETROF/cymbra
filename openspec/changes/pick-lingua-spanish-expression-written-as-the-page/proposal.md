# pick-lingua-spanish-expression-written-as-the-page — « hace tiempo » answers `hace tiempo`, not `hacer tiempo`

## Why

Row 44g of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside its 57 changes. Change 44b (`add-lingua-spanish-expression-keys`, merged as #852, not yet
released) keys a Spanish expression by what Spanish's analysis reads in its headword and names it by
the headword that wins its key. Two headwords whose words share their dictionary forms share the key,
and the pack keeps one: es-fr's `hacer tiempo` (« Atermoyer », to play for time) and `hace tiempo`
(« Jadis, belle lurette », long ago) are both `hacer tiempo`, so **« hace tiempo » is answered
`hacer tiempo`, « Atermoyer »** — 85 times on the corpus, 86 with « Hace tiempos ». Change 44e
(`pick-lingua-expression-written-as-the-page`, proposal #887, being implemented) fixes the same defect
in French (« ça va » answered `ça ira`) and left Spanish out (its D6), because es-fr is read. The owner
decided on 2026-10-10 (44e's open question 2): **Spanish gets the same rule, in a change of its own,
right after 44e and before the release that brings es-fr's readers 44b's names** (44b's task 6.4), so
no reader ever sets a status on a name that later moves. This is that change.

Measured on es-fr's and es-en's committed tables and change 44d's 464,574 Spanish selections
(Tatoeba, Spanish Wikipedia, UD Spanish, five Gutenberg books, change 3's pages: 4,160,681 tokens):
- **es-fr: 98 keys several headwords reach** (99 other headwords, `echar a perder`, `echado a perder`
  and `echarse a perder` on one), **es-en 138** (141). The other headwords are not in the pack today.
- **4,418 of es-fr's 205,743 matches fall on such a key, and 584 are named by a headword the page does
  not write**: `un poco` on « un poquito » 111, `hacer tiempo` on « hace tiempo » 86, `buen día` on
  « buenos días » 79, `red social` on « redes sociales » 59. es-en: 12,659 of 236,765 and 1,598
  (`medio de comunicación` on « medios de comunicación » 101, `no ser nada` on « no es nada » 99).
- 44e's « closest spelling » rule misfires in Spanish where French had no case: es-en's
  `tenerlos puestos` (« to have balls ») is closer to « tenía puestos unos audífonos » than
  `tener puesto` (« to wear »), its `puestos` written as the page — 6 times on the corpus.

## What Changes

- **A Spanish pack keeps every headword of a key** (design D1): 44e's builder rule and its optional
  `expr.others` section, opened from French to Spanish. es-fr 2,231,951 → 2,234,330 B (+2,379),
  es-en 2,608,413 → 2,612,075 B (+3,662); `pack_version` and `FORMAT_VERSION` stay, the two pins
  move. A core that predates the section reads the packs as today.
- **A Spanish match is named by the headword written most like the page** (D2), 44e's choice for
  Spanish: « hace tiempo » answers `hace tiempo`, « buenos días » `buenos días`, « dando vueltas »
  `dar vueltas`. On the corpus: es-fr 584 matches renamed, es-en 1,598, none gained or lost.
- **A headword written with a clitic pronoun is chosen only where the page writes one** (D3, the
  Spanish guard): `tenerlos puestos`, `hacerse ilusiones`, `puedes ayudarme` are weighed only where
  a clitic pronoun is attached to one of the run's words (« hacerme ilusiones ») or is the page's word
  just before the run (« No te hagas ilusiones »); where no headword of the key qualifies, all are
  weighed, as today. « tenía puestos unos audífonos » answers `tener puesto`; es-en 6 matches move
  against 44e's rule alone, es-fr none; no match is gained or lost.
- **A lemma chain settled before stays a reader's own only where it names no headword** (D4): 44b's
  rule keeping a status set on a lemma chain (`tener en contar`) holds, except where the chain is a
  headword's own name (`hacer tiempo`, `un poco`, `red social`: 71 of es-fr's 98 shared keys): such a
  record is that headword's, so a status on `un poco` is never read on « un poquito ».
- **Review reads a Spanish card's gloss under its own headword** (D5): a card `hace tiempo` reviewed
  in another language than its gloss shows `hace tiempo`'s gloss, not `hacer tiempo`'s.
- **What moves** (D6): `es-fr.golden` its `pack` line and « darse cuenta » (`dar cuenta` « Rendre
  compte » → `darse cuenta` « Se rendre compte »), `es-en.golden` its `pack` line, both five phrase
  probes added — « hace tiempo », « buenos días », « un poquito », « No te hagas ilusiones. », « El
  guardaespaldas tenía puestos unos audífonos. » —; `fr-en.golden` its `beside es-en` line alone;
  `tables/es-fr/pin.json` and `tables/es-en/pin.json`; the pack sizes the es-fr and es-en READMEs
  and `SOURCES.md` state. Spanish's analyser version, `pack_version`, the tables,
  `word-card-es-en.txt` and every other snapshot do not move.
- **English and French do not move** (D7): no English pack carries the section; en-fr, en-es,
  fr-en's and fr-es's output are byte for byte.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A Spanish pack keeps every headword of a key several reach* (44e's
  rule and section for Spanish packs, `pack_version` kept and the pins moved, no English pack
  carrying it).
- `lingua-analysis`: ADDED — *A Spanish match is named by the headword written as the page* (44e's
  choice, the clitic guard, a lemma chain that names a headword read as that headword's).
- `lingua-decks-review`: ADDED — *Review reads a Spanish expression card's gloss under its own
  headword*.

No requirement is modified. The new ones narrow 44e's three requirements (*A French pack keeps every
headword of a key several reach*, *A French match is named by the headword written as the page* and
*Review reads a French expression card's gloss under its own headword*, each of which leaves Spanish
as it was), 44b's four (*A Spanish pack keys its expressions as Spanish is read*, *Spanish expressions
are found on Spanish's reading of a selection*, *A Spanish expression settled under its lemmas keeps
that key*, *Review finds a Spanish expression card by its name*) and `add-lingua-expression-table`'s
*Multi-word expression table* and *Expression lookup in a phrase gloss*; those changes are open, so
each new requirement says what it takes the place of rather than MODIFY them. The section is
additive and derived from the committed tables, as 44b's *Versioned pack container, keyed by
language pair* allows without a new `pack_version`. The changes whose requirements it builds on are in
`archiveAfter`.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-pack` — `build_pack` keeps a Spanish key's other headwords (44e's French
    condition opened to Spanish); `committed_tables.rs` checks them on es-fr's and es-en's tables.
  - `crates/lingua-core` — *new*: Spanish's clitic test (`analysis/spanish.rs`, the cascade's own
    enclitic rule), the Spanish arm of the choice in `engine::match_expressions`, the lemma-chain
    exception in `settled_chain`; *consumed*: 44e's `section::EXPR_OTHERS`,
    `Pack::expression_headwords`, `Pack::expression_gloss_named`, `headword_reading`.
  - `crates/lingua-wasm` — `readable_gloss` reads a Spanish card under its headword; five phrase
    probes in the Spanish scenario; `es-fr.golden`, `es-en.golden` and `fr-en.golden` re-blessed.
  - `scripts/lingua-data/tables/es-fr/pin.json`, `tables/es-en/pin.json` — the pack's sha256 and
    size; `tables/es-fr/README.md`, `tables/es-en/README.md` and `SOURCES.md` — the pack's size.
  - `apps/lingua-extension` — *consumed*, its code and snapshots unchanged.

  ID, Music, Live, the back office, the site, the backend, the Apple host app (its embedded packs
  come with the release), the agent plugin (which reads no expression) and every reducer and table
  are untouched.
- **Release.** es-fr is read: merging changes no reader's extension. The change reaches es-fr's
  readers with the extension release the owner decides — the one that brings 44b's names (44b's task
  6.4), which waits for this change (task 6.3) —, after approving the re-bless (6.1). es-en is not
  listed yet.
- **Compatibility.** The packs gain an optional section an older core ignores; no stored format, wire
  field, table or `pack_version` moves, and no record is rewritten. A status set before the release on
  a lemma chain that is no headword's name keeps answering (44b's D6); one set on a chain that is a
  headword's name is that headword's (D4, open question 2).
- **Effort, against no programme estimate (outside the 57)**: 0.5–1 ideal day (design, *Effort*).
