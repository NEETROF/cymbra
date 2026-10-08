## MODIFIED Requirements

### Requirement: The Lingua page names its languages and publishes their coverage
The site's Lingua page SHALL be built from the pairs Cymbra Lingua ships, in each site language a shipped pair is glossed in and in French and English: for each pair it SHALL name the studied language and the native language, publish the share of the 5,000, 10,000 and 20,000 commonest words of the studied language that have a gloss in the native language in the shipped dictionary, and say whether extended translation serves it, directly or through English. Every pair SHALL be measured the same way, from the committed tables. The page in a language SHALL present first the pairs glossed in that language. The page SHALL say that a gloss is never a machine translation.

#### Scenario: Two languages
- **WHEN** Cymbra Lingua ships English and Spanish for French speakers
- **THEN** the French and English pages name English and Spanish, and show both pairs' figures side by side

#### Scenario: The figures follow the tables
- **WHEN** a pair's committed tables change and the published figures no longer match them
- **THEN** the coverage check fails until the figures are measured again

#### Scenario: Translation for English only
- **WHEN** extended translation serves English and not yet Spanish
- **THEN** the page says so, and that Spanish follows

#### Scenario: English speakers learning Spanish
- **WHEN** es-en ships beside en-fr and es-fr
- **THEN** the English page presents Spanish for English speakers first, with its figures, then the pairs for French speakers

#### Scenario: Before any pair glossed in Spanish
- **WHEN** every shipped pair is glossed in French or English
- **THEN** no Spanish Lingua page is built
