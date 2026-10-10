## ADDED Requirements

### Requirement: A French expression's contractions meet the words as its headword writes them
The phrase gloss SHALL match a French expression on a run of a selection only where, between each two adjacent tokens of the run, the tokens are one written word exactly where the expression's headword writes its two pieces at that place as one word: two pieces the headword writes as the contraction « au » (`à` and `le`) or « aux » (`à` and `les`) SHALL meet only two tokens sharing one source span — a written « au » or « aux » that *French tokenisation pre-pass* splits —, and two pieces the headword writes apart SHALL meet only two tokens with spans of their own. A run's first token sharing its source span with the token before the run — the article of a contraction whose `à` is outside the run — and its last token sharing its source span with the token after the run — the `à` of a contraction whose article is outside the run, which *A French match never ends inside a written word* then covers — SHALL meet a piece the headword writes alone. The headword SHALL be the expression's name the pack carries, or its key where the pack carries none, read by French's pre-pass as *A French pack keys its expressions as French is read* reads it; a name holding neither the word « au » nor the word « aux » SHALL be read as writing every piece apart. A run whose tokens do not meet SHALL NOT match at that length, and the shorter runs SHALL be tried as *French expressions are found on French's reading of a selection* tries them, every other rule of that requirement, of *Expression lookup in a phrase gloss*, of *A French match never ends inside a written word* and of *A French expression's pieces meet the words as its headword writes them* still applying. The tokens SHALL be what they are without the table, and a page's analysis, French's analyser version, the packs, and English and Spanish selections SHALL NOT change.

#### Scenario: A pronoun after à is not a contraction
- **WHEN** « Il est prêt à le faire » is glossed with a French pack holding `au fait`
- **THEN** no expression is reported

#### Scenario: The contraction
- **WHEN** « Au fait, tu viens ? » is glossed with a French pack holding `au fait`
- **THEN** one match covers `À`, `le` and `fait`, and its key is `au fait`

#### Scenario: Pieces written apart do not meet a contraction
- **WHEN** « au revoir » is glossed with a French pack holding `à le revoir`, written apart, and no `au revoir`
- **THEN** no expression is reported

#### Scenario: The last à before the page's article
- **WHEN** « grâce au soleil » is glossed with a French pack holding `grâce à`
- **THEN** one match covers `grâce`, `à` and `le`, and its key is `grâce à`

#### Scenario: The first article after the page's à
- **WHEN** « aux miennes » is glossed with a French pack holding `les miennes`
- **THEN** one match covers `les` and `miennes`, and its key is `les miennes`

#### Scenario: An elision and a contraction in one headword
- **WHEN** « armés jusqu’aux dents » is glossed with a French pack holding `armé jusqu'aux dents`
- **THEN** one match covers `armés`, `jusque`, `à`, `les` and `dents`, and its key is `armé jusqu'aux dents`

#### Scenario: What the French baseline shows
- **WHEN** the French invariance baseline is re-blessed with the phrase probes « Il est prêt à le faire », « Au fait, tu viens ? », « grâce au soleil » and « aux miennes »
- **THEN** over fr-en and over fr-es « Il est prêt à le faire » answers none, « Au fait, tu viens ? » answers `au fait`, « grâce au soleil » answers `grâce à` over `grâce`, `à` and `le`, and « aux miennes » answers `les miennes` over `les` and `miennes`; and every probe recorded before is byte for byte

#### Scenario: English and Spanish do not move
- **WHEN** the English, Spanish, es-en and en-es invariance baselines run after the rule is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
