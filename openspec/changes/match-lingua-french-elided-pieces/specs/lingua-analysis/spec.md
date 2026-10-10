## ADDED Requirements

### Requirement: A French expression's pieces meet the words as its headword writes them
The phrase gloss SHALL match a French expression on a run of a selection only where each piece of the expression's headword meets the run's token at its place as the headword writes it: a piece the headword writes elided — `c'`, `ç'`, `d'`, `j'`, `l'`, `m'`, `n'`, `qu'`, `s'`, `t'`, `jusqu'`, `lorsqu'`, `puisqu'` or `quoiqu'` — SHALL meet only an elided token, and a piece the headword writes in full SHALL meet only a token written in full, except the headword's last piece written in full, which SHALL meet either, French eliding it or not by the word that follows on the page. A token SHALL be elided when *French tokenisation pre-pass* split it off as an elided word, its source span ending on its apostrophe. The headword SHALL be the expression's name the pack carries, or its key where the pack carries none, read by French's pre-pass as *A French pack keys its expressions as French is read* reads it; a name holding no apostrophe SHALL be read as written in full. A run whose pieces do not meet SHALL NOT match at that length, and the shorter runs SHALL be tried as *French expressions are found on French's reading of a selection* tries them, every other rule of that requirement, of *Expression lookup in a phrase gloss* and of *A French match never ends inside a written word* still applying. The tokens SHALL be what they are without the table, and a page's analysis, French's analyser version, the packs, and English and Spanish selections SHALL NOT change.

#### Scenario: An elided article is not a pronoun
- **WHEN** « Il a décidé de le faire » is glossed with a French pack holding `de l'`
- **THEN** no expression is reported

#### Scenario: The elided article
- **WHEN** « de l’eau » is glossed with a French pack holding `de l'`
- **THEN** one match covers `de` and `le`, and its key is `de l'`

#### Scenario: A word written in full is not its elision
- **WHEN** « d’un hiver » is glossed with a French pack holding `de un`
- **THEN** no expression is reported

#### Scenario: A shorter run takes the place
- **WHEN** « d’un peu plus » is glossed with a French pack holding `de un` and `un peu`
- **THEN** one match covers `un` and `peu`, and its key is `un peu`

#### Scenario: The last piece follows the page
- **WHEN** « parce qu’il pleut » is glossed with a French pack holding `parce que`
- **THEN** one match covers `parce` and `que`, and its key is `parce que`

#### Scenario: An elided piece inside a question
- **WHEN** « Qu’est-ce qu’il attend » is glossed with a French pack holding `qu'est-ce que`
- **THEN** one match covers `Que`, `est`, `ce` and `que`, and its key is `qu'est-ce que`

#### Scenario: Another form written in full
- **WHEN** « Ce sont mes amis » and « C’était l’hiver » are glossed with a French pack holding `c'est`
- **THEN** the first reports no expression, and the second one match, `c'est`, covering `Ce` and `était`

#### Scenario: What the French baseline shows
- **WHEN** the French invariance baseline is re-blessed with the phrase probes « de l’eau », « Il a décidé de le faire », « parce qu’il pleut » and « d’un hiver »
- **THEN** over fr-en « de l’eau » answers `de l'`, « parce qu’il pleut » answers `parce que`, « Il a décidé de le faire » and « d’un hiver » answer none, and every probe recorded before is byte for byte

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after the rule is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
