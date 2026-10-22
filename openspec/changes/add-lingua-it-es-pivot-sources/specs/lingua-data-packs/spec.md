# Spec Delta

## ADDED Requirements

### Requirement: it-es's further sources are derived and pinned like every source
For it-es alone (the Italian programme's I4 and I11), the pack update SHALL derive three further sources and pin each file by its sha256 in it-es's pin, as it pins every source: from the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries' dumps, for each translation table, the Italian and the Spanish words it lists under one sense, kept together with that sense; from the Italian Wiktionary's dump, its Spanish section; and, from a dated Wikidata Lexemes dump mirrored as a release asset of its own, the pairs of an Italian and a Spanish sense linked to the same item. Every source SHALL be written by people on each side; none SHALL be machine-translated. No pack SHALL read these files before the pair that glosses Italian in Spanish is reduced, and no other pair's sources, pin or tables SHALL move.

#### Scenario: A table's sense keeps its words together
- **WHEN** an English entry's translation table lists, under one sense, the Italian `finestra` and the Spanish `ventana`, and under another sense other words
- **THEN** the derived file holds `finestra` and `ventana` together under the first sense, and the other sense's words apart

#### Scenario: A table without both languages
- **WHEN** a translation table lists an Italian word but no Spanish one under a sense
- **THEN** nothing is derived from that sense

#### Scenario: The mirrored Lexemes dump
- **WHEN** the dated Wikidata Lexemes dump the pin names is no longer served by Wikidata
- **THEN** the update fetches the release asset that mirrors it, checked against the pinned sha256

#### Scenario: Nothing else moves
- **WHEN** it-es's further sources are derived
- **THEN** every other pair's sources, pins and tables are byte for byte what they were
