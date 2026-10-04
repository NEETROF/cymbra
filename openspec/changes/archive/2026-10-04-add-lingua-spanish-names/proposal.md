# add-lingua-spanish-names — a novel's characters are not unknown words

## Why

The owner's dogfood (2026-10-04) measured the Spanish words with no French gloss in their books.
Character names topped the list:
- Nela, 338 times in Marianela;
- Augusto (358) and Eugenia (211) in Niebla;
- Rocinante, Camila, Lotario and Anselmo in Don Quijote.

They are about 30 % of the words without a gloss in Marianela and Niebla.

The analyser sets a capitalised word aside as a proper noun only when the lexicon does not know it.
These names are words of the lexicon: `augusto` is an adjective, and wordfreq ranks `nela`. They have
no gloss. So every occurrence is underlined as unknown, counts against the reader's percentage, and
opens a card that says « Pas de traduction dans le pack. ».

The owner chose the rule: a word the book almost always writes with a capital is treated as a name.

A finding of the Spanish programme's dogfood, not a numbered change of
`docs/lingua/spanish-programme.md`.

## What Changes

- **A Spanish document's names**: a form is set aside like a proper noun outside the lexicon when, in
  the document analysed (a page, or a section of a book), it is:
  - never written in lowercase;
  - capitalised at least once in mid-sentence, after a word, a comma or a semicolon;
  - the form of a dictionary form the pack does not gloss.

  Every occurrence of it is then set aside, at the head of a sentence too.
- **What stays a word**:
  - a name the pack glosses (`Dios`);
  - a form also written in lowercase (`el augusto monarca`);
  - a word that is only capitalised at the head of sentences;
  - a word the reader has marked.
- **The Spanish analyser version becomes `1.2.0`**, since the classification changes. The es-fr pack is
  built with it.
- **English does not change.** Its analyser, its version and its baseline stay as they are.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *A Spanish document's names are set aside*. *Known-token percentage*,
  which excludes out-of-lexicon proper nouns, is not modified: the names join them.

## Impact

- **Code.**
  - `crates/lingua-core`: the page analysis computes the document's names, and the Spanish analyser
    version moves.
  - The es-fr tables' manifest carries the new version, and `pin.json` records the pack. No table
    content changes.
- **Measured** on the owner's books with the real engine, against the tables of the same day:

  | Book | occurrences without a gloss, before → after |
  |---|---|
  | Marianela | 1,479 → 1,047 (−29 %) |
  | Niebla | 1,894 → 1,220 (−36 %) |
  | Lazarillo | 797 → 774 |
  | Don Quijote | 15,210 → 13,305 (−13 %) |

  The words set aside are the books' characters and places.
- **Products.** Cymbra Lingua only, once the es-fr pack ships (`enable-lingua-spanish`): no extension
  code changes, since it already sets the proper-noun class aside. No server or proto change.
