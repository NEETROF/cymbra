## ADDED Requirements

### Requirement: The reading surfaces speak the interface language
The popup and its page, the HUD, the injected drawer, the word card, the selection card, the side panel's page and the reader with its library SHALL read every text they show from the catalogue's modules for the interface language, read before the surface renders its copy; their pages and injected hosts SHALL carry the interface language as `lang`; a number they show SHALL be formatted for the interface language, the French exactly as before. For a reader of French every text SHALL be byte for byte what it was.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the popup, the side panel, the reader, a page's HUD and a word card
- **THEN** every text is byte for byte what it was, and each surface carries `lang="fr"`

#### Scenario: An English-native reader
- **WHEN** the interface language is English and the reader opens the word card on a Spanish page
- **THEN** the card's labels, actions and listen buttons are the English catalogue's, and the card carries `lang="en"`

#### Scenario: A page before its script
- **WHEN** the popup opens
- **THEN** no French static text shows before the catalogue fills the page, and the page's `lang` is the interface language once filled

#### Scenario: Off the baseline
- **WHEN** the lint runs after this change
- **THEN** none of these surfaces' files is on the baseline, and none holds a French literal
