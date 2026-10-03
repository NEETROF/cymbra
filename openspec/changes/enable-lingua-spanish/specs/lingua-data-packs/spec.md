## MODIFIED Requirements

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair gives the default studied language, and SHALL build each listed pair's pack from that pair's committed tables, checked against that pair's own recorded sha256. It SHALL refuse a pack whose analyser version is not the version of the pack's own studied language. Every package of a release SHALL carry exactly the listed packs, each byte-identical across Chromium, Firefox and Safari. The list SHALL hold en-fr then es-fr, and a package whose list is anything else SHALL be refused.

#### Scenario: Building the listed packs
- **WHEN** a release builds its packs with the list holding en-fr and es-fr
- **THEN** it builds each pack from its own tables, each sha256 is the one that pair's pin records, and the package carries those two packs and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pack's analyser version differs from the core's version for that pack's studied language
- **THEN** the build fails with a message naming the pack, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list other than en-fr then es-fr
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack is built from its tables after this change
- **THEN** it is byte-for-byte the pack built before, and English stays the default studied language
