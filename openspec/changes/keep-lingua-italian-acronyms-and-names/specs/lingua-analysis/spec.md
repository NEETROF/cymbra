# Spec Delta

## ADDED Requirements

### Requirement: An Italian word standing alone in capitals is read as the acronym its pack lists
Before Italian's lemmatisation cascade, the core SHALL read an Italian token as an acronym of its pack when the token is one word written wholly in capitals — two letters or more and no lowercase letter, read without its soft hyphens —, when neither the nearest word of two letters or more written before it nor the one written after it, in the text being read, is written wholly in capitals, and when the pack's forms list the token as written, in NFC and not lowercased; the token's dictionary form SHALL then be the acronym, capitals kept, and the cascade SHALL not run on it. A neighbour SHALL be judged by its written word, so an articulated preposition or an elided piece written in capitals beside the token is a neighbour in capitals, and one written in lowercase is not; Italian's tokenisation pre-pass SHALL run before the rule, so the pieces of a split word are read by the pre-pass and a single letter is never an acronym. Any other Italian token SHALL be read as before, lowercased. The rule SHALL bump Italian's analyser version and no other, and no part of it SHALL run on text read as English, Spanish or French, whose output SHALL NOT move.

#### Scenario: NATO in a sentence
- **WHEN** « I paesi della NATO hanno votato. » is analysed as Italian with a pack listing `NATO` as an acronym and `nato` as *nascere*'s participle
- **THEN** « NATO » reads as the acronym `NATO`, and the cascade does not run on it

#### Scenario: A headline in capitals
- **WHEN** « LA NATO HA VOTATO » is analysed as Italian with the same pack
- **THEN** « NATO » reads as `nato`, *nascere*'s participle, as the words around it are in capitals too

#### Scenario: A participle in lowercase
- **WHEN** « sono nato a Roma » and « Nato a Roma » are analysed as Italian
- **THEN** both « nato » and « Nato » read as *nascere*

#### Scenario: An articulated preposition beside the acronym
- **WHEN** « negli USA » and « NEGLI USA » are analysed as Italian with a pack listing `USA` as an acronym and `usa` as *usare*'s form
- **THEN** « negli USA » reads `in` + `gli` + the acronym `USA`, and « NEGLI USA » reads `in` + `gli` + `usa`, its neighbour being written in capitals

#### Scenario: A contraction in capitals
- **WHEN** « AI MIEI AMICI » and « Ai miei amici » are analysed as Italian
- **THEN** « AI » and « Ai » are read by the pre-pass as `a` + `i`, single letters, and no acronym is read

#### Scenario: An acronym whose lowercase is no other word's form
- **WHEN** « l'ONU e la RAI » is analysed as Italian with a pack listing `onu` and `rai` as their own lemmas and no acronym row for them
- **THEN** « ONU » and « RAI » read as `onu` and `rai`, by the cascade, as before

#### Scenario: A word card asks about the acronym
- **WHEN** a word card asks the grammar of « NATO » for the dictionary form `NATO`
- **THEN** it answers the acronym's card, apart from `nato`'s

#### Scenario: A pack built before is refused
- **WHEN** an Italian pack built for the analyser version before this rule is loaded
- **THEN** it is refused as built for another analyser generation, and a French, Spanish or English pack loads as before

#### Scenario: The other languages do not move
- **WHEN** the six invariance baselines run after this change
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
