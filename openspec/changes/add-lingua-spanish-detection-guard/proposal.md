# add-lingua-spanish-detection-guard — Catalan and Galician are not Spanish

## Why

whichlang, the detector that gates every block and votes on each document's language, has no
Catalan or Galician class. It reads many of their blocks as Spanish. Measured on Wikipedia extracts
(2026-10-03: 1,214 Catalan paragraphs, 318 Galician, 442 Spanish):
- **Catalan**: 50.8 % of the paragraphs read as Spanish, 65.0 % of the text.
- **Galician**: 27.0 % of the paragraphs, 21.6 % of the text.

For a reader of Spanish, a Catalan news site would be highlighted as broken Spanish: every word
unknown, the percentage meaningless, and words that do not exist in Spanish proposed for the deck.
The programme's study measured 40–55 % without a guard and ≈16–18 % with one built on function words
(`docs/lingua/spanish-programme.md`, Architecture: Detection). It asked that the remaining leak be
written into the spec rather than promised away.

This is change 19, in G1, the internal Spanish build. No reader studies Spanish yet, and English is
untouched.

## What Changes

- **A guard on Spanish detection.** A block whichlang reads as Spanish is not Spanish when its
  Catalan or Galician function words outnumber its Spanish ones. The tables are short and sorted:
  - Catalan: `amb`, `els`, `per`, `però`, `també`, `és`…, plus the elisions `l'`, `d'`, `s'`,
    `n'`;
  - Galician: `unha`, `non`, `polo`, `tamén`, `hai`, `xa`…;
  - Spanish: `y`, `los`, `las`, `por`, `muy`, `hay`, `fue`…, words neither neighbour uses.

  A tie, or no marker at all, stays Spanish.
- **Everywhere detection is used.** The guard applies to the block gate (`block_is_studied`) and to
  the document vote (`detect_document_language`): a guarded block neither counts as Spanish text nor
  votes for Spanish.
- **Measured with the guard**, on the same extracts:
  - Catalan: 8.5 % of the paragraphs and 1.4 % of the text still read as Spanish;
  - Galician: 6.6 % of the paragraphs and 0.7 % of the text;
  - Spanish: 416 of the 420 paragraphs whichlang read as Spanish are kept. The 4 it refused are
    Catalan and French bibliography entries and a list of Catalan articles.

  The remaining leak is short lines with no marker at all (captions, list items), and the spec says
  so.
- **Spanish's analyser version becomes `1.1.0`**: the guard changes which Spanish blocks are
  analysed. English stays `1.1.0`, and its baseline does not move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Catalan and Galician are not read as Spanish*. It archives after
  `add-lingua-spanish-analysis`, whose version requirement it follows.

## Impact

- **Products.** Cymbra Lingua's analysis core: `crates/lingua-core/src/analysis/language.rs` and the
  Spanish version. The extension, the agent and the back office consume it unchanged. No pack,
  server or proto change. ID, Music, Live and the site are not affected.
- **Release.** G1, internal. No reader studies Spanish, and the English baseline (S0) is unchanged.
