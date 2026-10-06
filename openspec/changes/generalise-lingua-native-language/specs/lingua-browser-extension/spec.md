## ADDED Requirements

### Requirement: A reader is served the pairs of their native language
The extension SHALL serve a reader only the listed pairs glossed in the reader's native language, read from the profile in their stored backup, and French when the profile names none or names one no listed pair is glossed in. Wherever a requirement speaks of the shipped pairs, the listed pairs or the default pair — the languages offered in Réglages and at onboarding, the language a surface reads in, the languages a device accepts from the sync, the packs the engine loads — it SHALL mean the listed pairs of the reader's native language, the default pair being the first of them. A pair SHALL be chosen by its studied and its native language, never by its studied language alone, and a pack glossed in another native language SHALL never be loaded. An installed extension SHALL keep its reader in French without asking.

#### Scenario: Every reader today
- **WHEN** a reader whose profile names French, or no native language, uses the extension with en-fr and es-fr listed
- **THEN** the engine starts on en-fr, every request names the language it named before, and Réglages offers English and Spanish as before

#### Scenario: A listed pair of another native language
- **WHEN** the list holds en-fr, es-fr and es-en, and a French reader reads a Spanish page
- **THEN** the page is read with the es-fr pack, and the es-en pack is never fetched

#### Scenario: An English speaker's pairs
- **WHEN** the list holds en-fr, es-fr and es-en, and the reader's native language is English
- **THEN** the engine starts on es-en, Spanish is the only language offered, and a request in English is refused before it reaches the engine

## MODIFIED Requirements

### Requirement: An estimated ladder shows English's typical vocabularies
When the ladder shows a language whose pack's levels are estimated, each level's typical vocabulary SHALL be the one English's CEFR lists give that level as the core records it, whatever packs the engine holds — 0 at A1, 1,292 at A2, 3,359 at B1, 7,988 at B2, 16,326 at C1 and 20,556 at C2 — and the ladder SHALL say that the figures are English's. A ladder of CEFR-list levels SHALL show its own figures, as before.

#### Scenario: The Spanish ladder
- **WHEN** the statistics open for Spanish
- **THEN** each level's « estimés » equals English's at that level, and the legend says they are taken from English

#### Scenario: No English pack
- **WHEN** the statistics open for Spanish on an engine that holds no English pack
- **THEN** the figures are the same, still taken from English

#### Scenario: An English dictionary update
- **WHEN** the en-fr pack is rebuilt from new tables
- **THEN** the Spanish ladder's figures do not change

#### Scenario: The English ladder
- **WHEN** the statistics open for English
- **THEN** the ladder shows the figures its own lists give, with its legend as before
