## ADDED Requirements

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
studying French, written wholly in capitals, two letters or more, and its own lemma, and SHALL refuse
any other, naming it. Editing the module SHALL re-pin fr-en and fr-es, and en-fr's, es-fr's, es-en's
and en-es's tables, pins, packs and invariance baselines SHALL not move.

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
