# Spec Delta

## ADDED Requirements

### Requirement: A package carries each base once and each listed pair's layer
A package SHALL carry, for the pairs its list names, one base per studied language and one layer per pair, each built from the committed tables and checked against its pin: the layer at the pair's path, `assets/packs/<pair>.lingua`, where a pair's whole pack lived, so that what reads the shipped natives from the package's pair names reads them unchanged; the base at `assets/bases/<studied>.lingua`. The build SHALL fail when the list names two pairs of one studied language whose bases differ, naming both pairs, and when a packaged layer's `base_version` is not its base's, naming the pair. The package SHALL carry no whole pack, and SHALL expose each part to the contexts that fetch it as it exposed the packs.

#### Scenario: The six pairs' parts
- **WHEN** a package is built with the list holding en-fr, es-fr, es-en, en-es, fr-en and fr-es
- **THEN** it carries the layers of the six pairs under `assets/packs/` and the English, Spanish and French bases under `assets/bases/`, nine files, and no whole pack

#### Scenario: The host app's natives
- **WHEN** the Safari package is built and its app's localisations are declared from the pair names under `assets/packs/`
- **THEN** the natives declared are those of the listed pairs, as before the parts shipped

#### Scenario: Two pairs, two bases
- **WHEN** a package is built while es-en's base, as its tables build it, differs from es-fr's
- **THEN** the build fails naming both pairs, and no package is written

#### Scenario: A layer of another base
- **WHEN** a package is built with a layer whose `base_version` is not that of the base packaged for its language
- **THEN** the build fails naming the pair, and no package is written

## MODIFIED Requirements

### Requirement: Reproducible offline build
Packs SHALL be built by a scripted, reproducible pipeline from reduced tables committed in the repository, and a release SHALL build its pack only from those tables, never from live upstream sources.
The raw data and the built pack SHALL NOT be committed; the reduced tables SHALL be, under their own
licences, with the sha256 of the pack they build and, since the parts ship, of the base and the
layer they build. The raw sources the tables were reduced from SHALL be pinned — at a commit and by
sha256 where they are hosted, or kept as a snapshot where they are not stable — so the tables can be
reduced again from the same bytes. Two runs of the pipeline over the same tables SHALL produce
identical packs and identical parts, and every package of a release — Chromium, Firefox and Safari —
SHALL carry those same parts. A build whose pack, base or layer does not match the recorded sha256
SHALL fail, whether in a release or in a pull request's checks.

#### Scenario: Rebuilding the pack
- **WHEN** the pipeline is re-run over the same tables
- **THEN** the resulting pack, base and layer are byte-for-byte identical to the previous ones

#### Scenario: An upstream source is gone
- **WHEN** a raw source of the dictionary no longer answers on release day
- **THEN** the release builds its parts from the committed tables and is not affected

#### Scenario: One dictionary per release
- **WHEN** a version is released to the Chrome Web Store, addons.mozilla.org and the App Store
- **THEN** the three packages carry byte-identical bases and layers

#### Scenario: The reviewer's rebuild
- **WHEN** a reviewer rebuilds the parts from the submitted source archive with no network
- **THEN** they obtain the bases and layers the package carries, byte for byte

#### Scenario: A pull request that breaks the pack
- **WHEN** a pull request changes the builder, the tables or a dependency so that the pack's, the base's or the layer's sha256 or its size budget no longer holds
- **THEN** its checks fail, before any release

#### Scenario: A reader pair's base
- **WHEN** a reader pair's pin records a base other than the one its language's reference pair records
- **THEN** the checks fail, naming the pair, its reference and the base
