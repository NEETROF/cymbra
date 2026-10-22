# Spec Delta

## ADDED Requirements

### Requirement: Italian is studied by readers of English
The extension SHALL offer Italian, in Settings and at onboarding, to a reader whose native language is English once a listed pair studies Italian glossed in English, after Spanish and French; an Italian document SHALL be read, highlighted, glossed, recorded and read aloud for them through it-en; and every surface that names a studied language, asks or shows its level, or says that its levels are estimated SHALL say so of Italian in the English interface. An Italian word on the card SHALL be marked as Italian inside the English lines, and the card SHALL name Italian's forms in English.

#### Scenario: An English speaker adds Italian
- **WHEN** it-en is listed and a reader whose native language is English, studying Spanish, ticks Italian in Settings
- **THEN** the profile holds Spanish then Italian, an Italian page is highlighted through it-en, and the Italian level block is titled "Estimated Italian level" in the English interface

#### Scenario: A stored profile naming Italian, English native
- **WHEN** the stored backup's profile studies Spanish then Italian with English as the native language
- **THEN** every surface that reads the profile without an engine reads Spanish then Italian, and the backup is written in schema version 4

#### Scenario: An Italian word on an English card
- **WHEN** a reader whose native language is English opens the card of an Italian verb in the passato remoto
- **THEN** the card's lines are English, its tense is named "past historic (passato remoto)", and the Italian words in it are marked as Italian
