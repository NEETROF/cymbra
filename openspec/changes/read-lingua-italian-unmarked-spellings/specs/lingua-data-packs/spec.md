# Spec Delta

## ADDED Requirements

### Requirement: An Italian spelling the dictionary gives only as a marked word reads as that word when the marked word is the commoner
Italian's reduction SHALL read a form the English Wiktionary's Italian section gives only as a misspelling, alternative, obsolete or nonstandard spelling of one other word — every sense of every entry of it such a spelling or a form of that word, whether the mark it lacks or changes is an accent or a letter; an apocope, an abbreviation, a clipping, a contraction, an initialism, an acronym, a pronunciation spelling or a pointer at several words being no spelling, and a pointer at a piece of Italian's tokenisation pre-pass being the piece's — as a form of that word, when wordfreq rates the word strictly commoner than the spelling and than each of the spelling's own forms; the spelling's paradigm — its inflections, the entries pointing at it, and every form all whose words lie inside it — SHALL follow it where no other word claims a form, and nothing of the spelling's own SHALL remain. A form the tables read through such a spelling alone, and did not read without it, SHALL keep no reading when English, French or Spanish writes it more often than Italian does, or when the section, MarkIT or Morph-it! names it a proper noun. A spelling with a sense of its own beside the marking, or whose senses spell two words, SHALL keep its entry, and so SHALL a spelling the owner keeps apart by name with its reason. The reduction's report SHALL name the spellings read and the ones refused with the test that refused them.

#### Scenario: A dictionary spelling
- **WHEN** the section gives `perchè` only as a misspelling of *perché*, and wordfreq rates *perché* the commoner
- **THEN** `perchè` is a form of *perché* in the tables, with no rank of its own

#### Scenario: A spelling with a sense of its own
- **WHEN** a spelling the section marks as another word's is also a word with a sense of its own (`e'`, also the apocope of *ei*)
- **THEN** it keeps its own entry and is not read by the rule

#### Scenario: Two words
- **WHEN** the section gives `nè` as a misspelling of *né* and as a misspelling of *neh*
- **THEN** it is not read by the rule, and the reduction's report says so

#### Scenario: The marked word is not the commoner
- **WHEN** the section marks a spelling as a word wordfreq rates rarer than the spelling
- **THEN** the spelling is not read by the rule, and the reduction's report says so

#### Scenario: A spelling the owner keeps apart
- **WHEN** the section gives `casinò` as an alternative form of *casino*, and the owner keeps it apart
- **THEN** it keeps its own entry, and the reduction's report names it among the spellings kept apart

#### Scenario: A form another word claims
- **WHEN** an inflection of a spelling the rule reads is also another word's form (`sguardi`, a row of the archaic `sguardare` and *sguardo*'s plural), or a form of one of its forms is (`accise`, a row of `acciso`, the participle of the archaic `accidere`, and *accisa*'s plural)
- **THEN** it stays that other word's form, and an inflection no other word claims follows the spelling to its word

#### Scenario: Another language's word, or a name
- **WHEN** a form the tables did not read before reads only through a spelling, and English writes it more often than Italian (`intro`, a row of the archaic `intrare`), or Morph-it! lists it as a proper noun (`abbado`)
- **THEN** it keeps no reading

### Requirement: The web's Italian spellings without their marks read as the words a reviewed list names
Italian's reduction SHALL read as a form of its accented word each spelling of a reviewed list — an Italian form of the tables ending in an accented vowel, written without the accent — kept when wordfreq attests the spelling at Zipf 2.5 or more, the section gives it no meaning of its own (a form of a word counting as one, a name's entry not) and does not give it as a spelling of its word, its accented word's lemma has a sense of its own in the section that is no name's (Italian's dictionary words, once written, re-checking every row), the spelling is less than half as frequent as that word, and neither English, French nor Spanish writes it more often than Italian does; and each exception row the owner settles outside those tests, with its reason, for a spelling the section gives as a rare homograph of the web's commonest spelling of a word, or the web's commonest misspelling of one (`pò` for *po'*). A dictionary update that names a listed spelling SHALL stop the reduction for a person, and so SHALL a row whose word the tables no longer rank; an entry pointing at a listed spelling SHALL follow it. A spelling that is a word of its own — `e`, `da`, `si`, `se`, `ne`, `la`, `li`, `eta`, `pero` —, one commoner than its word (`papa`), or one another language of the matrix writes more (`cafe`) SHALL NOT be listed. A listed spelling SHALL carry no rank, level, gloss or dictionary-word bit of its own and, once Italian's readings exist, its word's readings.

#### Scenario: A web spelling
- **WHEN** « piu », « puo » and « cioe » are looked up after the tables are reduced
- **THEN** each reads as `più`, *potere* (`può`'s word) and `cioè`

#### Scenario: The owner's exceptions
- **WHEN** « perche », « cosi », « citta », « caffe », « gia », « liberta » and « pò » — which the section gives as *perca*'s plural, *coso*'s plural, *citto*'s feminine, *caffo*'s feminine plural, *gire*'s archaic imperfect, *liberto*'s feminine and an archaic form of *potere* — are looked up after the tables are reduced with the seven exception rows
- **THEN** each reads as `perché`, `così`, `città`, `caffè`, `già`, `libertà` and `poco`

#### Scenario: A word of its own
- **WHEN** « e », « da », « papa », « eta » and « pero » are looked up
- **THEN** each reads as its own word (*and*, *from*, *pope*, the letter, the pear tree), never as *è*, *dà*, *papà*, *età* or *però*

#### Scenario: Below the threshold
- **WHEN** an accent-less spelling wordfreq attests under Zipf 2.5 is looked up
- **THEN** it is not a form, and reads as an unknown word

#### Scenario: A dictionary update names a row
- **WHEN** a later dump gives one of the listed spellings an entry of its own
- **THEN** the reduction stops, naming the row, for a person to settle it

#### Scenario: A row's word no longer ranked
- **WHEN** a row names a word the reduced tables do not rank
- **THEN** the reduction stops, naming the row: a row follows the tables' choices and never makes one

#### Scenario: Nothing of its own
- **WHEN** the it-fr pack is built with the list
- **THEN** no listed spelling has a rank, a level, a gloss or a dictionary-word bit of its own
