## ADDED Requirements

### Requirement: The card of English-native readers of Spanish is pinned on the real pack
The word card of English-native readers of Spanish SHALL be bounded by a golden of its own: the engine's answers over the real es-en pack built from the committed tables — glosses, sense runs, expressions and the grammar of the reference golden's Spanish probes — and the English lines the card renders from them. A change that moves either SHALL fail until the golden is re-blessed in the pull request that moves it, saying why. The card's text heuristics SHALL treat a gloss by the rules of the edition that wrote it, a French gloss exactly as before.

#### Scenario: A Spanish form
- **WHEN** an English-native reader opens the card on « vino » in a Spanish page, es-en listed
- **THEN** the grammar line names, in English, the third person singular of the preterite of « venir », with « venir » marked as Spanish, and the gloss reads as the English Wiktionary writes it

#### Scenario: A heading
- **WHEN** a noun of the Spanish probes is opened
- **THEN** its gloss is headed by its part of speech and gender in English

#### Scenario: The golden moves
- **WHEN** a re-reduction of es-en or a renderer change moves a line the golden pins
- **THEN** the tests fail until it is re-blessed in that pull request

#### Scenario: French readers unchanged
- **WHEN** en-fr's and es-fr's rows and cards are rendered after this change
- **THEN** every line is byte for byte what it was
