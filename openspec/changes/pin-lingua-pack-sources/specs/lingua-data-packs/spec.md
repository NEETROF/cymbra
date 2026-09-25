## MODIFIED Requirements

### Requirement: Reproducible offline build
Packs SHALL be built by a scripted, reproducible pipeline from a pinned, dated snapshot, and a release SHALL build its pack only from that snapshot, never from live upstream sources.
The snapshot SHALL be the reduced tables the pack is built from, kept outside the repository and
identified in it by sha256, together with the sha256 of the pack they build. The raw data SHALL NOT
be committed; the address, date and sha256 of each raw source a snapshot was reduced from SHALL be
recorded with it. Two runs of the pipeline over the same snapshot SHALL produce identical packs, and
every package of a release — Chromium, Firefox and Safari — SHALL carry that same pack. A release
whose pack does not match the pinned sha256, or whose snapshot cannot be fetched, SHALL fail.

#### Scenario: Rebuilding the pack
- **WHEN** the pipeline is re-run over the same snapshot
- **THEN** the resulting pack is byte-for-byte identical to the previous one

#### Scenario: An upstream source is gone
- **WHEN** a raw source of the dictionary no longer answers on release day
- **THEN** the release builds its pack from the pinned snapshot and is not affected

#### Scenario: One dictionary per release
- **WHEN** a version is released to the Chrome Web Store, addons.mozilla.org and the App Store
- **THEN** the three packages carry byte-identical packs

#### Scenario: The reviewer's rebuild
- **WHEN** a reviewer rebuilds the pack from the submitted source archive with no network
- **THEN** they obtain the pack the package carries, byte for byte

#### Scenario: A snapshot that cannot be fetched
- **WHEN** a pull request is checked while the pinned snapshot is missing or does not match its sha256
- **THEN** the check fails, naming the snapshot

## ADDED Requirements

### Requirement: A dictionary refresh is a reviewed decision
The live upstream sources SHALL be read only by a refresh that proposes a new snapshot, and a new snapshot SHALL reach a release only through a reviewed change of the pin.
The refresh SHALL report, against the pinned snapshot, the lemmas, glosses, levels and expressions
added, removed and changed, and the new pack's size against its budget. A change to the reduction
rules SHALL reach a release only through a refresh: a pin whose recorded reduction rules differ from
the repository's SHALL fail the checks.

#### Scenario: Refreshing
- **WHEN** a maintainer runs the refresh
- **THEN** a new snapshot is proposed with a report of what it changes, and releases keep the pinned one until the pin change is merged

#### Scenario: Reduction rules changed without a refresh
- **WHEN** the reducer is edited and the pin still names a snapshot made by the previous reducer
- **THEN** the checks fail, asking for a refresh

### Requirement: A pack says which dictionary it is
A pack's `pack_version` SHALL identify the snapshot it was built from.

#### Scenario: Two releases, one dictionary
- **WHEN** two releases are built from the same pinned snapshot
- **THEN** their packs report the same `pack_version`

#### Scenario: A refreshed dictionary
- **WHEN** a release is built after the pin moved to a new snapshot
- **THEN** its pack reports a different `pack_version`
