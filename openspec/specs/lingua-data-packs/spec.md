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
Packs SHALL be built by a scripted, reproducible pipeline from reduced tables committed in the repository, and a release SHALL build its pack only from those tables, never from live upstream sources.
The raw data and the built pack SHALL NOT be committed; the reduced tables SHALL be, under their own
licences, with the sha256 of the pack they build. The raw sources the tables were reduced from SHALL
be pinned — at a commit and by sha256 where they are hosted, or kept as a snapshot where they are
not stable — so the tables can be reduced again from the same bytes. Two runs of the pipeline over
the same tables SHALL produce identical packs, and every package of a release — Chromium, Firefox
and Safari — SHALL carry that same pack. A build whose pack does not match the recorded sha256 SHALL
fail, whether in a release or in a pull request's checks.

#### Scenario: Rebuilding the pack
- **WHEN** the pipeline is re-run over the same tables
- **THEN** the resulting pack is byte-for-byte identical to the previous one

#### Scenario: An upstream source is gone
- **WHEN** a raw source of the dictionary no longer answers on release day
- **THEN** the release builds its pack from the committed tables and is not affected

#### Scenario: One dictionary per release
- **WHEN** a version is released to the Chrome Web Store, addons.mozilla.org and the App Store
- **THEN** the three packages carry byte-identical packs

#### Scenario: The reviewer's rebuild
- **WHEN** a reviewer rebuilds the pack from the submitted source archive with no network
- **THEN** they obtain the pack the package carries, byte for byte

#### Scenario: A pull request that breaks the pack
- **WHEN** a pull request changes the builder, the tables or a dependency so that the pack's sha256 or its size budget no longer holds
- **THEN** its checks fail, before any release

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

### Requirement: A dictionary update is a reviewed decision
Live upstream sources SHALL be read only to propose new tables, and new tables SHALL reach a release only through a pull request a person opens and merges.
The proposal SHALL come with a report, against the committed tables, of the lemmas, glosses, levels
and expressions added, removed and changed, and of the pack's size against its budget. No workflow
SHALL open or approve that pull request. A change to the reduction rules SHALL be applied to the
pinned raw sources, so that its diff shows the rule change and no upstream change; tables whose
recorded reduction rules differ from the repository's SHALL fail the checks.

#### Scenario: Updating the dictionary
- **WHEN** a maintainer runs the update
- **THEN** new tables and a report of what they change are pushed to a branch, and releases keep the committed tables until a person merges a pull request from it

#### Scenario: Reduction rules changed
- **WHEN** the reducer is edited
- **THEN** the checks fail until the tables are reduced again from the pinned raw sources, and that pull request's diff holds only what the edit changes

### Requirement: Upstream breakage is detected before it is needed
The upstream sources SHALL be checked on a monthly schedule without changing anything, and a source that cannot be fetched, a reduction that fails, or a table that collapses SHALL fail that check visibly.

#### Scenario: A source moved
- **WHEN** the monthly check runs and a source no longer answers at its address, or its format changed so that a table loses most of its rows
- **THEN** the check fails and says which source, while releases keep building from the committed tables

#### Scenario: Nothing wrong upstream
- **WHEN** the monthly check runs and every source reduces
- **THEN** it reports how far upstream has drifted from the committed tables, and commits and publishes nothing

### Requirement: A pack says which dictionary it is
A pack's `pack_version` SHALL identify the snapshot of tables it was built from.

#### Scenario: Two releases, one dictionary
- **WHEN** two releases are built from the same committed tables
- **THEN** their packs report the same `pack_version`

#### Scenario: An updated dictionary
- **WHEN** a release is built after new tables were merged
- **THEN** its pack reports a different `pack_version`

