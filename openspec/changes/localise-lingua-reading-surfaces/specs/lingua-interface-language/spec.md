## ADDED Requirements

### Requirement: The reading surfaces speak the interface language
The popup and its page, the HUD, the injected drawer, the word card, the selection card, the side panel's page and the reader with its library SHALL read every text they show from the catalogue's modules for the interface language, read before the surface is built, and handed to it; their pages and injected hosts SHALL carry the interface language as `lang`; a number or a percentage they show SHALL be formatted for the interface language, the French exactly as each surface writes it today. For a reader of French every text SHALL be byte for byte what it was, and a test SHALL assert it for every one of these surfaces.

#### Scenario: Every reader today
- **WHEN** a reader of French opens the popup, the side panel, the reader, a page's HUD and a word card
- **THEN** every text is byte for byte what it was, and each surface carries `lang="fr"`

#### Scenario: An English-native reader
- **WHEN** the interface language is English and the reader opens the word card on a Spanish page
- **THEN** the card's labels, actions and listen buttons are the English catalogue's, and the card carries `lang="en"`

#### Scenario: A page before its script
- **WHEN** the popup opens
- **THEN** nothing of the page shows before the catalogue fills it, and the page's `lang` is the interface language once filled

#### Scenario: A surface without a spec today
- **WHEN** the popup, the drawer or the side panel is rendered with the French catalogue in a test
- **THEN** the text it shows is the text the HTML page or the module held before this change

#### Scenario: Off the baseline
- **WHEN** the lint runs after this change
- **THEN** none of these surfaces' files is on the baseline, and none holds a French literal
