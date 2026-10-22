# Spec Delta

## ADDED Requirements

### Requirement: An Italian spelling the dictionary gives only as a marked word reads as that word when the marked word is the commoner
Italian's reduction SHALL read a form the English Wiktionary's Italian section gives only as a misspelling, alternative, obsolete or nonstandard spelling of another word — every sense of every entry of it such a spelling or a form of that word, whether the mark it lacks or changes is an accent or a letter — as a form of that word, when wordfreq rates the word strictly commoner than the spelling and than each of the spelling's own forms; the spelling's inflections and the entries pointing at it SHALL follow it, and nothing of the spelling's own SHALL remain. A spelling with a sense of its own beside the marking SHALL keep its entry. The reduction's report SHALL name the spellings read and the ones refused with the test that refused them.

#### Scenario: A dictionary spelling
- **WHEN** the section gives `perchè` only as a misspelling of *perché*, and wordfreq rates *perché* the commoner
- **THEN** `perchè` is a form of *perché* in the tables, with no rank of its own

#### Scenario: A spelling with a sense of its own
- **WHEN** a spelling the section marks as another word's is also a word with a sense of its own
- **THEN** it keeps its own entry and is not read by the rule

#### Scenario: The marked word is not the commoner
- **WHEN** the section marks a spelling as a word wordfreq rates rarer than the spelling
- **THEN** the spelling is not read by the rule, and the reduction's report says so

### Requirement: The web's Italian spellings without their marks read as the words a reviewed list names
Italian's reduction SHALL read as a form of its accented word each spelling of a reviewed list — an Italian form of the tables ending in an accented vowel, written without the accent — kept when wordfreq attests the spelling at Zipf 2.5 or more, the section gives it no meaning of its own (a form of a word counting as one) and does not give it as a spelling of its word, its accented word's lemma has a sense of its own in the section that is no name's (Italian's dictionary words, once written, re-checking every row), the spelling is less than half as frequent as that word, and neither English, French nor Spanish writes it more often than Italian does; and each exception row the owner settles outside those tests, with its reason, for a spelling the section gives as a rare homograph of the web's commonest spelling. A dictionary update that names a listed spelling SHALL stop the reduction for a person, and an entry pointing at a listed spelling SHALL follow it. A spelling that is a word of its own — `e`, `da`, `si`, `se`, `ne`, `la`, `li` —, one commoner than its word (`papa`), or one another language of the matrix writes more (`eta`, `pero`, `cafe`) SHALL NOT be listed. A listed spelling SHALL carry no rank, level, gloss or dictionary-word bit of its own and, once Italian's readings exist, its word's readings.

#### Scenario: A web spelling
- **WHEN** « piu », « puo » and « gia » are looked up after the tables are reduced
- **THEN** each reads as `più`, `può` and `già`

#### Scenario: The owner's exceptions
- **WHEN** « perche », « cosi », « citta » and « caffe » — which the section gives as *perca*'s plural, *coso*'s plural, *citto*'s feminine and *caffo*'s feminine plural — are looked up after the tables are reduced with the owner's four rows
- **THEN** each reads as `perché`, `così`, `città` and `caffè`

#### Scenario: A word of its own
- **WHEN** « e », « da », « papa », « eta » and « pero » are looked up
- **THEN** each reads as its own word (*and*, *from*, *pope*, the letter, the pear tree), never as *è*, *dà*, *papà*, *età* or *però*

#### Scenario: Below the threshold
- **WHEN** an accent-less spelling wordfreq attests under Zipf 2.5 is looked up
- **THEN** it is not a form, and reads as an unknown word

#### Scenario: A dictionary update names a row
- **WHEN** a later dump gives one of the listed spellings an entry of its own
- **THEN** the reduction stops, naming the row, for a person to settle it

#### Scenario: Nothing of its own
- **WHEN** the it-fr pack is built with the list
- **THEN** no listed spelling has a rank, a level, a gloss or a dictionary-word bit of its own
