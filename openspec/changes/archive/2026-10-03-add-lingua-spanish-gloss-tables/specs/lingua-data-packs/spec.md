## ADDED Requirements

### Requirement: The Spanish pack's French glosses
The es-fr pack SHALL gloss its lemmas and expressions in French from the French Wiktionary's Spanish entries, through the rules every pair shares. Where those say nothing, it SHALL fall back, in order, on the French translations the Spanish Wiktionary lists for the Spanish entry, then on the French entries whose translation tables list it. A gloss SHALL never be English or machine-translated, and a proper noun's translation SHALL gloss nothing. Curated locutions, whose glosses a person writes, SHALL win over every source. The share of the commonest lemmas glossed SHALL be published with the tables.

#### Scenario: The French Wiktionary glosses a word
- **WHEN** the tables are reduced and the French Wiktionary has a Spanish entry for `casa`
- **THEN** `casa` is glossed from that entry (« Maison »), with its senses grouped by part of speech

#### Scenario: A translation fills a gap
- **WHEN** the tables are reduced and the French Wiktionary has no Spanish entry for `sector`, while the Spanish Wiktionary lists *secteur* among its French translations
- **THEN** `sector` is glossed « Secteur », as a noun

#### Scenario: A proper noun's translation glosses nothing
- **WHEN** a French entry for a place name lists its Spanish name as a translation
- **THEN** that Spanish name takes no gloss from it

### Requirement: Sources derived from whole Wiktionary dumps are pinned
A pair MAY read kaikki's dump of a whole Wiktionary edition. The pipeline SHALL never keep such a dump whole: it SHALL keep each file the pair derives from it (a language's entries, or the translations its entries list into another language) as an asset of the snapshot's release, recorded in `pin.json` by the sha256 of its decompressed bytes, and SHALL refuse a fetched file whose bytes differ.

#### Scenario: An update keeps the derived files
- **WHEN** `lingua-pack-update` reads today's sources for es-fr
- **THEN** the files derived from the French and Spanish Wiktionaries' dumps are recorded in `pin.json` and published with the snapshot's release, and the dumps are not kept

#### Scenario: A re-reduction refuses other bytes
- **WHEN** `build.sh --reduce es-fr` fetches a derived file whose decompressed sha256 differs from `pin.json`
- **THEN** it fails, naming the source and the file
