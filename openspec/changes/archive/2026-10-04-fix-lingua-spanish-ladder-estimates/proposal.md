# fix-lingua-spanish-ladder-estimates — Spanish's « estimés » read like English's

## Why

The owner's dogfood (2026-10-04) compared the two ladders. English's « estimés » column, the
vocabulary a reader of each level typically has, grows as a reader's does: ≈ 1,300 at A2, 3,400 at
B1, 8,000 at B2, 16,000 at C1, 21,000 at C2. Spanish's read ≈ 1,100, 2,300, 4,500, 7,000, 8,000:
« du n'importe quoi ».

The computation is the same, `level_vocabulary`. The words below a level count as known, and each
frequency band's known share extends to its words that have no level. English's CEFR lists put
C1 and C2 words deep into the rare ones, so the share reaches the whole dictionary. Spanish's levels
are estimated (`add-lingua-spanish-levels`): a cut by frequency rank that ends at rank 12,069. So
each figure only restates the band below, and nothing beyond rank 12,069 has a level to extend
from.

The Spanish levels take English's level sizes (1,020 A1, 1,158 A2…). The owner asked for the Spanish
ladder to read like English's.

A finding of the Spanish programme's dogfood, not a numbered change of
`docs/lingua/spanish-programme.md`.

## What Changes

- **A ladder of estimated levels borrows English's « estimés ».** When a pack's levels are estimated
  and the English pack, whose levels come from CEFR lists, is loaded, `levelLadder` gives each level
  English's typical vocabulary and says where it came from (`typicalFrom: "en"`).
- **The legend says so**: « estimés : le vocabulaire qu'a en général un lecteur de ce niveau, repris
  de l'anglais, dont l'espagnol reprend les tailles de niveaux. »
- **English is unchanged**: its ladder answers exactly as before, so the English baseline does not
  move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED — *An estimated ladder shows English's typical vocabularies*.

## Impact

- **Code.**
  - `crates/lingua-wasm` `level_ladder`: the borrowing, and `typicalFrom`, present only when
    borrowed.
  - `apps/lingua-extension`: `LevelRow.typicalFrom`, the ladder's legend, and a label in
    `language-labels.ts`.

  No pack, analyser, server or proto change.
- **Products.** Cymbra Lingua: the Spanish ladder, once the es-fr pack ships (`enable-lingua-spanish`).
  English reads as before.
