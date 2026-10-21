# Spec Delta

## MODIFIED Requirements

### Requirement: A language's pack is loaded the first time it is needed
The extension's engine SHALL start with the default pair's pack only — its language's base and its layer, assembled — and SHALL load another listed pair's pack, its base and its layer, the first time a request in that pair's studied language is made, or the first time synced records in that language are applied, once per engine; a base SHALL be fetched once per engine, since an engine holds one pair per studied language. A request in a language no listed pair studies SHALL be refused with an explicit error before it reaches the engine. Requests about the whole reader SHALL load no pack.

#### Scenario: The default pack only, at start
- **WHEN** the engine starts with the shipped list holding en-fr
- **THEN** it loads the English base and the en-fr layer and fetches no other file

#### Scenario: A second listed language, loaded once on first use
- **WHEN** the list holds en-fr and es-fr, and two pages are analysed in Spanish one after the other
- **THEN** the Spanish base and the es-fr layer are fetched and loaded once, before the first analysis, and both analyses are answered in Spanish

#### Scenario: Synced records bring their pack
- **WHEN** status changes in Spanish are applied to an engine that has not loaded the es-fr pack, with es-fr listed
- **THEN** the Spanish base and the es-fr layer are loaded first and the changes are recorded under Spanish

#### Scenario: A language nothing ships
- **WHEN** a request names Spanish and the shipped list holds only en-fr
- **THEN** it fails with an error naming the language and the shipped pairs, and the engine receives nothing

#### Scenario: A failed load is not remembered
- **WHEN** fetching the Spanish base or the es-fr layer fails on a first request in Spanish, with es-fr listed
- **THEN** that request fails, and the next request in Spanish fetches the parts again

#### Scenario: Whole-reader requests load nothing
- **WHEN** the extension takes a backup or counts the cards due
- **THEN** no pack beyond those already loaded is fetched
