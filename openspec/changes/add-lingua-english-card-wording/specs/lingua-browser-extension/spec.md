## ADDED Requirements

### Requirement: The card of English-native readers of Spanish is pinned on the real pack
The word card of English-native readers of Spanish SHALL be bounded by a golden of its own: the engine's answers over the real es-en pack built from the committed tables — glosses, sense runs, expressions and the grammar of the reference golden's Spanish probes and of further lemmas — and the English lines the card renders from them, read from the golden, never from the tables. The golden SHALL be run by the checks that run the other baselines and re-blessed where they are; a change that moves either SHALL fail until it is re-blessed in the pull request that moves it, saying why. A row's cut SHALL not end on an opening mark, a French gloss's row staying byte for byte what it was.

#### Scenario: A probe of the reference golden
- **WHEN** the es-en golden's grammar probe of « vino » read as a form of « venir » is rendered with the interface in English
- **THEN** its grammar line, gloss headings, pages and row are the ones the snapshot pins, and the gloss is the es-en pack's, unaltered

#### Scenario: A cut after an opening mark
- **WHEN** a row's first sense is cut just after an opening quotation mark
- **THEN** the row ends on the word before it, then the ellipsis

#### Scenario: The golden moves
- **WHEN** a re-reduction of es-en or of es-fr, or a renderer change, moves a line the golden or its snapshot pins
- **THEN** the checks fail until both are re-blessed in that pull request

#### Scenario: French readers unchanged
- **WHEN** en-fr's and es-fr's rows and cards are rendered after this change
- **THEN** every line is byte for byte what it was
