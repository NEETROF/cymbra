# Spec Delta

## ADDED Requirements

### Requirement: Italian is studied by readers of Spanish
The extension SHALL offer Italian, in its settings (« Ajustes » in the Spanish interface) and at onboarding, to a reader whose native language is Spanish once a listed pair studies Italian glossed in Spanish, after English and French; an Italian document SHALL be read, highlighted, glossed, recorded and read aloud for them through it-es; and every surface that names a studied language, asks or shows its level, or says that its levels are estimated SHALL say so of Italian in the Spanish interface. An Italian word on the card SHALL be marked as Italian inside the Spanish lines, and the card SHALL name Italian's forms in Spanish.

#### Scenario: A Spanish speaker adds Italian
- **WHEN** it-es is listed and a reader whose native language is Spanish, studying English, ticks Italian in « Ajustes »
- **THEN** the profile holds English then Italian, an Italian page is highlighted through it-es, and the Italian level block is titled « Nivel de italiano estimado » in the Spanish interface

#### Scenario: A stored profile naming Italian, Spanish native
- **WHEN** the stored backup's profile studies English then Italian with Spanish as the native language
- **THEN** every surface that reads the profile without an engine reads English then Italian, and the backup is written in schema version 4

#### Scenario: An Italian word on a Spanish card
- **WHEN** a reader whose native language is Spanish opens the card of an Italian verb in the passato remoto
- **THEN** the card's lines are Spanish, its tense is named « pretérito perfecto simple de indicativo », and the Italian words in it are marked as Italian
