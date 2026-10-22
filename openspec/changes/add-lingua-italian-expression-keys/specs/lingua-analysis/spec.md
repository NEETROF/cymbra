# Spec Delta

## ADDED Requirements

### Requirement: Italian expressions are found on Italian's reading of a selection
The phrase gloss SHALL find an Italian pack's expressions on the selection's tokens written as *An Italian pack keys its expressions as Italian is read* writes a key — each token's dictionary form, the determiners that requirement lists written as the pre-pass gives them — over runs of two to seven tokens, the longest run first, a token belonging to at most one match. A match whose last token is followed by a token sharing its source span — the article of an articulated preposition the pre-pass split — SHALL cover that token too, so that no match ends inside a written word. An Italian run no key matches whose last piece is `del`, `dello`, `della`, `dei`, `degli` or `delle` SHALL be tried once more with that piece read as `di`, and a match SHALL cover it. A run SHALL match only where each piece of the expression's headword meets the run's token at its place as the headword writes it: a piece the headword writes elided — one of the elided forms of *Italian tokenisation pre-pass*, a truncation read as one by its span — SHALL meet only an elided token and a piece written in full only a token written in full, except the headword's last piece written in full, which SHALL meet either; two pieces the headword writes as one articulated preposition SHALL meet only two tokens sharing one source span, and two it writes apart only tokens with spans of their own, the run's first token sharing its span with the token before the run and its last token sharing its span with the token after it meeting a piece the headword writes alone; the headword SHALL be the expression's name the pack carries, or its key, read by Italian's pre-pass, and a name holding neither an apostrophe nor an articulated preposition SHALL be read as written in full and apart. Where several headwords of the pack reach the run's key, the match SHALL be named by the headword, among those whose pieces meet the run, written as the run's tokens at the most places, compared in lowercase with the typographic apostrophe read as the straight one, and between equals by the first in the pack's order; a run no headword meets SHALL NOT match at that length, the shorter runs being tried. A match SHALL report as its key the expression's name, and the reader's status of the expression SHALL be read on that name. The rule of *An expression built on a verb gives its last word to a longer expression* SHALL read Italian as it reads Spanish, its guard on the selection's next token « que » reading « che », once the pack's grammar says what a verb is. The tokens SHALL be what they are without the table, a page's analysis SHALL NOT change, and Italian's analyser version SHALL NOT move for these rules. For an Italian selection, these rules SHALL take the place of the matching rule of *Expression lookup in a phrase gloss* and of the sentences of *French expressions are found on French's reading of a selection*, *Spanish expressions are found on Spanish's reading of a selection*, *A French match never ends inside a written word*, *A French expression's pieces meet the words as its headword writes them*, *A French expression's contractions meet the words as its headword writes them*, *A French match is named by the headword written as the page* and *A Spanish match is named by the headword written as the page* that keep every other language's selections as before, every other rule of those requirements still applying. English, Spanish and French selections SHALL be matched as before.

#### Scenario: An articulated preposition read as two words
- **WHEN** « cotto al dente » is glossed as Italian with a pack keying `al dente` as `a il dente`
- **THEN** the match covers `a`, `il` and `dente`, is named `al dente`, and its status is read on `al dente`

#### Scenario: The article of a contraction is covered
- **WHEN** « vicino al mare » is glossed as Italian with a pack holding `vicino a`
- **THEN** the match covers `vicino`, `a` and the `il` sharing « al »'s span, so a selection of the written words is answered whole

#### Scenario: The di family read as di
- **WHEN** « a causa del ritardo » is glossed as Italian with a pack holding `a causa di` and no key `a causa del`
- **THEN** the match is `a causa di`, covering « del »

#### Scenario: An elided piece is not a word in full
- **WHEN** « fare l'amore » and « fare lo stesso » are glossed as Italian with a pack holding `fare l'amore`
- **THEN** the first matches and the second does not, `lo` being written in full where the headword writes `l'`

#### Scenario: A word in full is not an elided piece
- **WHEN** « d'un tratto » and « di un tratto » are glossed as Italian with a pack holding `d'un tratto`
- **THEN** « d'un tratto » matches and « di un tratto » does not

#### Scenario: A contraction is not two words apart
- **WHEN** « andare a il mare », written apart, and « andare al mare » are glossed as Italian with a pack holding `andare al mare`
- **THEN** « andare al mare » matches and the words apart do not

#### Scenario: The headword written as the page
- **WHEN** « quello che sia » is glossed as Italian with a pack whose key is reached by `quel che sia` and `quello che sia`
- **THEN** the match is named `quello che sia`

#### Scenario: A truncation
- **WHEN** « un po' di pane » and « un poco di pane » are glossed as Italian with a pack holding `un po'`
- **THEN** « un po' » matches `un po'` and « un poco » does not, the truncation reading as an elided piece by its span

#### Scenario: The overlap rule
- **WHEN** « ha bisogno di tempo » is glossed as Italian with a pack whose grammar reads `avere` as a verb and which holds `avere bisogno` and `bisogno di`
- **THEN** the rule of *An expression built on a verb gives its last word to a longer expression* decides as it does for Spanish

#### Scenario: The other languages do not move
- **WHEN** the six invariance baselines run after this change
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
