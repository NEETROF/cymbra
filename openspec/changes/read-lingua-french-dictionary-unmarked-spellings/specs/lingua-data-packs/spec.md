## ADDED Requirements

### Requirement: A French spelling the dictionary gives only as a marked word reads as that word when the marked word is the commoner
French's reference reduction, fr-en's, SHALL read as a form of a marked word every word the English Wiktionary's French section gives only as that word written without its accents, cedillas, diaereses or circumflexes — every sense of every entry of the word an alternative, obsolete, archaic, nonstandard or misspelt spelling or form of that one word — when wordfreq, the version the reduction pins, rates the marked word strictly commoner than the spelling and than each of the spelling's forms, the inflections its entries list and the entries pointing at it. Such a spelling SHALL be read as change 43 reads an ASCII spelling of a ligature or a post-1990 spelling: its entries SHALL make it no lemma, no ranked lemma, no French dictionary word, no levelled word and no word fr-en or fr-es glosses; its forms SHALL follow it to the marked word's lemma along their part of speech; its readings SHALL be filed under the marked word; and the marked word SHALL keep its level, readings and glosses. A word whose spelling is the commoner, a spelling with a form commoner than its marked word, and a word with an entry or a sense of its own beside the spelling SHALL keep the reading change 43's rules give it. The reviewed table of spellings read-lingua-french-ca-as-ca-cedilla adds, which reads the spellings no entry links to their word, SHALL neither gain nor lose a row by this rule, and no other word SHALL be read by its marks alone. Since French's analysis looks every token up lowercased, a read spelling SHALL read as its marked word whatever its case. fr-es SHALL read the spellings through the `tables/fr/` fr-en writes, and en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and invariance baselines SHALL not move.

#### Scenario: « etre » reads as « être »
- **WHEN** fr-en is reduced from its pinned sources, the section gives `etre` only as « obsolete spelling of être », and wordfreq rates `être` commoner than `etre`
- **THEN** `tables/fr/forms.tsv` maps `etre` to *être*, `etre` is in none of `freq.tsv`, `lexical.tsv` and `level.tsv` nor in `tables/fr-en/gloss.tsv`, and *être* keeps its level and its gloss

#### Scenario: A spelling's forms follow it
- **WHEN** the section gives `repondre` only as « obsolete spelling of répondre », its entry lists `repond` and `repondu` among its inflections, and wordfreq rates `répondre` commoner than each
- **THEN** `repondre`, `repond` and `repondu` map to *répondre*, and their readings are filed under *répondre*

#### Scenario: A spelling of a form reaches its lemma
- **WHEN** the section gives `etait` only as « obsolete spelling of était », a form of *être*
- **THEN** `etait` maps to *être*

#### Scenario: The commoner spelling keeps its own word
- **WHEN** the section gives `diesel` only as « alternative form of diésel », and wordfreq rates `diesel` commoner than `diésel`
- **THEN** `diesel` maps to *diesel* and keeps its rank, its level and its gloss

#### Scenario: A form commoner than the marked word keeps the spelling apart
- **WHEN** the section gives `teter` only as « alternative form of téter », an entry gives `tete` as one of its inflections, and wordfreq rates `tete` commoner than `téter`
- **THEN** neither `teter` nor `tete` maps to *téter*

#### Scenario: A word with an entry of its own
- **WHEN** the section gives `surs` both as « masculine plural of sur » and as « alternative spelling of sûrs »
- **THEN** `surs` keeps the reading change 43's rules give it and is not read as *sûr*

#### Scenario: The reviewed table is not this rule
- **WHEN** fr-en is reduced
- **THEN** `ca`, `age`, `ages` and `forcement` read as the reviewed table names them, and `ou`, `a`, `sur` and `cote` read as themselves

#### Scenario: Written with a capital
- **WHEN** a reader studying French opens « Etat » in « un coup d'Etat »
- **THEN** the word reads as *état* and the selection's phrase gloss reports the expression `coup d'état`

#### Scenario: A spelling meets its word's expressions
- **WHEN** a French page writes « il etait une fois » and the pack holds the expression `il était une fois`
- **THEN** the selection's phrase gloss reports `il était une fois` over the four words

#### Scenario: fr-es follows French's tables
- **WHEN** fr-es is reduced again on the `tables/fr/` fr-en writes
- **THEN** every gloss, sense run and expression of a lemma it keeps is byte for byte as before, a read spelling shows its marked word's Spanish gloss when fr-es glosses that word, and its pin records the new studied tables and its pack

#### Scenario: Nothing else moves
- **WHEN** the change's tables are committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines pass without re-blessing
