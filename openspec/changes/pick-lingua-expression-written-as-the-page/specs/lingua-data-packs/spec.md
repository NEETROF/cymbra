## ADDED Requirements

### Requirement: A French pack keeps every headword of a key several reach
A pack studying French SHALL keep every headword that reaches an expression key, not only the one that wins it. The key SHALL answer the headword that wins it under *A French pack keys its expressions as French is read*, with the gloss and the name that requirement gives it; each other headword reaching the same key SHALL be an expression of its own, with its own gloss and its name — its headword as the dictionary writes it —, filed in the pack's expression glosses and names under an id that no key maps to; and an optional section SHALL list, for each key several headwords reach, its other headwords, in the order that requirement ranks them. A core that does not read that section SHALL find every key answered by the headword, gloss and name it answered before. The section SHALL be derived from the committed tables: `pack_version` SHALL NOT move for it, and the pack's new sha256 and size SHALL be recorded in its pin. For a pack studying French, these rules SHALL take the place of the rule of *Multi-word expression table* and of *A French pack keys its expressions as French is read* by which a key several entries reach keeps one of them and the others are left out, every other rule of those requirements still applying. A pack studying English or Spanish SHALL carry no such section, byte for byte.

#### Scenario: Two headwords reach one key
- **WHEN** a French pack is built from a source holding `ça va` and `ça ira`, both keyed `ça aller`
- **THEN** the key `ça aller` answers `ça ira`, the headword that wins it, with its gloss, and the pack also holds `ça va` with its own gloss, listed as the key's other headword

#### Scenario: Three headwords reach one key
- **WHEN** a French pack is built from a source holding `allez-y`, `allons-y` and `vas-y`, all keyed `aller y`
- **THEN** the key answers `allez-y`, and `allons-y` and `vas-y` are listed after it, in that order, each with its own gloss

#### Scenario: A key one headword reaches
- **WHEN** a French pack is built from a source holding `au revoir` alone on its key
- **THEN** the key answers `au revoir` as before, and the section lists nothing for it

#### Scenario: A core that does not read the section
- **WHEN** a French pack carrying the section is read by a core that ignores it
- **THEN** every key answers the headword, gloss and name it answered in the pack built without the section

#### Scenario: The committed French packs
- **WHEN** the fr-en and fr-es packs are built from their committed tables
- **THEN** each carries the section, its `pack_version` and `analyzer_version` are the ones it had, and its pin records its new sha256 and size

#### Scenario: English and Spanish packs do not move
- **WHEN** the en-fr, en-es, es-fr and es-en packs are built from their committed tables
- **THEN** each has the sha256 its pin records, and none carries the section
