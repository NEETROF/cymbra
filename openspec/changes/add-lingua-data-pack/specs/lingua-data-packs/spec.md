# lingua-data-packs — linguistic data packs

## ADDED Requirements

### Requirement: Versioned pack container, keyed by language pair
A pack SHALL be a single versioned container, keyed by pair (studied language → native language), holding: metadata (the pair, `pack_version`, the compatible `analyzer_version`, licences), a form→lemma FST, a frequency table (ranks), compressed glosses indexed by lemma, and a NOTICE file. The core SHALL refuse a pack whose analyser version is incompatible.

#### Scenario: Loading the EN→FR pack
- **WHEN** the extension starts with the (en → fr) pack embedded
- **THEN** the core exposes lemmatisation, frequency ranks and French glosses for English

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
