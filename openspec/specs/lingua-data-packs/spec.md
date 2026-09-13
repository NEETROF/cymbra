# lingua-data-packs Specification

## Purpose
TBD - created by archiving change add-lingua-data-pack. Update Purpose after archive.
## Requirements
### Requirement: Versioned pack container, keyed by language pair
A pack SHALL be a single versioned container, keyed by pair (studied language → native language), holding: metadata (the pair, `pack_version`, the compatible `analyzer_version`, licences), a form→lemma FST, a frequency table (ranks), compressed glosses indexed by lemma, an optional per-lemma CEFR level table (present for pairs that have licence-clean CEFR data, absent otherwise), and a NOTICE file. The core SHALL refuse a pack whose analyser version is incompatible. Adding the CEFR level table changes the pack format and bumps `analyzer_version`.

#### Scenario: Loading the EN→FR pack
- **WHEN** the extension starts with the (en → fr) pack embedded
- **THEN** the core exposes lemmatisation, frequency ranks, French glosses and CEFR levels for English

#### Scenario: Pair without CEFR data
- **WHEN** a pack for a pair with no licence-clean CEFR data is loaded
- **THEN** it loads with no level table and the core reports levels as unavailable for that language

#### Scenario: Incompatible pack
- **WHEN** a pack declares an `analyzer_version` incompatible with the core
- **THEN** loading fails with an explicit error and no partial analysis is produced

### Requirement: Licence hygiene
The build pipeline SHALL accept only sources whose licence permits commercial use (AGID, WordNet, wordfreq CC BY-SA, kaikki CC BY-SA) and SHALL reject any GPL, AGPL or non-commercial source (documented denylist). Every pack SHALL embed the complete stack of notices, and the user interface SHALL expose an attributions page.

#### Scenario: Notices embedded
- **WHEN** a pack is built
- **THEN** its NOTICE carries the AGID attributions (including its upstream stack, WordNet among them), wordfreq and kaikki, and the extension's "Attributions" page displays them

### Requirement: Reproducible offline build
Packs SHALL be built by a scripted, reproducible pipeline from dated sources, and the raw data SHALL NOT be committed. Two runs of the pipeline over the same sources SHALL produce identical packs.

#### Scenario: Rebuilding the pack
- **WHEN** the pipeline is re-run over the same source files
- **THEN** the resulting pack is byte-for-byte identical to the previous one

### Requirement: Size budget
The (en → fr) pack embedded in the extension SHALL stay under 5 MB (FST + frequencies + compressed glosses). If it goes over, the build SHALL fail, and the remedy SHALL reduce gloss coverage, never the FST or the frequencies.

#### Scenario: Arbitrating size
- **WHEN** `gloss.zst` pushes the build past 5 MB
- **THEN** the build fails, telling the operator to reduce the number of glossed lemmas

### Requirement: CEFR level source and enumeration by band
The English pack's CEFR levels SHALL be built from licence-clean sources — CEFR-J Wordlist
v1.6 for A1–B2 and Octanove Vocabulary Profile C1/C2 v1.0 for C1–C2 — joined to lemmas the
same way frequency and glosses are, with one CEFR level per lemma resolved by a defined
collapse rule when a source lists several. The build SHALL carry the required attributions
in NOTICE (the CEFR-J citation string; Octanove CC BY-SA 4.0 attribution, with share-alike
applied to the derived level table), and the licence guard SHALL admit these sources. The
core SHALL let a caller enumerate the lemmas (with glosses) of a given CEFR level, or of a
frequency-rank band when no CEFR data exists.

#### Scenario: Enumerating a level
- **WHEN** a caller requests the lemmas of level B1
- **THEN** it receives the B1 lemmas with their glosses, drawn from the pack

#### Scenario: Attribution carried in NOTICE
- **WHEN** the English pack is built with CEFR levels
- **THEN** NOTICE contains the CEFR-J citation and the Octanove CC BY-SA 4.0 attribution

