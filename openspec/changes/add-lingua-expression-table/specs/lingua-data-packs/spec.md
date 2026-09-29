## ADDED Requirements

### Requirement: Multi-word expression table
A pack built for a pair whose sources hold multi-word entries SHALL carry an expression table: those entries, from the same licence-clean dictionary source as its glosses, each keyed by the sequence of dictionary forms of its words and holding one native-language gloss.
The keys SHALL be produced by the core's own lemmatiser at build time, against the lexicon
that build assembled, so that a key is exactly what the reader's analysis produces from the
words on the page. An entry SHALL be left out when any of its words is absent from that
lexicon, when its headword falls outside the character set the pair's reducer accepts, when it is a proper noun
only, or when no sense survives the source's form-of filtering. Where two entries reach one
key, the entry whose headword already is the key SHALL keep it. The table is optional and
additive: a pack without it loads, a pack with it loads on a core that does not read it, and
its presence SHALL NOT change `analyzer_version`.

#### Scenario: An inflected spelling and its dictionary form reach one key
- **WHEN** the source holds both `breaking point` and `break point`, each with its own gloss
- **THEN** the table holds one entry, keyed `break point`, carrying the gloss of the entry whose headword is that key

#### Scenario: A word the lexicon does not hold
- **WHEN** an expression holds a word the pack's lexicon does not hold
- **THEN** that expression is left out of the table

#### Scenario: A pack built without the table
- **WHEN** a pair's sources carry no usable multi-word entry
- **THEN** the pack is built with no expression section and the core reports no expression for any selection

#### Scenario: An older core reads a pack that has the table
- **WHEN** a core built before this change loads a pack carrying the expression section
- **THEN** the pack loads and behaves as it did, the section being ignored
