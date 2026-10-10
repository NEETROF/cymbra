## ADDED Requirements

### Requirement: A French match never ends inside a written word
The phrase gloss SHALL extend a French expression match whose last token is followed by a token sharing its source span — the article `le` or `les` of a written « au » or « aux », read as `à` + `le`/`les` — over that token too, as *French expressions are found on French's reading of a selection* finds the match, so that no French match ends inside a written word and a selection of the written words is answered whole. The match's key, gloss and status SHALL be what they are without the extension, the tokens SHALL be what they are without the table, and a page's analysis, French's analyser version, the packs and English selections SHALL NOT change.

#### Scenario: An expression ending on the à of au
- **WHEN** « jusqu'au soir » is glossed with a French pack holding `jusqu'à`
- **THEN** one match covers `jusque`, `à` and `le`, and `soir` is outside it

#### Scenario: The written words selected
- **WHEN** « jusqu'au » is glossed with the same pack
- **THEN** one match, `jusqu'à`, covers the whole selection, its three tokens

#### Scenario: English split words as before
- **WHEN** « I don't » is glossed with an English pack holding `i do`
- **THEN** the match covers `I` and `do` only
