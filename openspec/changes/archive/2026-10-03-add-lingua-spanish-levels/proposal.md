# add-lingua-spanish-levels — estimated CEFR levels for Spanish

## Why

A level table drives the reader's level declaration, the words presumed known below it, the
statistics' ladder and « Renforcer un niveau ». English has CEFR-J and Octanove's levels. No
Spanish CEFR list can be shipped: ELELex is non-commercial, the PCIC all rights reserved, and
CEFR-J Spanish unpublished. The licence requests are with the owner. Without a table, Spanish would
fall back to the frequency slider.

Decision D1 of the programme (`docs/lingua/spanish-programme.md`) settles it:
- Spanish gets **estimated levels**, a table derived from frequencies;
- it is labelled « niveau estimé » wherever a level is shown;
- if the derived scale is not monotone, it falls back to three bands.

Measured on English's 8,302 CEFR lemmas, with the derivation this change uses:
- **39.8 % exact, 82.6 % within one level**. The study found 40 % and 83 %.
- **The scale is monotone.** Median frequency ranks rise from A1 to C2: 929, 2,216, 4,055,
  8,165, 15,295, 26,559. The mean true level of each estimated level also rises: 1.67, 2.41,
  3.12, 3.89, 4.67, 5.03.

So the three-band fallback is not needed.

This is change 23, in G1, the internal Spanish build. It depends on D1, and only it does.

## What Changes

- **`reduce-es-fr.py` writes `level.tsv`.** The commonest lemmas, in rank order, take English's
  band sizes: 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1, 876 C2. A lemma with no French
  gloss, or only a proper noun's, is skipped, as a CEFR list would leave it out (`the`,
  `twitter`, `madrid`). That gives 8,302 lemmas, A1 from rank 1 to 1,086, C2 from 10,446 to
  12,069.
- **A pack says its levels are estimated.**
  - The manifest's meta carries `levels_estimated: true`.
  - `PackMeta` gains the flag as optional, absent when false, so en-fr's bytes do not move.
  - `Pack::levels_estimated()`, the engine's `levelsEstimated(language)`, and the extension's
    port and language views expose it.
- **The extension labels estimated levels.** For a language whose pack's levels are estimated:
  - the level titles read « estimé » (« Niveau d'espagnol estimé »);
  - Réglages, the statistics and onboarding say the levels are estimated from word frequency;
  - the ladder's « enseignés » column, which assumes teaching lists, becomes « courants »;
  - onboarding's confirmation reads « Niveau enregistré : B1 (estimé) ».

  English reads exactly as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *The Spanish pack's estimated levels* and *A pack says when its
  levels are estimated*.
- `lingua-browser-extension`: ADDED — *Estimated levels are labelled as such*.

## Impact

- **Products.** Cymbra Lingua:
  - `scripts/lingua-data/` (the reducer, the tables);
  - `crates/lingua-core` (`PackMeta`, `Pack`);
  - `crates/lingua-wasm` (`levelsEstimated`);
  - `apps/lingua-extension`: the port, the language labels, Réglages, the popup, the statistics
    and onboarding.

  The English baseline and the en-fr pack do not move. No package carries the es-fr pack until
  `enable-lingua-spanish`. The server, the protos, ID, Music, Live, the back office and the site
  are not affected.
- **Sources and licences**: none new. The levels derive from the tables the pack already holds.
- **Release.** G1, internal: nothing ships.
