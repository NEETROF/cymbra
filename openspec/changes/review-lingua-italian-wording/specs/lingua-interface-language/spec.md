## ADDED Requirements

### Requirement: The Italian wording is one vocabulary, reviewed across its surfaces
Cymbra Lingua's Italian SHALL use one term per product concept across the extension's interface, its manifest, the Safari app, the site and the store listings, as a committed glossary records them beside their French source terms. Before Italian is shown to a reader, every Italian surface SHALL have been read together against the French source and the glossary, and the review's findings and their resolutions SHALL be committed. Where the text is keyed messages — the extension's catalogue, `_locales/it` and the Safari app's activation page — a test SHALL fail when an Italian message does not use the glossary's term for a concept its French counterpart names, unless the glossary lists that key as an exception with its reason.

#### Scenario: A drifted term
- **WHEN** the French review panel's name is « panneau de révision » in two keys, and the Italian says « pannello di ripasso » in one and « pannello di revisione » in the other
- **THEN** the glossary test fails, naming the second key and the concept

#### Scenario: An exception with its reason
- **WHEN** an Italian message rightly words a concept otherwise, and the glossary lists its key with the reason
- **THEN** the test passes

#### Scenario: Reviewed before shown
- **WHEN** the release that first shows the Italian interface is prepared
- **THEN** the review's report is committed, every finding resolved or answered with a source
