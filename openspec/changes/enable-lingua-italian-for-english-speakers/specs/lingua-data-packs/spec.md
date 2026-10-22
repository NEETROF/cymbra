# Spec Delta

## MODIFIED Requirements

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair glossed in a native language gives that native language's default studied language, and SHALL build each listed pair's layer, and its studied language's base, from the pair's committed tables, each checked against the pair's own recorded sha256. It SHALL refuse a base whose analyser version is not the version of its studied language, and a layer whose `base_version` is not its base's. Every package of a release SHALL carry exactly the listed pairs' layers and their studied languages' bases, one of each, byte-identical across Chromium, Firefox and Safari. The list SHALL hold en-fr, es-fr, es-en, en-es then fr-en, followed by fr-es once fr-es's tables are committed at or above its coverage floor, then it-fr, then it-en, and a package whose list is anything else SHALL be refused. A listed pair held to a coverage floor SHALL measure at or above it on its committed tables.

#### Scenario: Building the listed packs
- **WHEN** a release builds its parts with the list holding en-fr, es-fr, es-en, en-es, fr-en, fr-es, it-fr then it-en
- **THEN** it builds each pair's layer and each language's base — the Italian one among them — from their own tables, each sha256 is the one that pair's pin records, and the package carries those layers and bases and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pair's base declares an analyser version other than the core's version for its studied language
- **THEN** the build fails with a message naming the base, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list other than en-fr, es-fr, es-en, en-es then fr-en, with fr-es after them or not as its floor allows, then it-fr and it-en
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack, its layer and the English base are built from their tables after this change
- **THEN** the pack is byte for byte what was built before, its layer and the English base are what change 3's checks build from the same tables, and English stays a French-native reader's default studied language

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
- **THEN** fr-en is followed by it-fr and it-en, no package carries fr-es, and a reader whose native language is Spanish is offered English alone

#### Scenario: No French for French speakers
- **WHEN** a reader whose native language is French uses a package built with this list
- **THEN** their pairs are en-fr, es-fr and it-fr, French is never offered, and neither French layer nor the French base is fetched

#### Scenario: A listed pair under its floor
- **WHEN** the list names a pair held to a coverage floor whose committed tables measure under it at one of the three tops
- **THEN** the coverage tests fail, naming the pair, the top and the figure

#### Scenario: French speakers learning Italian
- **WHEN** a reader whose native language is French uses a package built with this list
- **THEN** English stays their default studied language, Italian is offered after Spanish, and an Italian page is read through it-fr, the Italian base and it-fr's layer fetched on first need

#### Scenario: No Italian for English and Spanish speakers
- **WHEN** it-fr is the one listed pair studying Italian, and a reader whose native language is English or Spanish uses a package built with it
- **THEN** Italian is never offered, and neither it-fr's layer nor the Italian base is fetched

#### Scenario: English speakers learning Italian
- **WHEN** a reader whose native language is English uses a package built with this list
- **THEN** Spanish stays their default studied language, Italian is offered after French, and an Italian page is read through it-en, the Italian base and it-en's layer fetched on first need

#### Scenario: No Italian for Spanish speakers
- **WHEN** a reader whose native language is Spanish uses a package built with this list
- **THEN** Italian is never offered, and neither Italian layer nor the Italian base is fetched
