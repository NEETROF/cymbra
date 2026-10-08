## ADDED Requirements

### Requirement: The card of Spanish-native readers of English is pinned on the real pack
The word card of Spanish-native readers of English SHALL be bounded by a golden of its own: the engine's answers over the real en-es pack built from the committed tables — glosses, sense runs, expressions and the grammar of the reference golden's English probes and of further lemmas — and the Spanish lines the card renders from them, read from the golden, never from the tables. The golden SHALL be run by the checks that run the other baselines and re-blessed where they are; a change that moves either SHALL fail until it is re-blessed in the pull request that moves it, saying why.

#### Scenario: A probe of the reference golden
- **WHEN** the en-es golden's grammar probe of "went" read as a form of "go" is rendered with the interface in Spanish
- **THEN** its line names the simple past by the name Spanish-language teaching of English gives it, never a Spanish tense's name, and its gloss headings, pages and row are the ones the snapshot pins

#### Scenario: The -ing form
- **WHEN** the en-es golden's probe of "running" read as a form of "run" is rendered with the interface in Spanish
- **THEN** the line names it as the -ing form, as Spanish-language teaching of English does, never as a Spanish gerundio

#### Scenario: The golden moves
- **WHEN** a re-reduction of en-es or of en-fr, or a renderer change, moves a line the golden or its snapshot pins
- **THEN** the checks fail until both are re-blessed in that pull request

#### Scenario: French readers unchanged
- **WHEN** en-fr's and es-fr's rows and cards are rendered after this change
- **THEN** every line is byte for byte what it was
