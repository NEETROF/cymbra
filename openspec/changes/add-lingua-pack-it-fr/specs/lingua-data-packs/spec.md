# Spec Delta

## ADDED Requirements

### Requirement: Italian is glossed in French from the French Wiktionary's Italian section and the translation tables
it-fr's glosses, sense runs and expressions SHALL be reduced, by the reducer that writes Italian's studied side, through the native side every pair shares with the French Wiktionary's rules, as es-fr's are, from three sources in this order: the French Wiktionary's Italian section's definitions; the Italian Wiktionary's French translations, in the table's order; the French Wiktionary's translation tables read backwards, a French entry that lists an Italian word glossing that word with the French headword, commonest French word first — at most three words per part of speech from a table, a typographic apostrophe in a headword or in an Italian word of either table read as `'`. Every gloss SHALL be French written by a person, never a machine translation nor a pivot through another language. A sense the source tags as a form of a word that no Italian entry of the section heads, by its exact spelling, and whose text the French edition's form-of pattern does not read as an inflection, SHALL gloss its headword as a definition does — where the target is written in lowercase, only if the headword has no other glossed sense once names' and acronyms' senses are set aside. A proper noun's or an acronym's sense SHALL NOT gloss a lemma the section gives a lowercase entry of its own, even one of form-of senses only. Italian's dictionary words SHALL be those of *Italian's dictionary words leave its names out*, the acronym rows glossed as *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* requires, a synonym's gloss read from its entry in the section whatever its part of speech, and the levels SHALL go to the dictionary words as *Italian's estimated levels* requires. it-fr's expressions SHALL be the section's glossed headwords with a space, then those the translation tables gloss, then the words without a space that Italian's pre-pass splits into two tokens or more, from the section and from the tables, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires.

#### Scenario: A word glossed by its definition
- **WHEN** it-fr is reduced and `casa` is looked up
- **THEN** its gloss is the section's « Maison; Domicile », as one noun run of two senses

#### Scenario: The fallbacks' order
- **WHEN** an Italian lemma the section does not gloss is listed both in an Italian entry's French translations and as the Italian translation of a French entry
- **THEN** it is glossed by the Italian table's French words first, at most three per part of speech, and by the French headwords read backwards only where that table has none

#### Scenario: A gloss read as a link
- **WHEN** the section's entry `del` carries the sense « Du. » tagged as a form of « Du », which no Italian entry heads, and `storie` the sense « Pluriel de storia » tagged as a form of « Pluriel »
- **THEN** `del`'s gloss opens on « Du », and `storie` stays a form, its text read as an inflection

#### Scenario: A link to a lowercase word beside a gloss
- **WHEN** `pelle` has its own glossed senses and a sense « Article contracté … » tagged as a form of a lowercase word no Italian entry heads
- **THEN** `pelle`'s gloss is its own senses' alone

#### Scenario: A surname keeps off a common word
- **WHEN** the section has `soldi` as a lowercase entry and `Soldi` as a surname, and `te` as a lowercase entry and `TE` as a province's acronym
- **THEN** neither `soldi` nor `te` is glossed by the surname or the province

#### Scenario: An acronym through a proper-noun synonym
- **WHEN** it-fr glosses the acronym row `USA`, whose synonym `Stati Uniti` the section files as a proper noun
- **THEN** the row's gloss joins `america`'s first sense and `Stati Uniti`'s « États-Unis », in the row's order, as change 11 requires

#### Scenario: The levels go to dictionary words
- **WHEN** it-fr is reduced with its glosses
- **THEN** every levelled lemma of `tables/it/level.tsv` is a dictionary word of `tables/it/lexical.tsv`

#### Scenario: Expressions from every source
- **WHEN** the section glosses `essere d’accordo` (written with the typographic apostrophe) and `all’improvviso`, and the translation tables gloss `meccanica quantica` and `d'accordo`
- **THEN** all four are expressions of `mwe.tsv`, keyed at build

### Requirement: it-fr is committed at its studied tables' snapshot, held to a coverage floor, and the Italian baseline runs on it
it-fr's native side SHALL be reduced from the pin Italian's studied tables were reduced from and the French and Italian editions' files the pin records, the English edition kept, fetching no other source, and committed beside them, every studied table but the dictionary words and the levels byte for byte; its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, SHALL stay under the size budget, and no package SHALL list it before Italian is enabled. Its coverage — the share of the 5,000, 10,000 and 20,000 commonest Italian lemmas with a gloss — SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["it-fr"]`, fixed before the first committed measurement at the study's figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail. The coverage, the glossed lemmas by source and a sample of glosses marked by source SHALL be shown in it-fr's README and in the sources' record, and stored in no pack. From the pull request that commits it-fr's glosses on, the Italian invariance baseline SHALL run over the pack built from the committed Italian tables, its golden re-blessed once in that pull request.

#### Scenario: The floor holds
- **WHEN** it-fr is reduced and its coverage is at or above `FLOORS["it-fr"]`
- **THEN** the tables are committed and the coverage is shown in the README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces it-fr and its committed tables measure under `FLOORS["it-fr"]`
- **THEN** the checks fail — the floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["it-fr"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The baseline on the committed tables
- **WHEN** the Italian baseline runs after this change
- **THEN** its pack is built from the committed `tables/it/` and `tables/it-fr/`, and its golden is the one this change re-blessed

#### Scenario: Nothing else moves
- **WHEN** it-fr's tables are committed
- **THEN** the six other pairs' tables, pins and packs are byte for byte as before and their baselines pass without re-blessing
