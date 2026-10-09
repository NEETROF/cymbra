## ADDED Requirements

### Requirement: Spanish expressions are found on Spanish's reading of a selection
The phrase gloss SHALL find a Spanish pack's expressions on the selection's tokens written as *A Spanish pack keys its expressions as Spanish is read* writes a key — each token's dictionary form, the determiners that requirement lists written as the pre-pass gives them — over runs of two to seven tokens, the longest run first, a token belonging to at most one match. A match whose last token is followed by a token sharing its source span — the article of `al` or `del` — SHALL cover that token too, so that no match ends inside a written word. A match SHALL report as its key the expression's name — its headword as the dictionary writes it, or the key itself when the pack carries no name for it — and the reader's status of the expression SHALL be read on that name, unless *A Spanish expression settled under its lemmas keeps that key* applies. The tokens SHALL be what they are without the table, a page's analysis SHALL not change, and Spanish's analyser version SHALL not move for these rules. For a Spanish selection, these rules SHALL take the place of the matching rule of *Expression lookup in a phrase gloss* (runs of dictionary forms) and of the sentence of *French expressions are found on French's reading of a selection* that matches Spanish selections as before, every other rule of those requirements still applying. English selections SHALL be matched as before, over runs of up to five tokens, on their lemmas.

#### Scenario: A contraction read as two words
- **WHEN** « al menos » is glossed with a Spanish pack holding `al menos`
- **THEN** one match covers the three tokens `a`, `el` and `menos`, and its key is `al menos`

#### Scenario: The name, not the lemmas
- **WHEN** « tener en cuenta » is glossed with a Spanish pack holding it
- **THEN** one match covers the three tokens, and its key is `tener en cuenta`

#### Scenario: An article entry does not answer another article
- **WHEN** « del barrio » is glossed with a Spanish pack holding `de las` and `de los` and no other expression it holds
- **THEN** no expression is reported

#### Scenario: The article of a contraction is covered
- **WHEN** « después del » is glossed with a Spanish pack holding `después de`
- **THEN** one match covers the three tokens `después`, `de` and `el`, and its key is `después de`

#### Scenario: An expression of seven tokens
- **WHEN** « al fin y al cabo » is glossed with a Spanish pack holding it
- **THEN** one match covers its seven tokens, and its key is `al fin y al cabo`

#### Scenario: The status follows the name
- **WHEN** the reader has marked `tener en cuenta` known and « tener en cuenta » is glossed with a Spanish pack holding it
- **THEN** the match is classified known

#### Scenario: What the Spanish baselines show
- **WHEN** the es-fr and es-en invariance baselines are re-blessed for these keys
- **THEN** « tener en cuenta » answers `tener en cuenta`, « al aire libre » answers `al aire libre` over its four tokens, « a la casa » answers `a la`, « del barrio » answers none, « al menos », « a la vez », « después del » and « al fin y al cabo » each answer one expression covering the whole selection, and no `analyse`, `gloss` or `word-grammar` probe moves

#### Scenario: English and French do not move
- **WHEN** the English, en-es and French invariance baselines run after Spanish's expression rules are added
- **THEN** every probe is byte for byte the output recorded before, but the French baseline's line describing the es-en pack beside it

### Requirement: A Spanish expression settled under its lemmas keeps that key
When the reader holds a status, or a withdrawn status, on the dictionary forms of a Spanish match's tokens joined by single spaces — taken before the article of a contraction is covered, which is the key the phrase gloss reported for those tokens before Spanish expressions were named — and none on the expression's name, the match SHALL report that lemma chain as its key and read the reader's status there. The rule SHALL apply to Spanish matches alone, and no status, card or sync record SHALL be rewritten by it.

#### Scenario: A status set before
- **WHEN** the reader marked `tener en contar` known before this rule and « tener en cuenta » is glossed with a Spanish pack holding it
- **THEN** the match's key is `tener en contar`, classified known

#### Scenario: A deck card made before
- **WHEN** the reader added `a el vez` to the deck before this rule and « a la vez » is glossed with a Spanish pack holding it
- **THEN** the match's key is `a el vez`, classified learning, so the card it opens acts on the reader's card `a el vez`

#### Scenario: Nothing settled
- **WHEN** the reader holds no record on `tener en contar` and « tener en cuenta » is glossed
- **THEN** the match's key is `tener en cuenta`

#### Scenario: A record on the name
- **WHEN** the reader holds records on both `tener en contar` and `tener en cuenta`
- **THEN** the match's key is `tener en cuenta`, its status read there
