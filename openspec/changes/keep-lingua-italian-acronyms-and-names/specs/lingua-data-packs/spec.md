# Spec Delta

## ADDED Requirements

### Requirement: Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms
Italian's reference reduction (it-fr's) and every pair glossing Italian SHALL read an Italian word as an acronym only from one reviewed table of an Italian rule module every pair glossing Italian imports, each row naming the acronym as written — wholly in capitals, two letters or more —, the expansions and the synonyms the English Wiktionary's Italian section's own entry for it gives, its part of speech and its reason; a row SHALL exist only where the section has an entry headed by the acronym and the acronym's lowercase is a form of another word of the tables, and a row SHALL be added only as a person's decision. The reference reduction SHALL write each row into Italian's studied tables as a form that is its own lemma, its capitals kept, with no rank and no level, the lowercase spelling keeping its own reading (`nato` stays a form of *nascere*), and as one of Italian's dictionary words whatever its sense runs and whether glossed, from the first reduction holding the row on. A pair SHALL gloss a row from its own glosses of the row's expansions, each cut to its first sense, in the row's order, an expansion it does not gloss passed over; when it glosses no expansion, from its own glosses of the row's synonyms the same way; and SHALL write no gloss when it glosses neither — never a gloss of its own words, a translation, or the entry's English. Editing the module SHALL re-pin it-fr, and the six other pairs' tables, pins, packs and invariance baselines SHALL NOT move.

#### Scenario: The rows
- **WHEN** it-fr is reduced with the reviewed table holding `NATO` and `USA`
- **THEN** `tables/it/forms.tsv` maps `NATO` to `NATO` and `USA` to `USA`, the forms holding a capital, `freq.tsv` and `level.tsv` hold neither, and `nato` and `usa` still map to *nascere* and *usare*

#### Scenario: A pair glosses the synonyms
- **WHEN** it-fr glosses `USA`, whose entry points at the English name and names the synonyms `America` and `Stati Uniti`
- **THEN** the row's gloss is it-fr's own glosses of `america` and `stati uniti`, each cut to its first sense, joined

#### Scenario: No expansion or synonym glossed
- **WHEN** it-fr glosses `NATO`, whose entry names no expansion and no synonym
- **THEN** it writes no gloss for the acronym, which stays a form of the pack and one of Italian's dictionary words

#### Scenario: A row is a decision
- **WHEN** the table is read
- **THEN** every row is written wholly in capitals, carries its reason, is headed by an entry of the section, and names only expansions and synonyms that entry gives; a row with no entry (`ANSA`) fails the reduction naming it

### Requirement: Italian's dictionary words leave its names out
Italian's dictionary words SHALL be its reference pair's glossed lemmas less those the reference's edition — the French Wiktionary's Italian section — files as a proper noun in every sense run, by the rule the three other studied languages follow, plus the acronym rows of *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms*, whatever their sense runs and whether glossed. The checks over the committed tables and the builder's check of a studied language's dictionary words SHALL hold `tables/it/lexical.tsv` to that set — the rows alone while the reference glosses nothing —, the rows exempted from the rule that a dictionary word is a ranked, glossed lemma that is no name.

#### Scenario: A name
- **WHEN** it-fr glosses `roma` by a proper noun's senses alone
- **THEN** `roma` is not a dictionary word, carries its gloss, and a selection still opens its card

#### Scenario: A word beside a name
- **WHEN** it-fr glosses `marco` by a name's sense run and a noun's (the coin)
- **THEN** `marco` is a dictionary word

#### Scenario: The rows are dictionary words
- **WHEN** `tables/it/lexical.tsv` lists `NATO`, glossed by no sense run, and `USA`, glossed by a proper noun's run
- **THEN** the checks accept both as dictionary words

#### Scenario: The checks
- **WHEN** `tables/it/lexical.tsv` lists a lemma it-fr glosses by proper-noun senses alone that is no row, or leaves out one it glosses otherwise
- **THEN** the checks fail, naming it-fr and the lemma

## MODIFIED Requirements

### Requirement: French's acronyms are a reviewed table, glossed through their expansions
French's reference reduction (fr-en's) and every pair glossing French SHALL read a French word as an acronym only from one reviewed table of a French rule module they both import, each row naming the acronym as written — wholly in capitals, two letters or more —, the expansions the English Wiktionary's French section's own entry for it points at, its part of speech and its reason; the table SHALL hold the row `CA` (`conseil d'administration`, `chiffre d'affaires`, `courant alternatif`, `comptable agréé`, a noun), and a row SHALL be added only as a person's decision.
The reference reduction SHALL write each row into French's studied tables as a form that is its own
lemma, its capitals kept, and as one of French's dictionary words, with no rank, no level and no
reading, the lowercase spelling keeping its own reading (`ca` stays a form of *ça*). Each pair SHALL
gloss a row from its own glosses of the row's expansions: for each expansion, in the row's order, its
gloss cut to its first sense, an expansion it does not gloss passed over, the senses joined as one run
of the row's part of speech, and no gloss when it glosses none — never a gloss of its own words, a
translation, or the expansion's French. An acronym row SHALL be the one kind of lemma a pair glosses
that is not ranked. The builder SHALL accept a form or a lemma holding a capital only in a pack
studying French or Italian, written wholly in capitals, two letters or more, and its own lemma, and
SHALL refuse any other, naming it. Editing the module SHALL re-pin fr-en and fr-es, and en-fr's,
es-fr's, es-en's and en-es's tables, pins, packs and invariance baselines SHALL not move.

#### Scenario: The row in French's tables
- **WHEN** fr-en is reduced from its pinned sources
- **THEN** `tables/fr/forms.tsv` maps `CA` to `CA`, the one form holding a capital, `tables/fr/lexical.tsv` lists `CA`, `freq.tsv`, `level.tsv` and `grammar.tsv` hold no `CA`, and `ca` still maps to `ça`

#### Scenario: Each pair glosses the acronym in its reader's language
- **WHEN** fr-en and fr-es are reduced
- **THEN** fr-en glosses `CA` « board of directors; revenue, turnover, net sales; alternating current » and fr-es « Administración, consejo de administración; Ingreso, cifra de negocios, facturación; Corriente alterna », each as one noun run of three senses, `comptable agréé`, glossed by neither, passed over

#### Scenario: No expansion glossed
- **WHEN** a pair glosses none of a row's expansions
- **THEN** it writes no gloss for the acronym, which stays a form of the pack and one of French's dictionary words

#### Scenario: A row is a decision
- **WHEN** the table is read
- **THEN** every row is written wholly in capitals, carries its reason, and names only expansions the section's own entry for it points at

#### Scenario: A stray capital
- **WHEN** a table hands the builder a form `Paris`, a form `Ca`, or a form `CA` read as another lemma, or a pack studying English or Spanish holds a form in capitals
- **THEN** the build fails and names the form

#### Scenario: The acronym has no rank
- **WHEN** fr-en's pack is loaded
- **THEN** `CA` has no rank and no level, is one of the pack's dictionary words, is not counted by the vocabulary estimate, and every other lemma fr-en glosses is a ranked lemma read as itself

#### Scenario: Nothing else moves
- **WHEN** the change's tables are committed
- **THEN** every other row of `tables/fr/`, of fr-en's and of fr-es's glosses, sense runs and expressions is byte for byte as before, and en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, their invariance baselines passing without re-blessing
