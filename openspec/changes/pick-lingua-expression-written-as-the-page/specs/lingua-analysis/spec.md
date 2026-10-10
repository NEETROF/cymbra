## ADDED Requirements

### Requirement: A French match is named by the headword written as the page
The phrase gloss SHALL, where several headwords of a French pack reach the key a run of a selection matches (*A French pack keeps every headword of a key several reach*), report as the match's name and gloss those of the headword written most like the run: each headword read by French's pre-pass as *A French pack keys its expressions as French is read* reads it, the headword whose pieces' text is the text of the run's tokens at the same places — compared in lowercase, the typographic apostrophe read as the straight one — at the most places, and between headwords written so at as many places, the first in the order the pack lists them. Only the headwords whose pieces meet the run as *A French expression's pieces meet the words as its headword writes them* and *A French expression's contractions meet the words as its headword writes them* require SHALL be weighed, and a run none of them meets SHALL NOT match at that length, the shorter runs being tried as *French expressions are found on French's reading of a selection* tries them. The reader's status SHALL be read on the name reported. A key one headword reaches SHALL be matched as before. For a French selection, these rules SHALL take the place of the sentence of *French expressions are found on French's reading of a selection* that reports the key's name, and of the reading of those two requirements on the key's name alone, every other rule of those requirements and of *Expression lookup in a phrase gloss* still applying. The tokens SHALL be what they are without the table, and a page's analysis, French's analyser version, and English and Spanish selections SHALL NOT change.

#### Scenario: The headword written as the page
- **WHEN** « Ça va ? » is glossed with a French pack whose key `ça aller` holds `ça ira`, which wins it, and `ça va`
- **THEN** one match covers `Ça` and `va`, its key is `ça va` and its gloss is `ça va`'s

#### Scenario: The headword that wins the key, written as the page
- **WHEN** « Ça ira. » is glossed with the same pack
- **THEN** one match covers `Ça` and `ira`, and its key is `ça ira`

#### Scenario: One of three headwords
- **WHEN** « Allons-y ! » is glossed with a French pack whose key `aller y` holds `allez-y`, `allons-y` and `vas-y`
- **THEN** one match covers `Allons` and `y`, and its key is `allons-y`

#### Scenario: No headword written as the page, one closer
- **WHEN** « Quelle que soit la distance » is glossed with a French pack whose key `quel que être` holds `quel que fût`, which wins it, and `quel que soit`
- **THEN** one match covers `Quelle`, `que` and `soit`, and its key is `quel que soit`

#### Scenario: No headword closer than another
- **WHEN** « ça allait » is glossed with a French pack whose key `ça aller` holds `ça ira`, which wins it, and `ça va`
- **THEN** one match covers `ça` and `allait`, and its key is `ça ira`

#### Scenario: A headword that does not meet the page
- **WHEN** « c'qui » is glossed with a French pack whose key `ce qui` holds `ce qui`, which wins it, and `c'qui`
- **THEN** one match covers the two tokens, and its key is `c'qui`

#### Scenario: A key one headword reaches
- **WHEN** « Ça ira. » is glossed with a French pack whose key `ça aller` holds `ça va` alone
- **THEN** one match covers `Ça` and `ira`, and its key is `ça va`

#### Scenario: The status follows the headword
- **WHEN** the reader has marked `ça va` known and « Ça ira. » and « Ça va ? » are glossed with a French pack whose key `ça aller` holds `ça ira` and `ça va`
- **THEN** « Ça va ? »'s match is classified known and « Ça ira. »'s is not

#### Scenario: What the French baseline shows
- **WHEN** the French invariance baseline is re-blessed with the phrase probes « Ça va ? », « Ça ira. », « Allons-y ! », « les miens » and « Quelle que soit la distance »
- **THEN** over fr-en « Ça va ? » answers `ça va`, « Ça ira. » `ça ira`, « Allons-y ! » `allons-y`, « les miens » `les miens` and « Quelle que soit la distance » `quel que soit`; over fr-es « Ça va ? » and « Ça ira. » answer `ça va`, « Allons-y ! » none, « les miens » `les miens` and « Quelle que soit la distance » `quel que soit`; and every probe recorded before is byte for byte but the pack's size

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after the rule is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
