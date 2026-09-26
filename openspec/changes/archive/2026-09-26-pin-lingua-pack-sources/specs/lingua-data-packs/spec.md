## MODIFIED Requirements

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

## ADDED Requirements

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
