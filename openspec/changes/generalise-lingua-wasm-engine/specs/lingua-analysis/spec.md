## ADDED Requirements

### Requirement: One WASM engine serves every studied language
The WASM engine SHALL hold at most one pack per studied language in a single instance, and SHALL take the studied language on every call whose answer depends on it; a call that names no language SHALL use the language of the first pack loaded. The engine SHALL refuse, with an explicit error, a call naming a language it holds no pack for, and a second pack for a language it already holds. Calls that concern the whole reader (backup, restore, resets, counts, the review session, sync exports and applies) SHALL cover every language the state holds.

#### Scenario: An engine built from the en-fr pack answers as before
- **WHEN** the engine is built from the en-fr pack and called without a language
- **THEN** every answer is byte-for-byte the one recorded by the English invariance baseline

#### Scenario: A second language is served by its own pack
- **WHEN** a Spanish pack is added to an engine built from the en-fr pack, and a page is analysed as Spanish
- **THEN** the analysis uses the Spanish pack and reports Spanish's analyser version, and an analysis that names no language is still English

#### Scenario: A language without a pack is refused
- **WHEN** a call names Spanish on an engine holding only the en-fr pack
- **THEN** it fails with an explicit error and the state is unchanged

#### Scenario: One pack per language
- **WHEN** a second English pack is added to an engine
- **THEN** it is refused and the engine keeps the first

### Requirement: Sync records carry their own language
The WASM engine SHALL export each status, declared level and card with the studied language it belongs to, and SHALL apply incoming records only for the languages it holds a pack for, leaving the others out of its state. A record without a language SHALL be read as English.

#### Scenario: English records export as English
- **WHEN** an engine holding the en-fr pack exports its statuses, declared levels and cards
- **THEN** every record names `en`, as before

#### Scenario: A Spanish record exports as Spanish
- **WHEN** a Spanish status is set on an engine holding the en-fr pack and a Spanish pack
- **THEN** its exported record names `es`

#### Scenario: Records of a language the engine does not study are skipped
- **WHEN** a Spanish status change is applied to an engine holding only the en-fr pack
- **THEN** nothing changes, as before; applied to an engine holding the Spanish pack, the status is recorded under Spanish
