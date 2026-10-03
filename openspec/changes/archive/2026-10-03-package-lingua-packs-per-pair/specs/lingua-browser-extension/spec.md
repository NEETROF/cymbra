## ADDED Requirements

### Requirement: A language's pack is loaded the first time it is needed
The extension's engine SHALL start with the default pair's pack only, and SHALL load another listed pair's pack the first time a request in that pair's studied language is made, or the first time synced records in that language are applied, once per engine. A request in a language no listed pair studies SHALL be refused with an explicit error before it reaches the engine. Requests about the whole reader SHALL load no pack.

#### Scenario: The default pack only, at start
- **WHEN** the engine starts with the shipped list holding en-fr
- **THEN** it loads the en-fr pack and fetches no other

#### Scenario: A second listed language, loaded once on first use
- **WHEN** the list holds en-fr and es-fr, and two pages are analysed in Spanish one after the other
- **THEN** the es-fr pack is fetched and loaded once, before the first analysis, and both analyses are answered in Spanish

#### Scenario: Synced records bring their pack
- **WHEN** status changes in Spanish are applied to an engine that has not loaded the es-fr pack, with es-fr listed
- **THEN** the es-fr pack is loaded first and the changes are recorded under Spanish

#### Scenario: A language nothing ships
- **WHEN** a request names Spanish and the shipped list holds only en-fr
- **THEN** it fails with an error naming the language and the shipped pairs, and the engine receives nothing

#### Scenario: A failed load is not remembered
- **WHEN** fetching the es-fr pack fails on a first request in Spanish, with es-fr listed
- **THEN** that request fails, and the next request in Spanish fetches the pack again

#### Scenario: Whole-reader requests load nothing
- **WHEN** the extension takes a backup or counts the cards due
- **THEN** no pack beyond those already loaded is fetched
