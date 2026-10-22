# Spec Delta

## ADDED Requirements

### Requirement: it-es's further sources are derived and pinned like every source
The pack update SHALL register and derive every file it-es reads and pin each by its sha256 in it-es's pin, as it pins every source: the Spanish Wiktionary's Italian section and Italian translations and the Italian Wiktionary's Spanish translations; from the English, French, German, Portuguese, Russian, Polish and Dutch Wiktionaries' dumps, for each translation table, the Italian and the Spanish words it lists under one sense — or under the whole entry when the table names none, marked so —, kept together; the Italian Wiktionary's Spanish section; and, from a dated Wikidata Lexemes dump read as an edition of its own, the pairs of an Italian and a Spanish sense linked to the same item or by a translation link. Each Italian and Spanish word SHALL be one a person wrote; their pairing SHALL be inferred from a shared sense, entry or item, never machine-translated. Only it-es's pin SHALL name the pivot and lexeme files; no pack SHALL read them before it-es is reduced, and no other pair's sources, pin or tables SHALL move.

#### Scenario: A table's sense keeps its words together
- **WHEN** an English entry's translation table lists, under one sense, the Italian `finestra` and the Spanish `ventana`, and under another sense other words
- **THEN** the derived file holds `finestra` and `ventana` together under the first sense, and the other sense's words apart

#### Scenario: A table numbering its senses only
- **WHEN** a Polish entry's translations are keyed by sense numbers alone
- **THEN** each number's Italian and Spanish words are kept apart from the other numbers'

#### Scenario: A table naming no sense
- **WHEN** a French entry's translation table names no sense
- **THEN** its Italian and Spanish words are kept under the whole entry, marked as naming none

#### Scenario: A table without both languages
- **WHEN** a translation table lists an Italian word but no Spanish one under a sense
- **THEN** nothing is derived from that sense

#### Scenario: The lexemes on a re-reduction
- **WHEN** it-es is reduced again from its pin
- **THEN** the lexeme pairs are read from it-es's sources release, checked against their sha256, and no Lexemes dump is fetched

#### Scenario: Nothing else moves
- **WHEN** it-es's sources are derived
- **THEN** every other pair's sources, pins and tables are byte for byte what they were
