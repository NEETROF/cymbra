## ADDED Requirements

### Requirement: A French expression built on a verb gives its preposition to a longer expression
The phrase gloss SHALL NOT keep a French match at a length where three things hold: the match is built on a verb — one of its tokens before its last is a form the pack's grammar reads, as the dictionary form the analysis gave it, as a verb or an auxiliary (*French's word grammar*, *A dictionary form is read as itself*), and no token before that one is a preposition —; its last token is a preposition of *French closed classes*, or the `à` of a written « au » or « aux », the token after the match sharing its source span; and the longest run starting on that last token that is a key and meets its headword, as *French expressions are found on French's reading of a selection*, *A French expression's pieces meet the words as its headword writes them* and *A French expression's contractions meet the words as its headword writes them* find runs, holds more tokens than the match or starts on that `à`. The match SHALL then give up its last token to the expression starting on it: the shorter runs SHALL be tried as for a run that does not meet its headword, else the next token, and the expression SHALL be found on that token as the scan finds any. A match that holds no such verb, that ends on no preposition, or after whose last token no longer run starts SHALL be kept as before, every other rule of *Expression lookup in a phrase gloss*, of *French expressions are found on French's reading of a selection*, of *A French match never ends inside a written word* and of the two requirements above still applying. A match's key, gloss and status SHALL be what they are without this rule, the tokens SHALL be what they are without the table, and a page's analysis, French's analyser version, the packs, and English and Spanish selections SHALL NOT change.

#### Scenario: A verb's expression gives the « au » to the expression starting on it
- **WHEN** « Elles sont au nombre de sept. » is glossed with a French pack holding `être à`, `nombre de` and `au nombre de`, its grammar reading « sont » as a form of the verb `être`
- **THEN** one match covers `à`, `le`, `nombre` and `de`, its key is `au nombre de`, and `sont` is in no match

#### Scenario: A preposition written apart goes to a longer expression
- **WHEN** « Il est à la fois sévère et amical. » is glossed with a French pack holding `être à` and `à la fois`, its grammar reading « est » as a form of `être`
- **THEN** one match covers `à`, `la` and `fois`, and its key is `à la fois`

#### Scenario: The giving expression is tried shorter
- **WHEN** « Elle se mit au lit. » is glossed with a French pack holding `se mettre`, `se mettre à` and `au lit`, its grammar reading « mit » as a form of `mettre`
- **THEN** one match covers `se` and `mit`, its key `se mettre`, and another covers `à`, `le` and `lit`, its key `au lit`

#### Scenario: A compound preposition keeps its à
- **WHEN** « Il resta jusqu'au début de la nuit. » is glossed with a French pack holding `jusqu'à` and `au début`
- **THEN** one match covers `jusque`, `à` and `le`, its key is `jusqu'à`, and no match is `au début`

#### Scenario: An expression no longer than the verb's does not take its preposition
- **WHEN** « Il venait de l'étranger. » is glossed with a French pack holding `venir de` and `de l'`, its grammar reading « venait » as a form of `venir`
- **THEN** one match covers `venait` and `de`, and its key is `venir de`

#### Scenario: No expression starts on the preposition
- **WHEN** « Ce livre est au voisin. » is glossed with a French pack holding `être à`, its grammar reading « est » as a form of `être`
- **THEN** one match covers `est`, `à` and `le`, and its key is `être à`

#### Scenario: What the French baseline shows
- **WHEN** the French invariance baseline is re-blessed with the phrase probes « Elles sont au nombre de sept. », « Il est à la fois sévère et amical. », « Il est au courant de tout. », « Elle se mit au lit. » and « grâce au travail du monteur »
- **THEN** over fr-en they answer `au nombre de`, `à la fois`, `au courant`, `se mettre` and `au lit`, and `grâce à`; over fr-es `être à` (fr-es holding no `au nombre de`), `à la fois`, `au courant`, `se mettre`, and `grâce à`; and every probe recorded before is byte for byte

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after the rule is added, and « Lo hizo de todos modos » is glossed with a Spanish pack holding `hacer de` and `de todos modos`
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and the Spanish selection answers `hacer de` as before
