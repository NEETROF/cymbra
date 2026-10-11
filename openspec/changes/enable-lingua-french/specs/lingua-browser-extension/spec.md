## ADDED Requirements

### Requirement: French is studied by readers of English and Spanish
The extension SHALL know French as a studied language, as it knows English and Spanish: a stored profile naming French SHALL be read as studying French; French SHALL be offered, in Réglages and at onboarding, to a reader whose native language has a listed pair studying French, after that native language's default, and SHALL never be offered to a reader whose native language is French; a French document SHALL be read, highlighted, glossed, recorded and read aloud through the reader's pair of French; and every surface that names a studied language, asks or shows its level, or says that its levels are estimated SHALL say so of French in the interface language, as it does of Spanish. A French word on the card SHALL be marked as French inside the interface language's lines.

#### Scenario: An English speaker adds French
- **WHEN** fr-en is listed and a reader whose native language is English, studying Spanish, ticks French in Réglages
- **THEN** the profile holds Spanish then French, a French page is highlighted through fr-en, and the French level block is titled "Estimated French level" in the English interface

#### Scenario: A Spanish speaker adds French
- **WHEN** fr-es is listed and a reader whose native language is Spanish, studying English, ticks French in Réglages
- **THEN** the profile holds English then French, a French page is highlighted through fr-es, and the French level block is titled « Nivel de francés estimado » in the Spanish interface

#### Scenario: A Spanish speaker without fr-es
- **WHEN** fr-en is listed and fr-es is not, and a reader whose native language is Spanish opens Réglages
- **THEN** French is not offered, and the choice of studied languages stays hidden as before

#### Scenario: A French speaker
- **WHEN** a reader whose native language is French opens Réglages with fr-en and fr-es listed
- **THEN** English and Spanish are offered as before, French is not, and a French page is not read

#### Scenario: A stored profile naming French
- **WHEN** the stored backup's profile studies Spanish then French with English as the native language
- **THEN** every surface that reads the profile without an engine reads Spanish then French, and the backup is written in schema version 3

#### Scenario: A French word on the card
- **WHEN** a reader whose native language is English opens the card of a French word
- **THEN** the card's lines are English, and the French words in them are marked as French for the voices, the hyphenation and the spell-check
