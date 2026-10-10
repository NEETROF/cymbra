## ADDED Requirements

### Requirement: A French spelling without its cedilla reads as the word a reviewed table names
French's reference reduction, fr-en's, SHALL read a word French writes without its cedilla or accent as another word only when a reviewed table of its reducer names it, each row with its reason, and the table SHALL hold the row `ca` → *ça* and no other. A form the table names SHALL be a form of the named word alone: none of its own entries in the English Wiktionary's French section SHALL make it a lemma, a ranked lemma, a French dictionary word, a levelled word or a word fr-en glosses, nor list an inflection of it, and no entry's inflection table SHALL make it a form of another word. A word the table does not name SHALL keep the reading change 43's rules give it — `ou`, `a`, `la`, `des`, `du`, `sur` and `cote` among them. Since the pack keys its forms in lower case and French's analysis looks every token up lowercased, the named spelling SHALL read as the named word whatever its case. fr-es SHALL read the spelling through the `tables/fr/` fr-en writes, its own glosses untouched by the row, and en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and invariance baselines SHALL not move.

#### Scenario: « ca » reads as « ça »
- **WHEN** fr-en is reduced from its pinned sources
- **THEN** `tables/fr/forms.tsv` maps `ca` to *ça*, `ca` is not in `freq.tsv`, `lexical.tsv` or `level.tsv` nor in `tables/fr-en/gloss.tsv`, and *ça* keeps its rank and its level

#### Scenario: The acronym and circa leave the pack
- **WHEN** the French section lists `CA` as a noun with four initialisms and the plural `CAs`, and `ca` as « abbreviation of circa »
- **THEN** neither entry makes `ca` a lemma or gives it a gloss, and `cas` is no form of `ca`

#### Scenario: Written in capitals
- **WHEN** a reader studying French opens « CA » in « Le CA a voté le budget. »
- **THEN** the word reads as *ça*, as « ca » and « Ca » do

#### Scenario: A spelling meets its word's expressions
- **WHEN** a French page writes « comme ca » and the pack holds the expression `comme ça`
- **THEN** the selection's phrase gloss reports `comme ça` over both words

#### Scenario: A word of its own is not read by its marks
- **WHEN** fr-en is reduced
- **THEN** `ou` maps to *ou*, `a` to *avoir* and `sur` to *sur*, not to *où*, *à* or *sûr*

#### Scenario: A row is a decision
- **WHEN** the reducer's table of spellings is read
- **THEN** every row names a ranked lemma that reads as itself, carries its reason, and is neither an elided piece nor `du` or `des`

#### Scenario: fr-es follows French's tables
- **WHEN** fr-es is reduced again on the `tables/fr/` fr-en writes
- **THEN** its glosses, senses and expressions are byte for byte as before, and its pin records the new studied tables and its pack

#### Scenario: Nothing else moves
- **WHEN** the change's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing
