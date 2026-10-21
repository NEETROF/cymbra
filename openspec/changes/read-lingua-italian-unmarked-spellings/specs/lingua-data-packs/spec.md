# Spec Delta

## ADDED Requirements

### Requirement: An Italian spelling the dictionary gives only as a marked word reads as that word when the marked word is the commoner
Italian's reduction SHALL read a form the English Wiktionary's Italian section gives only as a misspelling, alternative, obsolete or nonstandard spelling of a word written with its marks — every sense of every entry of it such a spelling or form of that word — as a form of that word, when wordfreq rates the marked word strictly commoner than the spelling and than each of the spelling's own forms; the spelling's inflections and the entries pointing at it SHALL follow it, and nothing of the spelling's own SHALL remain — no rank, no level, no dictionary word, no gloss. A spelling with a sense of its own beside the marking, or one the marked word is not commoner than, SHALL NOT be read by it.

#### Scenario: A dictionary spelling
- **WHEN** the section gives `perche` only as a misspelling of *perché*, and wordfreq rates *perché* the commoner
- **THEN** `perche` is a form of *perché* in the tables, with no rank of its own

#### Scenario: A spelling with a sense of its own
- **WHEN** a spelling the section marks as another word's is also a word with a sense of its own
- **THEN** it keeps its own entry and is not read by the rule

#### Scenario: The marked word is not the commoner
- **WHEN** the section marks a spelling as a word wordfreq rates rarer than the spelling
- **THEN** the spelling is not read by the rule, and the tables say so

### Requirement: The web's Italian spellings without their marks read as the words a reviewed list names
Italian's reduction SHALL read as a form of its accented word each spelling of a reviewed list — an Italian form of the tables ending in an accented vowel, written without the accent — kept when wordfreq attests the spelling at Zipf 2.5 or more, the section gives it no meaning of its own and does not give it as a spelling of its word, its accented word is an Italian dictionary word, the spelling is less than half as frequent as that word, and no other language of the matrix writes it more often than Italian does; a spelling that is also an Italian word (`e`, `da`, `si`, `se`, `ne`, `la`, `li`, `papa`, `eta`) SHALL NOT be on the list. Each row SHALL carry its reason, and a spelling SHALL carry its word's readings and nothing of its own.

#### Scenario: A web spelling
- **WHEN** « cosi », « citta », « piu » and « puo » are looked up after the tables are reduced
- **THEN** each reads as `così`, `città`, `più` and `può`, with that word's readings

#### Scenario: A word of its own
- **WHEN** « e », « da » and « papa » are looked up
- **THEN** each reads as its own word (*and*, *from*, *pope*), never as *è*, *dà* or *papà*

#### Scenario: Below the threshold
- **WHEN** an accent-less spelling wordfreq attests under Zipf 2.5 is looked up
- **THEN** it is not a form, and reads as an unknown word

#### Scenario: Nothing of its own
- **WHEN** the it-fr pack is built with the list
- **THEN** no listed spelling has a rank, a level, a gloss or a dictionary-word bit of its own
