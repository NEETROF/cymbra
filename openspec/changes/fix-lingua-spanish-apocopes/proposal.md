# fix-lingua-spanish-apocopes — `bueno` has its gloss again

## Why

The owner's dogfood (2026-10-04) asked how often a Spanish word has no translation. Measured with the
real engine and the es-fr pack on their books (Marianela, Niebla, Lazarillo, Don Quijote):
- 99.7–99.8 % of the words among the 1,000 commonest are glossed;
- about one in four beyond the 5,000 commonest is not.

The commonest word without a gloss was `buen`: 559 times in Don Quijote, 61 in Niebla. `bueno`
lacked one too.

The cause is in the reduction. kaikki's entry for the apocope `buen` lists the full word as a row of
its table, tagged `standard`. The reducer read that row as an inflection, so `bueno` became a form of
`buen`. UD Spanish-GSD lemmatizes `bueno` as `buen` 26 times out of 27, so GSD's counts chose `buen`.
`buen` has no French gloss. The card of one of the commonest Spanish adjectives said « Pas de
traduction dans le pack. ». The same row made `malo` a form of the noun *mal*.

A finding of the Spanish programme's dogfood, not a numbered change of
`docs/lingua/spanish-programme.md`.

## What Changes

- **A `standard` row is no inflection.** The row naming the standard word an entry is a variant of is
  read neither as a form of the entry nor as a reading.
- **An adjective's or a determiner's apocope reads as its full word**:
  - `buen` → *bueno*, `gran` → *grande*, `primer` → *primero*;
  - `tercer` → *tercero*, `algún` → *alguno*, `cualquier` → *cualquiera*;
  - `san` → *santo*.

  kaikki tags the sense `apocopic` and names the full word in `alt_of`. Only its first target counts,
  and only that target's first word.
- **An adverb or a numeral kaikki calls apocopic stays a word of its own.** `muy` is « Très », not
  *mucho*, and `un` is the article, not *uno*. GSD lemmatizes both to their full words.
- **The es-fr tables are reduced again** from the pinned sources. `bueno` is glossed « Bon », and
  `malo` « Mauvais, méchant ». The gates still pass.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A Spanish apocope reads as its full word*.

## Impact

- **Data.** `scripts/lingua-data/reduce-es-fr.py` and every es-fr table re-reduced from the pinned
  snapshot; `pin.json` records the new rules and pack. `reduce_common.py` does not change, so en-fr
  does not move.
- **Measured.** On UD Spanish-PUD:
  - words resolved stay at 99.38 %;
  - content words taking PUD's lemma move from 95.92 % to 95.79 % (gate 93.5 %);
  - auxiliaries stay at 97.95 %.

  The content-word drop is PUD's own inconsistency: it keeps `gran` as its own lemma but takes
  `primer` to *primero*. On the owner's books, every word among the 1,000 commonest is now glossed,
  and the words without a gloss drop by 3.6 to 4.6 %.
- **Products.** Cymbra Lingua only, once the es-fr pack ships (`enable-lingua-spanish`). No
  extension, engine, server or proto change.
