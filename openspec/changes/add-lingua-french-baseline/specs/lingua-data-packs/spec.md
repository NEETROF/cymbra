## MODIFIED Requirements

### Requirement: A pack names the language it studies
The core SHALL read the language a pack studies from the pack's metadata (`studied`, an ISO 639-1 code), SHALL refuse to load a pack whose studied language it has no analyser for, and SHALL check the pack's `analyzer_version` against the analyser version of that language, never of another. Adding a studied language to the core SHALL leave every existing pack loadable and byte-identical.

#### Scenario: Loading the EN→FR pack names English
- **WHEN** the en-fr pack is loaded
- **THEN** the core reports its studied language as English and accepts its `analyzer_version` `1.1.0`

#### Scenario: A pack for a language the core cannot analyse
- **WHEN** a pack whose metadata names `pt` is loaded by a core with no Portuguese analyser
- **THEN** loading fails with an explicit error naming the language, and no partial analysis is produced

#### Scenario: Versions are compared within a language
- **WHEN** one Spanish pack declares Spanish's analyser version, and another Spanish pack declares English's `1.1.0`
- **THEN** the first loads and the second is refused as built for another analyser generation

#### Scenario: The en-fr pack does not change
- **WHEN** the en-fr pack is built from its committed tables after the core gains a second language
- **THEN** its sha256 is the one recorded in `pin.json` before the change

#### Scenario: A French pack at French's analyser version
- **WHEN** a pack whose metadata names `fr` declares French's analyser version `0.1.0`, and another names `fr` and declares `1.1.0`
- **THEN** the first loads and reports French as its studied language, and the second is refused as built for another analyser generation

#### Scenario: A French pack glossed in French
- **WHEN** a pack whose metadata studies `fr` and is glossed in `fr` is loaded
- **THEN** it is refused as glossed in the language it studies

#### Scenario: The shipped and committed packs do not change
- **WHEN** every committed pair's pack (en-fr, es-fr, es-en and en-es) is built from its committed tables after the core gains French
- **THEN** each sha256 is the one its `pin.json` records
