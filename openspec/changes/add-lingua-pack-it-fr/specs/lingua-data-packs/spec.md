# Spec Delta

## ADDED Requirements

### Requirement: Italian is glossed in French from the French Wiktionary's Italian section and the translation tables
it-fr's glosses, sense runs and expressions SHALL be reduced, by the reducer that writes Italian's studied side, from the French Wiktionary's Italian section first, then from the French Wiktionary's translation tables read backwards — a French entry that lists an Italian word glossing that word with the French headword —, then from the Italian Wiktionary's French translations, at most three words per part of speech from a table, through the native side every pair shares with the French Wiktionary's rules, as es-fr's are; every gloss SHALL be French written by a person, never a machine translation nor a pivot through another language. A sense the source tags as a form of a capitalised word that no Italian entry of the section heads, and whose text the French edition's form-of patterns do not read as an inflection, SHALL gloss its headword as a definition does. Italian's dictionary words SHALL be it-fr's glossed lemmas less those the section glosses by a proper noun's senses alone, plus the acronym rows; the acronym rows SHALL be glossed as *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* requires; and the estimated levels SHALL go to the dictionary words alone. it-fr's expressions SHALL be the section's glossed headwords with a space and the words without one that Italian's pre-pass splits into two tokens or more, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires.

#### Scenario: A word glossed by its definition
- **WHEN** it-fr is reduced and `casa` is looked up
- **THEN** its gloss is the section's « Maison; Domicile… », as one noun run

#### Scenario: A gloss read as a link
- **WHEN** the section's entry `del` carries the sense « Du. » tagged as a form of « Du », which no Italian entry heads, and `del` is a ranked lemma
- **THEN** `del` is glossed « Du », and a sense « Pluriel de giorno. » tagged as a form of `giorno` stays a form

#### Scenario: A translation table glosses what the section leaves out
- **WHEN** an Italian lemma the section does not gloss is listed as the Italian translation of a French entry
- **THEN** it is glossed by that French headword, at most three words per part of speech

#### Scenario: Names leave the words to learn
- **WHEN** it-fr glosses `roma` by a proper noun's senses alone and `marco` by a name's and a noun's
- **THEN** `roma` is no dictionary word and carries its gloss, `marco` is one, and the acronym rows are dictionary words

#### Scenario: The levels go to dictionary words
- **WHEN** it-fr is reduced with its glosses
- **THEN** every levelled lemma of `tables/it/level.tsv` is a dictionary word of `tables/it/lexical.tsv`

#### Scenario: An expression without a space
- **WHEN** the section glosses `d'accordo`
- **THEN** it is an expression of `mwe.tsv`, keyed `di accordo` at build

### Requirement: it-fr is committed at its studied tables' snapshot, held to a coverage floor, and the Italian baseline runs on it
it-fr's native side SHALL be reduced from the pin Italian's studied tables were reduced from and the French and Italian editions' files the pin records, fetching no other source, and committed beside them, every studied table but the dictionary words and the levels byte for byte; its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, SHALL stay under the size budget, and no package SHALL list it before Italian is enabled. Its coverage — the share of the 5,000, 10,000 and 20,000 commonest Italian lemmas with a gloss — SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["it-fr"]`, fixed before the first committed measurement and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail. The coverage, the glossed lemmas by source and a sample of glosses marked by source SHALL be shown in it-fr's README and in the sources' record, and stored in no pack. From the pull request that commits it-fr's glosses on, the Italian invariance baseline SHALL run over the pack built from the committed Italian tables, its golden re-blessed once in that pull request.

#### Scenario: The floor holds
- **WHEN** it-fr is reduced and its coverage is at or above `FLOORS["it-fr"]`
- **THEN** the tables are committed and the coverage is shown in the README

#### Scenario: Under the floor
- **WHEN** a later reduction of it-fr measures under `FLOORS["it-fr"]`
- **THEN** the update fails, naming the band and the figure

#### Scenario: The baseline on the committed tables
- **WHEN** the Italian baseline runs after this change
- **THEN** its pack is built from the committed `tables/it/` and `tables/it-fr/`, and its golden is the one this change re-blessed

#### Scenario: Nothing else moves
- **WHEN** it-fr's tables are committed
- **THEN** the six other pairs' tables, pins and packs are byte for byte as before and their baselines pass without re-blessing
