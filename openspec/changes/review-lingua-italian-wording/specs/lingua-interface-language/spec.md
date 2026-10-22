## ADDED Requirements

### Requirement: The Italian wording is one vocabulary across its surfaces
Cymbra Lingua's Italian SHALL use one term per product concept across the extension's interface, its manifest, the Safari app, the site and the store listings, as a committed glossary records them: for each concept, its French forms, the accepted Italian forms and the rejected variants. A test SHALL read the extension's catalogue, `_locales/it` and the Safari app's activation page, comparing each message with its French counterpart under the same key, and SHALL fail, naming the key and the concept, when the French uses a concept the Italian does not name with an accepted form, or when an Italian message uses a rejected variant. Messages written per audience rather than translated SHALL be left out of the comparison, and an exception SHALL carry its reason and fail once it no longer applies.

#### Scenario: A drifted term
- **WHEN** the French review panel's name is « panneau de révision » in two keys, and the Italian says « pannello di ripasso » in one and « pannello di revisione » in the other
- **THEN** the glossary test fails, naming the second key and the concept

#### Scenario: A rejected variant with no French anchor
- **WHEN** an Italian message uses a variant the glossary rejects, though its French names no concept
- **THEN** the glossary test fails, naming the key and the variant

#### Scenario: A word inside a longer word
- **WHEN** a French message holds « cartes » or « décompte » and the glossary's French form is « carte » or « compte »
- **THEN** no match is read, the forms being matched as whole words by their letters

#### Scenario: An exception that no longer applies
- **WHEN** the glossary lists a key as an exception and the key's French no longer names the concept
- **THEN** the glossary test fails, naming the stale exception
