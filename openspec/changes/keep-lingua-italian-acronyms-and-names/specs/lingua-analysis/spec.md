# Spec Delta

## ADDED Requirements

### Requirement: An Italian word standing alone in capitals is read as the acronym its pack lists
Before Italian's lemmatisation cascade, the core SHALL read an Italian token as an acronym of its pack when the token is one word written wholly in capitals — two letters or more and no lowercase letter, read without its soft hyphens —, when neither the nearest word of two letters or more written before it nor the one written after it, in the text being read, is written wholly in capitals, and when the pack's forms list the token as written, in NFC and not lowercased; the token's dictionary form SHALL then be the acronym, capitals kept, and the cascade SHALL not run. Italian's tokenisation pre-pass SHALL run before it, so an articulated preposition or an elided form written in capitals is read by the pre-pass, never as an acronym. Any other token SHALL be read as before, lowercased. English's, Spanish's and French's analyses SHALL NOT move.

#### Scenario: NATO in a sentence
- **WHEN** « I paesi della NATO hanno votato. » is analysed as Italian with a pack listing `NATO` as an acronym and `nato` as *nascere*'s participle
- **THEN** « NATO » reads as the acronym `NATO`, and the cascade does not run on it

#### Scenario: A headline in capitals
- **WHEN** « LA NATO HA VOTATO » is analysed as Italian with the same pack
- **THEN** « NATO » reads as `nato`, *nascere*'s participle, as the words around it are in capitals too

#### Scenario: A participle in lowercase
- **WHEN** « sono nato a Roma » and « Nato a Roma » are analysed as Italian
- **THEN** both « nato » and « Nato » read as *nascere*

#### Scenario: A contraction in capitals
- **WHEN** « AI MIEI AMICI » and « Ai miei amici » are analysed as Italian
- **THEN** « AI » and « Ai » are read by the pre-pass as `a` + `i`, and the acronym rule never sees them

#### Scenario: An acronym whose lowercase is no word
- **WHEN** « l'ONU e la RAI » is analysed as Italian with a pack listing neither `onu` nor `rai`
- **THEN** « ONU » and « RAI » read as unknown words, as before

#### Scenario: A word card asks about the acronym
- **WHEN** a word card asks the grammar of « NATO » for the dictionary form `NATO`
- **THEN** it answers the acronym's card, apart from `nato`'s

#### Scenario: The other languages do not move
- **WHEN** the six invariance baselines run after this change
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
