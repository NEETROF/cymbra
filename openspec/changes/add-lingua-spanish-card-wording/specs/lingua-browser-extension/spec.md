## ADDED Requirements

### Requirement: The card of Spanish-native readers of English is pinned on the real pack
The word card of Spanish-native readers of English SHALL be bounded by a golden of its own: the engine's answers over the real en-es pack built from the committed tables — glosses, sense runs, expressions and the grammar of the reference golden's English probes — and the Spanish lines the card renders from them. A change that moves either SHALL fail until the golden is re-blessed in the pull request that moves it, saying why. The card's text heuristics SHALL treat a gloss by the rules of the edition that wrote it, a French gloss exactly as before.

#### Scenario: A English form
- **WHEN** a Spanish-native reader opens the card on "went" in an English page, en-es listed
- **THEN** the grammar line names, in Spanish, the past simple of "go" in RAE/ASALE terms, with "go" marked as English, and the gloss reads as the Spanish edition and the translation tables give it

#### Scenario: A heading
- **WHEN** an English verb of the probes with an -ing form is opened
- **THEN** the line names the -ing form as M10 fixes it

#### Scenario: The golden moves
- **WHEN** a re-reduction of en-es or a renderer change moves a line the golden pins
- **THEN** the tests fail until it is re-blessed in that pull request

#### Scenario: French readers unchanged
- **WHEN** en-fr's and es-fr's rows and cards are rendered after this change
- **THEN** every line is byte for byte what it was
