## ADDED Requirements

### Requirement: Réglages speaks the interface language
Réglages — its tabs, its blocks' titles and every text of its blocks (studied languages, levels, the page bar, read-aloud, books, display, colours, translation, shortcuts, account, synchronisation, reset) — SHALL read its copy from the catalogue's modules for the interface language, which its host reads with its preferences and hands it; the blocks' titles SHALL live in the catalogue, where the hosts' lint reads them; a message built from parts SHALL be one message taking its parts; the French SHALL be byte for byte what it was.

#### Scenario: Every reader today
- **WHEN** a reader of French opens Réglages from the popup, the side panel or the drawer
- **THEN** every tab, title, text and confirmation is byte for byte what it was

#### Scenario: A Spanish-native reader
- **WHEN** the interface language is Spanish and the reader opens Réglages
- **THEN** the tabs, titles and blocks are the Spanish catalogue's, and the translation setting's cost is formatted for Spanish

#### Scenario: The titles live once
- **WHEN** a host writes a block's title as a literal
- **THEN** the hosts' lint fails, naming the host

#### Scenario: Off the baseline
- **WHEN** the lint runs after this change
- **THEN** none of Réglages' files is on the baseline, and none holds a French literal
