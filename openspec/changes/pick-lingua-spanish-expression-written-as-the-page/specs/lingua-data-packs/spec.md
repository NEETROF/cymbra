## ADDED Requirements

### Requirement: A Spanish pack keeps every headword of a key several reach
A pack studying Spanish SHALL keep every headword that reaches an expression key, as *A French pack keeps every headword of a key several reach* requires of a pack studying French. The key SHALL answer the headword that wins it under *A Spanish pack keys its expressions as Spanish is read*, with the gloss and the name that requirement gives it; each other headword reaching the same key SHALL be an expression of its own, with its own gloss and its name — its headword as the dictionary writes it —, filed in the pack's expression glosses and names under an id that no key maps to; and the optional section a French pack lists them in SHALL list, for each key several headwords reach, its other headwords, in the order that requirement ranks them. A core that does not read that section SHALL find every key answered by the headword, gloss and name it answered before. The section SHALL be derived from the committed tables: `pack_version` SHALL NOT move for it, and the pack's new sha256 and size SHALL be recorded in its pin. For a pack studying Spanish, these rules SHALL take the place of the sentence of *A French pack keeps every headword of a key several reach* by which a pack studying Spanish carries no such section, and of the rule of *A Spanish pack keys its expressions as Spanish is read* and of *Multi-word expression table* by which a key several headwords reach keeps one of them, every other rule of those requirements still applying. A pack studying English SHALL carry no such section, byte for byte.

#### Scenario: Two headwords reach one key
- **WHEN** a Spanish pack is built from a source holding `hacer tiempo` and `hace tiempo`, both keyed `hacer tiempo`
- **THEN** the key `hacer tiempo` answers `hacer tiempo`, the headword that wins it, with its gloss, and the pack also holds `hace tiempo` with its own gloss, listed as the key's other headword

#### Scenario: Three headwords reach one key
- **WHEN** a Spanish pack is built from a source holding `echar a perder`, `echado a perder` and `echarse a perder`, all keyed `echar a perder`
- **THEN** the key answers `echar a perder`, and `echado a perder` and `echarse a perder` are listed after it, in that order, each with its own gloss

#### Scenario: A key one headword reaches
- **WHEN** a Spanish pack is built from a source holding `tener en cuenta` alone on its key
- **THEN** the key answers `tener en cuenta` as before, and the section lists nothing for it

#### Scenario: A core that does not read the section
- **WHEN** a Spanish pack carrying the section is read by a core that ignores it
- **THEN** every key answers the headword, gloss and name it answered in the pack built without the section

#### Scenario: The committed Spanish packs
- **WHEN** the es-fr and es-en packs are built from their committed tables
- **THEN** each carries the section, its `pack_version` and `analyzer_version` are the ones it had, and its pin records its new sha256 and size

#### Scenario: English and French packs do not move
- **WHEN** the en-fr, en-es, fr-en and fr-es packs are built from their committed tables
- **THEN** each has the sha256 its pin records, and neither English pack carries the section
