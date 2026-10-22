# Spec Delta

## ADDED Requirements

### Requirement: Italian is studied by readers of French
The extension SHALL know Italian as a studied language, as it knows English, Spanish and French: a stored profile naming Italian SHALL be read as studying Italian; Italian SHALL be offered, in Réglages and at onboarding, to a reader whose native language has a listed pair studying Italian, after that native language's other studied languages, and to no other reader; an Italian document SHALL be read, highlighted, glossed, recorded and read aloud through the reader's pair of Italian; and every surface that names a studied language, asks or shows its level, or says that its levels are estimated SHALL say so of Italian in the interface language, as it does of Spanish and French. An Italian word on the card SHALL be marked as Italian inside the interface language's lines.

#### Scenario: A French speaker adds Italian
- **WHEN** it-fr is listed and a reader whose native language is French, studying English, ticks Italian in Réglages
- **THEN** the profile holds English then Italian, an Italian page is highlighted through it-fr, and the Italian level block is titled « Niveau d'italien estimé » in the French interface

#### Scenario: Readers with no pair of Italian
- **WHEN** it-fr is the one listed pair studying Italian, and a reader whose native language is English or Spanish opens Réglages or the onboarding
- **THEN** Italian is not offered, and an Italian page is not read

#### Scenario: A stored profile naming Italian
- **WHEN** the stored backup's profile studies English then Italian with French as the native language
- **THEN** every surface that reads the profile without an engine reads English then Italian, and the backup is written in schema version 4

#### Scenario: An Italian word on the card
- **WHEN** a reader whose native language is French opens the card of an Italian word
- **THEN** the card's lines are French, and the Italian words in them are marked as Italian for the voices, the hyphenation and the spell-check

## MODIFIED Requirements

### Requirement: No word is read in a language whose guard keeps the document's language out
A word card SHALL NOT be read in Spanish when the document declares Catalan, Galician or Occitan, SHALL NOT be read in French when the document declares Catalan, Occitan or Romanian, and SHALL NOT be read in Italian when the document declares one of the neighbours Italian's guard keeps — Catalan, Galician and the regional languages of Italy and Corsica it names —, the languages the detection guards keep out of those languages, in their two- or three-letter codes. Such a card SHALL be the one it was before the requirement *A word the document's language does not know is read in another studied language*.

#### Scenario: A Catalan page
- **WHEN** a reader who studies Spanish and French selects `arribar` on a page that declares `ca`
- **THEN** the card is not read in Spanish, and is the one it was before

#### Scenario: A Galician page declared with three letters
- **WHEN** a reader who studies English and Spanish selects a word on a page that declares `glg`
- **THEN** the card is not read in Spanish

#### Scenario: A French page
- **WHEN** a reader whose native language is English, who studies Spanish and French, opens the card of `hablaba` on a page that declares `fr`
- **THEN** the card is read in Spanish

#### Scenario: A Sicilian page
- **WHEN** it-fr is listed and a reader whose native language is French, who studies Spanish and Italian, selects a word on a page that declares `scn`
- **THEN** the card is not read in Italian
