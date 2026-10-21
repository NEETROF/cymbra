# Spec Delta

## ADDED Requirements

### Requirement: A pack assembles from a base and a layer
A pack SHALL also be built as two containers of the same envelope: a *base*, holding a studied language's sections — the form→lemma FST, the lemmas, the frequency ranks, the level table, the paradigms and the dictionary words when the language has them, and its tag pool, which SHALL be the language's pinned pool followed by the tags of its readings outside it and nothing of any pair's senses — with metadata naming the studied language, the compatible `analyzer_version`, a `base_version` and whether its levels are estimated; and a *layer*, holding a pair's sections — the glosses, their sense runs, the tags only those senses carry, the expression tables when the pair has them, and the pair's NOTICE — with metadata naming the pair, `pack_version`, the pair's licences and the `base_version` of the base it was built against. The engine SHALL assemble a base and a layer into the pack their whole container holds: every named section with the same bytes, the tag pool being the base's followed by the layer's sense tags as a whole pack lays its pool out, the same studied and native languages, `pack_version`, `analyzer_version`, licences, levels and NOTICE, so that whatever reads a pack reads an assembled one alike. The engine SHALL refuse to assemble a layer with a base of another studied language, and every reader of a whole pack or of its metadata SHALL refuse a base or a layer as not a whole pack, naming the part and the one it needs; a plugin that follows the packs of a directory then skips it as a pack that cannot be loaded (*The languages followed are the packs installed*), naming the file. A base and a layer SHALL be built only from inputs that carry the studied language's pinned tag pool, the build naming the pool file when they do not; a base SHALL hold its tag pool whenever its language has a pinned pool, readings or not. A whole pack SHALL keep loading as before, with its bytes.

#### Scenario: The assembled pack is the whole pack
- **WHEN** en-fr is built whole, and as a base and a layer, from the same committed tables
- **THEN** the assembled pack holds every named section of the whole pack with the same bytes, reports the same pair, `pack_version`, `analyzer_version`, licences, levels and NOTICE, and the English invariance baseline passes on either

#### Scenario: The pool is the base's and the layer's
- **WHEN** en-es is built as a base and a layer from the committed tables
- **THEN** the base's pool is English's pinned pool and its readings' tags, the layer's sense tags are `NUM` alone, and the assembled pool is byte for byte the whole en-es pack's

#### Scenario: A layer of another language
- **WHEN** the es-fr layer is assembled with the English base
- **THEN** assembly fails with an explicit error naming both languages, and no partial pack is produced

#### Scenario: A part given to a reader of whole packs
- **WHEN** a base, or a layer, is given to the loader of a whole pack, or to the reader of a pack's studied language or pair
- **THEN** each refuses it with an explicit error naming the part and the other part it needs, and a plugin that follows the packs of a directory skips the file as one that cannot be loaded, names it, and follows the others

#### Scenario: The whole pack still loads
- **WHEN** the en-fr pack built whole from its committed tables is loaded by this engine
- **THEN** it loads as before, and the agent plugin's `pack.lingua` is read unchanged

#### Scenario: Inputs without a pinned pool
- **WHEN** a pair's inputs carry no pinned tag pool, as a test fixture's do
- **THEN** the split build fails naming the pool file, and the whole pack builds from them as before, byte for byte

### Requirement: A layer is built against its base
A base's `base_version` SHALL be a digest of the base container's bytes from its section count to its end, written by the builder, so that any reader of the container can recompute it; a layer SHALL carry the `base_version` of the base it was built with, and the builder SHALL write both parts from one run over one pair's inputs, so that a layer never names a base it was not built with. The engine SHALL refuse to assemble a layer whose `base_version` is not the base's, naming both versions. A whole pack SHALL carry no `base_version`.

#### Scenario: A layer and its base
- **WHEN** the en-es base and layer are built in one run and assembled
- **THEN** they assemble, and the layer's `base_version` is the base's

#### Scenario: A layer built against another base
- **WHEN** a layer is assembled with a base whose `base_version` differs from the one the layer names
- **THEN** assembly fails with an explicit error naming both versions, and no partial pack is produced

### Requirement: One base per studied language
The base built from any pair's committed tables SHALL be byte-identical to the base built from the reference pair's of the same studied language, metadata included; the checks over the committed tables SHALL fail, naming both pairs and what differs, when it is not. The parts of every committed pair SHALL each stay under the size budget.

#### Scenario: One English base
- **WHEN** en-fr's and en-es's bases are built from the committed tables
- **THEN** they are byte-identical, and so are es-fr's and es-en's, and fr-en's and fr-es's

#### Scenario: A pair whose manifest disagrees with its language's reference
- **WHEN** a second pair studying Spanish declares its levels estimated otherwise than es-fr's manifest does
- **THEN** the checks fail, naming both pairs and the field

#### Scenario: Every part under budget
- **WHEN** the six committed pairs are built as bases and layers
- **THEN** every base and every layer is under 5 MiB

## MODIFIED Requirements

### Requirement: Versioned pack container, keyed by language pair
A pack SHALL be a single versioned container, keyed by pair (studied language → native language), holding: metadata (the pair, `pack_version`, the compatible `analyzer_version`, licences), a form→lemma FST, a frequency table (ranks), compressed glosses indexed by lemma, an optional per-lemma CEFR level table (present for pairs that have licence-clean CEFR data, absent otherwise), an optional multi-word expression table, optional grammar tables, and a NOTICE file. The same content SHALL also be built as two containers of that envelope — a base per studied language and a layer per pair — which the engine assembles at load into the pack described here (*A pack assembles from a base and a layer*). The core SHALL refuse a pack whose analyser version is incompatible. A table that only a new interface reads is additive: it SHALL be optional, a core that does not know it SHALL ignore it, and adding it SHALL leave `analyzer_version` alone. A table added with new tables SHALL bump `pack_version`, as new tables do (*A pack says which dictionary it is*); a section the builder derives from tables already committed — an expression table's names — SHALL leave `pack_version` alone, the pack's new sha256 and size being recorded in its pin (*Reproducible offline build*).

#### Scenario: Loading the EN→FR pack
- **WHEN** the extension starts with the (en → fr) pack embedded
- **THEN** the core exposes lemmatisation, frequency ranks, French glosses, CEFR levels, expressions and word grammar for English

#### Scenario: Pair without CEFR data
- **WHEN** a pack for a pair with no licence-clean CEFR data is loaded
- **THEN** it loads with no level table and the core reports levels as unavailable for that language

#### Scenario: Incompatible pack
- **WHEN** a pack declares an `analyzer_version` incompatible with the core
- **THEN** loading fails with an explicit error and no partial analysis is produced

#### Scenario: A pack whose only new table is additive
- **WHEN** a pack is rebuilt with an expression table and no other change
- **THEN** its `analyzer_version` is the one the core already accepted, and its `pack_version` is new

#### Scenario: A pack whose new tables are grammar tables
- **WHEN** a pack is rebuilt with grammar tables and no other change
- **THEN** its `analyzer_version` is the one the core already accepted, and its `pack_version` is new

#### Scenario: A section derived from committed tables
- **WHEN** the es-fr pack is rebuilt with its expressions' names from its committed tables and no other change
- **THEN** its `analyzer_version` and its `pack_version` are the ones it had, and `tables/es-fr/pin.json` records its new sha256 and size

#### Scenario: The same pack, whole or in parts
- **WHEN** a pair is built whole and as a base and a layer from the same tables
- **THEN** the whole pack and the assembled parts hold every named section with the same bytes, the tag pool included

### Requirement: Size budget
The (en → fr) pack embedded in the extension SHALL stay under 5 MiB (5 × 1024 × 1024 bytes, the figure the builder enforces), covering every table it carries: the FST, the frequencies, the compressed glosses and the optional level, expression and grammar tables. A base and a layer SHALL each stay under the same budget. If a pack or a part goes over, the build SHALL fail naming what to reduce, and the remedy SHALL take from the optional tables first — the expressions, longest entries then rarest, then the readings of the rarest dictionary forms — then from gloss coverage, never from the FST or the frequencies, which every page analysis depends on; for a part, the remedy SHALL be one the part carries: a layer's expressions, then its glosses; a base's readings — a base carrying no readings that exceeds the budget SHALL fail naming no remedy, its forms and frequencies being what every analysis depends on. The part of speech of a gloss's senses follows its gloss, and SHALL leave the pack only with it.

#### Scenario: Arbitrating size
- **WHEN** `gloss.zst` pushes a pack carrying no expression or grammar table past the budget
- **THEN** the build fails, telling the operator to reduce the number of glossed lemmas

#### Scenario: The expression table is what pushes the pack over
- **WHEN** the expression table takes a pack past the budget
- **THEN** the build fails and names the expression table as what must be reduced

#### Scenario: The grammar readings are what push the pack over
- **WHEN** a pack carrying no expression table is taken past the budget by its grammar readings
- **THEN** the build fails and names the readings of the rarest dictionary forms as what must be reduced

#### Scenario: A part over budget
- **WHEN** a layer's expression table takes the layer past the budget, or a base's readings take the base past it
- **THEN** the build fails, names the part and what it must reduce — the layer's expression table, the base's readings — and writes neither part
