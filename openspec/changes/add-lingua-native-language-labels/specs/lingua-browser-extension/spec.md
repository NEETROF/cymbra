## ADDED Requirements

### Requirement: Languages are named in the interface language
Every name of a language the interface shows, and every sentence built around one — a level's title, a note on estimated levels, a missing-text notice, a voice's absence, a platform's voice name — SHALL come from the catalogue's entry for the interface language, written in that language's own grammar: its articles, contractions and gendered forms where it has them; the module the surfaces call SHALL take the interface language and SHALL hold no literal. The level scale SHALL be named « CEFR » in French and English and « MCER » in Spanish. A name of a language in any interface language SHALL NOT be written outside the catalogue — the engine's enum names being data, not copy — and a lint SHALL check it on string literals, with the catalogue's baseline for the files not yet moved.

#### Scenario: Every reader today
- **WHEN** a reader of French opens Réglages, the popup, the statistics or the onboarding
- **THEN** every title, note and notice that names a language reads byte for byte as before

#### Scenario: An English-native reader
- **WHEN** the interface language is English and the reader's Spanish level is estimated
- **THEN** the level's title reads "Estimated Spanish level", the note names the CEFR, and the Windows voice is named "English (United States)"

#### Scenario: A Spanish-native reader
- **WHEN** the interface language is Spanish and a page holds no English text
- **THEN** the popup says « No se detectó texto en inglés en esta página », and a level's note names the MCER

#### Scenario: Checked by lint
- **WHEN** a module outside the catalogue writes "Spanish" or « español » as a name
- **THEN** the lint fails, naming the file and the line

## MODIFIED Requirements

### Requirement: Languages are named in one place
Every name of a studied language the interface shows — in a title, a notice, a prompt, a voice's name — SHALL come from one module per interface language, `src/i18n/<language>/languages.ts`, in that language's own grammar, and no surface SHALL hard-code a language's name. A lint SHALL check that no name of a language, in any interface language, is written in a string literal outside those modules, with the catalogue's baseline for the files not yet moved.

#### Scenario: A Spanish page without enough text
- **WHEN** a page in Spanish holds too little text for the engine
- **THEN** the popup says no Spanish text was detected on this page, in the interface language, naming Spanish from the languages' module

#### Scenario: Checked by lint
- **WHEN** a module outside `src/i18n/*/languages.ts` writes « anglais », "Spanish" or « español » as a name
- **THEN** the lint fails, naming the file and the line
