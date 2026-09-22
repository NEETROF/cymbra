## ADDED Requirements

### Requirement: Multi-word expression table
A pack built for a pair whose sources hold multi-word entries SHALL carry an expression table: those entries, from the same licence-clean dictionary source as its glosses, each keyed by the sequence of dictionary forms of its words and holding one native-language gloss.
The keys SHALL be produced by the core's own lemmatiser at build time, against the lexicon
that build assembled, so that a key is exactly what the reader's analysis produces from the
words on the page. An entry SHALL be left out when any of its words is absent from that
lexicon, when its headword is not of the studied language's script, when it is a proper noun
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

## MODIFIED Requirements

### Requirement: Versioned pack container, keyed by language pair
A pack SHALL be a single versioned container, keyed by pair (studied language → native language), holding: metadata (the pair, `pack_version`, the compatible `analyzer_version`, licences), a form→lemma FST, a frequency table (ranks), compressed glosses indexed by lemma, an optional per-lemma CEFR level table (present for pairs that have licence-clean CEFR data, absent otherwise), an optional multi-word expression table, and a NOTICE file. The core SHALL refuse a pack whose analyser version is incompatible. A table that only a new interface reads is additive: it SHALL be optional, a core that does not know it SHALL ignore it, and adding it SHALL bump `pack_version` and leave `analyzer_version` alone.

#### Scenario: Loading the EN→FR pack
- **WHEN** the extension starts with the (en → fr) pack embedded
- **THEN** the core exposes lemmatisation, frequency ranks, French glosses, CEFR levels and expressions for English

#### Scenario: Pair without CEFR data
- **WHEN** a pack for a pair with no licence-clean CEFR data is loaded
- **THEN** it loads with no level table and the core reports levels as unavailable for that language

#### Scenario: Incompatible pack
- **WHEN** a pack declares an `analyzer_version` incompatible with the core
- **THEN** loading fails with an explicit error and no partial analysis is produced

#### Scenario: A pack whose only new table is additive
- **WHEN** a pack is rebuilt with an expression table and no other change
- **THEN** its `analyzer_version` is the one the core already accepted, and its `pack_version` is new

### Requirement: Size budget
The (en → fr) pack embedded in the extension SHALL stay under 5 MiB (5 × 1024 × 1024 bytes, the figure the builder enforces), covering every table it carries: the FST, the frequencies, the compressed glosses and the optional level and expression tables. If it goes over, the build SHALL fail naming what to reduce, and the remedy SHALL take from the optional tables first — the expressions, longest entries then rarest — then from gloss coverage, never from the FST or the frequencies, which every page analysis depends on.

#### Scenario: Arbitrating size
- **WHEN** `gloss.zst` pushes the build past the budget
- **THEN** the build fails, telling the operator to reduce the number of glossed lemmas

#### Scenario: The expression table is what pushes the pack over
- **WHEN** the expression table takes a pack past the budget
- **THEN** the build fails and names the expression table as what must be reduced
