# Spec Delta

## ADDED Requirements

### Requirement: A pack assembles from a core and a layer
A pack SHALL also be built as two containers of the same envelope: a *core*, holding a studied language's sections — the form→lemma FST, the lemmas, the frequency ranks, the level table, the tag pool, the paradigms and the dictionary words when the language has them — with a metadata naming the studied language, the compatible `analyzer_version`, a `core_version` and whether its levels are estimated; and a *layer*, holding a pair's sections — the glosses, their sense runs, the expression tables when the pair has them, and the pair's NOTICE — with a metadata naming the pair, `pack_version`, the pair's licences and the `core_version` of the core it was built against. The core SHALL assemble a core and a layer into the pack their whole container holds: the same sections with the same bytes, the same studied and native languages, `pack_version`, `analyzer_version`, licences, levels and NOTICE, so that whatever reads a pack reads an assembled one alike. The core SHALL refuse to assemble a layer with a core of another studied language, SHALL refuse a core or a layer loaded alone as a whole pack, and SHALL name the part and what is missing in each refusal. A whole pack SHALL keep loading as before, with its bytes.

#### Scenario: The assembled pack is the whole pack
- **WHEN** en-fr is built whole, and as a core and a layer, from the same committed tables
- **THEN** the assembled pack holds every section of the whole pack with the same bytes, reports the same pair, `pack_version`, `analyzer_version`, licences, levels and NOTICE, and the English invariance baseline passes on either

#### Scenario: A layer of another language
- **WHEN** the es-fr layer is assembled with the English core
- **THEN** assembly fails with an explicit error naming both languages, and no partial pack is produced

#### Scenario: A part loaded as a whole pack
- **WHEN** a core, or a layer, is given to the loader of a whole pack
- **THEN** loading fails with an explicit error naming the part and the other part it needs

#### Scenario: The whole pack still loads
- **WHEN** the en-fr pack built whole from its committed tables is loaded by this core
- **THEN** it loads as before, and the agent plugin's `pack.lingua` is read unchanged

### Requirement: A layer is built against its core
A core's `core_version` SHALL be a digest of the core's own bytes, written by the builder; a layer SHALL carry the `core_version` of the core it was built with, and the builder SHALL write both parts from one run over one pair's inputs, so that a layer never names a core it was not built with. The core SHALL refuse to assemble a layer whose `core_version` is not the core's, naming both versions. A whole pack SHALL carry no `core_version`.

#### Scenario: A layer and its core
- **WHEN** the en-es core and layer are built in one run and assembled
- **THEN** they assemble, and the layer's `core_version` is the core's

#### Scenario: A layer built against another core
- **WHEN** a layer is assembled with a core whose `core_version` differs from the one the layer names
- **THEN** assembly fails with an explicit error naming both versions, and no partial pack is produced

### Requirement: One core per studied language
The core built from any pair's committed tables SHALL be byte-identical to the core built from the reference pair's of the same studied language, metadata included; the checks over the committed tables SHALL fail, naming both pairs and what differs, when it is not. The parts of every committed pair SHALL each stay under the size budget.

#### Scenario: One English core
- **WHEN** en-fr's and en-es's cores are built from the committed tables
- **THEN** they are byte-identical, and so are es-fr's and es-en's, and fr-en's and fr-es's

#### Scenario: A pair whose manifest disagrees with its language's reference
- **WHEN** a second pair studying Spanish declares its levels estimated otherwise than es-fr's manifest does
- **THEN** the checks fail, naming both pairs and the field

#### Scenario: Every part under budget
- **WHEN** the six committed pairs are built as cores and layers
- **THEN** every core and every layer is under 5 MiB

## MODIFIED Requirements

### Requirement: Versioned pack container, keyed by language pair
A pack SHALL be a single versioned container, keyed by pair (studied language → native language), holding: metadata (the pair, `pack_version`, the compatible `analyzer_version`, licences), a form→lemma FST, a frequency table (ranks), compressed glosses indexed by lemma, an optional per-lemma CEFR level table (present for pairs that have licence-clean CEFR data, absent otherwise), an optional multi-word expression table, optional grammar tables, and a NOTICE file. The same content SHALL also be built as two containers of that envelope — a core per studied language and a layer per pair — which the core assembles at load into the pack described here (*A pack assembles from a core and a layer*). The core SHALL refuse a pack whose analyser version is incompatible. A table that only a new interface reads is additive: it SHALL be optional, a core that does not know it SHALL ignore it, and adding it SHALL leave `analyzer_version` alone. A table added with new tables SHALL bump `pack_version`, as new tables do (*A pack says which dictionary it is*); a section the builder derives from tables already committed — an expression table's names — SHALL leave `pack_version` alone, the pack's new sha256 and size being recorded in its pin (*Reproducible offline build*).

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
- **WHEN** a pair is built whole and as a core and a layer from the same tables
- **THEN** the whole pack and the assembled parts hold the same sections with the same bytes

### Requirement: Size budget
The (en → fr) pack embedded in the extension SHALL stay under 5 MiB (5 × 1024 × 1024 bytes, the figure the builder enforces), covering every table it carries: the FST, the frequencies, the compressed glosses and the optional level, expression and grammar tables. A core and a layer SHALL each stay under the same budget. If a pack or a part goes over, the build SHALL fail naming what to reduce, and the remedy SHALL take from the optional tables first — the expressions, longest entries then rarest, then the readings of the rarest dictionary forms — then from gloss coverage, never from the FST or the frequencies, which every page analysis depends on. The part of speech of a gloss's senses follows its gloss, and SHALL leave the pack only with it.

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
- **WHEN** a layer's expression table takes the layer past the budget, or a core's readings take the core past it
- **THEN** the build fails, names the part and what must be reduced, and writes neither part

### Requirement: A form's readings do not depend on the pack's native language
Two packs of one studied language built from the same forms, frequencies and readings SHALL store those readings byte for byte alike, whatever tags their glosses' senses carry.
A pack built with its studied language's pinned tag pool SHALL hold, in this order, that pool, then
the tags of its readings outside it, each part sorted; a sense run tagged outside both SHALL fail the
build, naming the tag and the studied language's pinned pool file, rather than be appended to the
pool, so that a pool belongs to the studied language alone (*One core per studied language*).
English's pinned pool SHALL be the pool the en-fr pack carried before this change with `NUM`
appended — the one tag a sense of en-es carries and no English reading does — and Spanish's and
French's SHALL be the pools the es-fr and fr-en packs carry, so that every id keeps its place and
every pair built from committed tables is built with its studied language's pinned pool. A pack
built without one, such as a test pack, SHALL keep a single sorted pool, as before. The checks SHALL
fail when two packs of one studied language, built from committed tables, store a form's readings
differently: when their pinned pools, their dictionary words or their studied tables — forms,
frequencies, levels and readings — differ.

#### Scenario: The shipped packs keep their pools
- **WHEN** the en-fr, es-fr and fr-en packs are built from their committed tables after this change
- **THEN** es-fr and fr-en carry the pools they carried before, with the sha256 their pins record; en-fr carries its pool with `NUM` as its last tag, every other tag at the id it had, the English invariance baseline passes unchanged, and its pin records the new sha256 and size

#### Scenario: A sense part of speech the first pack never used
- **WHEN** a pack studying English is built from the en-fr studied tables with a sense run tagged `NUM`
- **THEN** it builds, its readings and its pool are stored byte for byte as en-fr's, and the core reads that run as `NUM`

#### Scenario: A sense tag outside the pool
- **WHEN** a pack studying Spanish is built from the es-fr studied tables with a sense run tagged `SYM`, which Spanish's pinned pool and readings do not hold
- **THEN** the build fails, naming `SYM` and `tables/es/tags.tsv`, and writes no pack

#### Scenario: Fewer sense tags
- **WHEN** a pack studying Spanish is built from the es-fr studied tables with no sense run tagged `INTJ` or `X`
- **THEN** its readings and its pool are stored byte for byte as es-fr's

#### Scenario: A reading outside the pinned pool
- **WHEN** a test pack studying Spanish gives `me` a reading whose tag Spanish's pinned pool does not hold
- **THEN** it builds, and the core reads that reading back

#### Scenario: Two packs of one language store readings differently
- **WHEN** the committed tables of a second pair studying Spanish hold a `grammar.tsv`, `forms.tsv`, `freq.tsv` or `level.tsv` other than es-fr's
- **THEN** the checks fail, naming both pairs and the table
