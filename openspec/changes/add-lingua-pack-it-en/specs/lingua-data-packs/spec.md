# Spec Delta

## ADDED Requirements

### Requirement: Italian is glossed in English from the English Wiktionary's Italian section and the Italian Wiktionary's English translations
it-en's glosses, sense runs and expressions SHALL be reduced by a reducer of its own, reading Italian's studied tables as committed, through the native side every pair shares with the English Wiktionary's rules, from the English Wiktionary's Italian section's definitions, then the Italian Wiktionary's English translations, at most three words per part of speech, and never from the English Wiktionary's translation tables read backwards. Every gloss SHALL be English written by a person. Before the shared rules, it-en's own pre-pass SHALL read the meaning a pointer sense carries after its pointer, where the pointer names a word but the sense also gives its meaning; SHALL keep a proper noun's or an acronym's sense off a lemma the section gives a lowercase entry of its own, and a name's run off a function word; SHALL gloss no word with a letter; and SHALL take out notes that are not glosses. The acronym rows of Italian's studied tables SHALL be glossed by *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* from it-en's own glosses. it-en's expressions SHALL be the section's glossed headwords with a space, then those the Italian Wiktionary's English translations gloss, then the words without a space that Italian's pre-pass splits and the section glosses with a meaning, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires. The English edition's shared rules, and every other pair's reducer, tables and pin, SHALL NOT move.

#### Scenario: A meaning a pointer carries
- **WHEN** the section's `del` carries the sense « contraction of di + il; of the, from the »
- **THEN** `del`'s gloss is « of the, from the », not the pointer's wording

#### Scenario: A name off a common word
- **WHEN** the section has `torno` as a lowercase entry glossed « lathe » and `Torno` as a name
- **THEN** `torno` is glossed « lathe » alone

#### Scenario: A name off a function word
- **WHEN** the section's `io` « I » shares its row with the moon `Io`
- **THEN** `io`'s gloss holds no run of the moon

#### Scenario: A gloss from the Italian Wiktionary
- **WHEN** the section has no entry for `perfino` and the Italian Wiktionary's `perfino` lists the English translation « even »
- **THEN** `perfino` is glossed « even »

#### Scenario: No table read backwards
- **WHEN** an English entry of the English Wiktionary lists an Italian word as its translation, and neither the section nor the Italian Wiktionary glosses that word
- **THEN** it-en does not gloss it

#### Scenario: Expressions from every source
- **WHEN** the section glosses `un po'` and `d'accordo`, and the Italian Wiktionary translates `meccanica quantica` into English
- **THEN** all three are expressions of `mwe.tsv`, keyed at build

#### Scenario: Nothing else moves
- **WHEN** it-en's tables are committed
- **THEN** `reduce_edition_en.py` and `reduce_common.py` are byte for byte what they were, and every other pair's tables, pins and goldens are unchanged

### Requirement: it-en is committed beside Italian's studied tables, held to a coverage floor, and not shipped
it-en's native side SHALL be reduced from its own pin — the English Wiktionary's Italian section as change 9 pins it, and the Italian dump's derived translations — and committed in `tables/it-en/`, reading `tables/it/` as committed and writing none of it; its pin SHALL record the studied tables it read; its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, and SHALL stay under the size budget. Its coverage of the 5,000, 10,000 and 20,000 commonest Italian lemmas SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["it-en"]`, fixed before the first committed measurement at the study's figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail the checks. An it-en invariance baseline SHALL run over the Italian corpus with the committed it-en pack beside it-fr's, the two packs reading every Italian word alike. No package SHALL list it-en before Italian is enabled for English speakers.

#### Scenario: The floor holds
- **WHEN** it-en is reduced and its coverage is at or above `FLOORS["it-en"]`
- **THEN** the tables are committed and the coverage is shown in their README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces it-en and its committed tables measure under `FLOORS["it-en"]`
- **THEN** the checks fail — the floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["it-en"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: One studied side, two natives
- **WHEN** the Italian corpus is read with the it-fr and the it-en packs
- **THEN** every token, lemma, reading and level is the same, and only the glosses differ

#### Scenario: Not shipped
- **WHEN** it-en's tables are committed
- **THEN** no package lists it-en, and no reader's package or engine moves
