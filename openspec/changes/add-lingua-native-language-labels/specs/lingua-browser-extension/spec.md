## ADDED Requirements

### Requirement: Languages are named in the interface language
Every name of a language the interface shows, and every sentence built around one — a level's title, a note on estimated levels, a missing-text notice, a voice's absence — SHALL come from the catalogue's entry for the interface language, written in that language's own grammar: its articles, contractions and gendered forms where it has them. The level scale SHALL be named « CEFR » in French and English and « MCER » in Spanish. A name of a language in any interface language SHALL NOT be written outside the catalogue, and a lint SHALL check it, with the catalogue's baseline for the files not yet moved.

#### Scenario: Every reader today
- **WHEN** a reader of French opens Réglages, the popup, the statistics or the onboarding
- **THEN** every title, note and notice that names a language reads byte for byte as before

#### Scenario: An English-native reader
- **WHEN** the interface language is English and the reader's Spanish level is estimated
- **THEN** the level's title reads "Estimated Spanish level", and the note names the CEFR

#### Scenario: A Spanish-native reader
- **WHEN** the interface language is Spanish and a page holds no English text
- **THEN** the popup says « No se detectó texto en inglés en esta página », and a level's note names the MCER

#### Scenario: Checked by lint
- **WHEN** a module outside the catalogue writes "Spanish" or « español » as a name
- **THEN** the lint fails, naming the file and the line
