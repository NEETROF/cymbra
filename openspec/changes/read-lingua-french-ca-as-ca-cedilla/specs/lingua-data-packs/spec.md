## ADDED Requirements

### Requirement: A French spelling without its marks reads as the word a reviewed table names
French's reference reduction, fr-en's, SHALL read a word French writes without its cedilla or accent as another word only when a reviewed table of its reducer names it, each row with its reason; the table SHALL hold the rows `ca` → *ça*, `age` → *âge*, `ages` → *âge* and `forcement` → *forcément*, and SHALL never hold `cote`, `tache`, `pale`, `foret` or `aine`, words a reader can mean. A form the table names SHALL be a form of the named word alone, whether or not an entry of the English Wiktionary's French section names it: none of its own entries SHALL make it a lemma, a ranked lemma, a French dictionary word, a levelled word or a word fr-en glosses, nor list an inflection of it, and no entry's inflection table SHALL make it a form of another word. A word the table does not name SHALL keep the reading change 43's rules give it — `ou`, `a`, `la`, `des`, `du`, `sur` and `cote` among them. Since the pack keys its forms in lower case and French's analysis looks every token up lowercased, a named spelling SHALL read as the named word whatever its case, until a rule of French's analysis reads a word written in capitals apart. fr-es SHALL read the spellings through the `tables/fr/` fr-en writes, and en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and invariance baselines SHALL not move.

#### Scenario: « ca » reads as « ça »
- **WHEN** fr-en is reduced from its pinned sources
- **THEN** `tables/fr/forms.tsv` maps `ca` to *ça*, `ca` is not in `freq.tsv`, `lexical.tsv` or `level.tsv` nor in `tables/fr-en/gloss.tsv`, and *ça* keeps its rank and its level

#### Scenario: The acronym and circa leave the pack
- **WHEN** the French section lists `CA` as a noun with four initialisms and the plural `CAs`, and `ca` as « abbreviation of circa »
- **THEN** neither entry makes `ca` a lemma or gives it a gloss, and `cas` is no form of `ca`

#### Scenario: A rare word an unmarked spelling swamps
- **WHEN** fr-en is reduced and the section glosses `age` « beam (central bar of a plough); shaft » and `forcement` « fixing number, cooking the books »
- **THEN** `age` and `ages` map to *âge* and `forcement` to *forcément*, none of the three is a ranked lemma, a dictionary word, a levelled word or a glossed word, and `forcements`, the noun's plural, is no form

#### Scenario: Written in capitals, until the acronym rule
- **WHEN** no rule of French's analysis reads a word written in capitals apart, and a reader studying French opens « CA » in « Le CA a voté le budget. »
- **THEN** the word reads as *ça*, as « ca » and « Ca » do

#### Scenario: A spelling meets its word's expressions
- **WHEN** a French page writes « comme ca » and the pack holds the expression `comme ça`
- **THEN** the selection's phrase gloss reports `comme ça` over both words

#### Scenario: A word of its own is not read by its marks
- **WHEN** fr-en is reduced
- **THEN** `ou` maps to *ou*, `a` to *avoir*, `sur` to *sur* and `cote` to *cote*, not to *où*, *à*, *sûr* or *côté*

#### Scenario: A row is a decision
- **WHEN** the reducer's table of spellings is read
- **THEN** every row names a ranked lemma that reads as itself, carries its reason, and is neither an elided piece nor `du` or `des`

#### Scenario: fr-es follows French's tables
- **WHEN** fr-es is reduced again on the `tables/fr/` fr-en writes
- **THEN** every gloss, sense run and expression it had is byte for byte as before, a lemma entering the cut may gain its gloss, and its pin records the new studied tables and its pack

#### Scenario: Nothing else moves
- **WHEN** the change's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing
