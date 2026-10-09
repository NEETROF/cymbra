## ADDED Requirements

### Requirement: French expressions are found on French's reading of a selection
The phrase gloss SHALL find a French pack's expressions on the selection's tokens written as *A French pack keys its expressions as French is read* writes a key — each token's dictionary form, the determiners and pronouns that requirement lists written as the pre-pass gives them — over runs of two to seven tokens, the longest run first, a token belonging to at most one match. A run that no key matches and whose last token reads `du` or `des` SHALL be tried once more with that token read as `de`, the match then covering it. A match SHALL report as its key the expression's name — its headword as the dictionary writes it, or the key itself when the pack carries no name for it — and the reader's status of the expression SHALL be read on that name. The tokens SHALL be what they are without the table, and a page's analysis SHALL not change. English and Spanish selections SHALL be matched as before, over runs of up to five tokens, on their lemmas.

#### Scenario: A contracted article
- **WHEN** « Au revoir » is glossed with a French pack holding `au revoir`
- **THEN** one match covers the three tokens `À`, `le` and `revoir`, and its key is `au revoir`

#### Scenario: An elided word
- **WHEN** « un coup d’œil » is glossed with a French pack holding `coup d'œil`
- **THEN** one match covers `coup`, `de` and `œil`, and its key is `coup d'œil`

#### Scenario: A word the pre-pass splits
- **WHEN** « D’abord » is glossed with a French pack holding `d'abord`
- **THEN** one match covers its two tokens, and its key is `d'abord`

#### Scenario: Another tense of the expression
- **WHEN** « il y avait » is glossed with a French pack holding `il y a`
- **THEN** one match covers the three tokens, and its key is `il y a`

#### Scenario: A contracted article is not the feminine one
- **WHEN** « au marché » is glossed with a French pack holding `à la`
- **THEN** no expression is reported

#### Scenario: An expression ending on de before a contracted article
- **WHEN** « à cause des » is glossed with a French pack holding `à cause de`
- **THEN** one match covers the three tokens, `des` included, and its key is `à cause de`

#### Scenario: An expression of six tokens
- **WHEN** « au fur et à mesure » is glossed with a French pack holding it
- **THEN** one match covers the six tokens `à`, `le`, `fur`, `et`, `à` and `mesure`

#### Scenario: The status follows the name
- **WHEN** the reader has marked `il y a` known and « il y avait » is glossed with a French pack holding `il y a`
- **THEN** the match is classified known

#### Scenario: What the French baseline shows
- **WHEN** the French invariance baseline is re-blessed for these keys
- **THEN** « Au revoir », « un coup d’œil », « D’abord », « au fur et à mesure », « à cause des » and « à la maison » each answer an expression, « il y a » and « il y avait » answer `il y a`, « au marché » and « jusqu'au soir » answer none, and no `analyse` probe moves

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after French's expression rules are added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
