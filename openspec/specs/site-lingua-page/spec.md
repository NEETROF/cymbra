# site-lingua-page Specification

## Purpose
TBD - created by archiving change add-site-lingua-spanish-pages. Update Purpose after archive.
## Requirements
### Requirement: The Lingua page names its languages and publishes their coverage
The site's Lingua page, in French and in English, SHALL name every language Cymbra Lingua teaches. For each language, it SHALL publish the share of the 5,000, 10,000 and 20,000 commonest words that have a French gloss in the shipped dictionary. Every language SHALL be measured the same way, from the committed tables. The page SHALL say which languages extended translation serves, and that a gloss is never a machine translation.

#### Scenario: Two languages
- **WHEN** Cymbra Lingua ships English and Spanish
- **THEN** both pages name English and Spanish, and show both languages' figures side by side

#### Scenario: The figures follow the tables
- **WHEN** a pair's committed tables change and the published figures no longer match them
- **THEN** the coverage check fails until the figures are measured again

#### Scenario: Translation for English only
- **WHEN** extended translation serves English and not yet Spanish
- **THEN** the page says so, and that Spanish follows

