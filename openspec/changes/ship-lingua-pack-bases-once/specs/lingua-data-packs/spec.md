# Spec Delta

## ADDED Requirements

### Requirement: A package carries each base once and each listed pair's layer
A package SHALL carry, for the pairs its list names, one base per studied language and one layer per pair, each built from the committed tables and checked against its pin: the layer at the pair's path, `assets/packs/<pair>.lingua`, where a pair's whole pack lived, so that what reads the shipped natives from the package's pair names reads them unchanged; the base at `assets/bases/<studied>.lingua`. The build SHALL fail when the list names two pairs of one studied language whose bases differ, naming both pairs, and when a packaged layer's `base_version` is not its base's, naming the pair. The package SHALL carry no whole pack, and SHALL expose each part to the contexts that fetch it as it exposed the packs. The source archive a store reviewer rebuilds from SHALL name each packaged base's and layer's recorded sha256 against its path.

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

#### Scenario: The reviewer's archive names the parts
- **WHEN** the source archive is assembled for a store reviewer
- **THEN** its README lists, for each listed pair, the recorded sha256 of its layer at `assets/packs/<pair>.lingua` and of its base at `assets/bases/<studied>.lingua`, and the reviewer's rebuild matches them

## MODIFIED Requirements

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair glossed in a native language gives that native language's default studied language, and SHALL build each listed pair's layer, and its studied language's base, from the pair's committed tables, each checked against the pair's own recorded sha256. It SHALL refuse a base whose analyser version is not the version of its studied language, and a layer whose `base_version` is not its base's. Every package of a release SHALL carry exactly the listed pairs' layers and their studied languages' bases, one of each, byte-identical across Chromium, Firefox and Safari. The list SHALL hold en-fr, es-fr, es-en, en-es then fr-en, followed by fr-es once fr-es's tables are committed at or above its coverage floor, and a package whose list is anything else SHALL be refused. A listed pair held to a coverage floor SHALL measure at or above it on its committed tables.

#### Scenario: Building the listed packs
- **WHEN** a release builds its parts with the list holding en-fr, es-fr, es-en, en-es, fr-en then fr-es
- **THEN** it builds each pair's layer and each language's base from their own tables, each sha256 is the one that pair's pin records, and the package carries those layers and bases and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pair's base declares an analyser version other than the core's version for its studied language
- **THEN** the build fails with a message naming the base, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list other than en-fr, es-fr, es-en, en-es then fr-en, with fr-es after them or not as its floor allows
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack, its layer and the English base are built from their tables after this change
- **THEN** each is byte for byte what was built before, and English stays a French-native reader's default studied language

#### Scenario: English speakers learning Spanish
- **WHEN** a reader whose browser is in English installs a package built with this list
- **THEN** the native-language choice is offered, preset to English, and the reader studies Spanish through es-en

#### Scenario: Spanish speakers learning English
- **WHEN** a reader whose browser is in Spanish installs a package built with this list
- **THEN** the native-language choice is offered, preset to Spanish, and the reader studies English through en-es

#### Scenario: English speakers learning French
- **WHEN** a reader whose native language is English uses a package built with this list
- **THEN** Spanish stays their default studied language, French is offered beside it, and a French page is read through fr-en

#### Scenario: Spanish speakers learning French
- **WHEN** fr-es is listed and a reader whose native language is Spanish uses a package built with this list
- **THEN** English stays their default studied language, French is offered beside it, and a French page is read through fr-es

#### Scenario: French for English speakers alone
- **WHEN** fr-es's first committed measurement fell under its floor, so that no fr-es table is committed
- **THEN** the list ends with fr-en, no package carries fr-es, and a reader whose native language is Spanish is offered English alone

#### Scenario: No French for French speakers
- **WHEN** a reader whose native language is French uses a package built with this list
- **THEN** their pairs are en-fr and es-fr as before, French is never offered, and neither French layer nor the French base is fetched

#### Scenario: A listed pair under its floor
- **WHEN** the list names a pair held to a coverage floor whose committed tables measure under it at one of the three tops
- **THEN** the coverage tests fail, naming the pair, the top and the figure

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
