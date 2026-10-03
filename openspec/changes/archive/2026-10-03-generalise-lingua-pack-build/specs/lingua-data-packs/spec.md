## ADDED Requirements

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair gives the default studied language, and SHALL build each listed pair's pack from that pair's committed tables, checked against that pair's own recorded sha256. It SHALL refuse a pack whose analyser version is not the version of the pack's own studied language. Every package of a release SHALL carry exactly the listed packs, each byte-identical across Chromium, Firefox and Safari. Until Spanish is enabled for readers, a package whose list is anything other than en-fr SHALL be refused.

#### Scenario: Building the listed packs
- **WHEN** a release builds its packs with the list holding en-fr
- **THEN** it builds the en-fr pack from the en-fr tables, its sha256 is the one en-fr's pin records, and the package carries that pack and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pack's analyser version differs from the core's version for that pack's studied language
- **THEN** the build fails with a message naming the pack, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list holding a pair other than en-fr, before Spanish is enabled for readers
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack is built from its tables after this change
- **THEN** it is byte-for-byte the pack built before, and only its path inside the package differs
