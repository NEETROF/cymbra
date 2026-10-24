# Spec Delta

## ADDED Requirements

### Requirement: Italian expressions are found on Italian's reading of a selection
The phrase gloss SHALL find an Italian pack's expressions on the selection's tokens written as *An Italian pack keys its expressions as Italian is read* writes a key — each token's dictionary form, the determiners that requirement lists written as the pre-pass gives them — over runs of two to seven tokens, the longest run first, a token belonging to at most one match. A match whose last token is followed by a token sharing its source span — the article of an articulated preposition the pre-pass split — SHALL cover that token too, so that no match ends inside a written word. An Italian run no key matches whose last piece is `del`, `dello`, `della`, `dei`, `degli` or `delle` SHALL be tried once more with that piece read as `di` where that `di` closes a governing frame — the key's first piece a preposition, the preposition of an articulated preposition included, or the run built on a verb as *An expression built on a verb gives its last word to a longer expression* reads one, or holding before any of its prepositions an auxiliary, `avere` or `essere`, that the pack's grammar gives no reading —, and a match SHALL cover it; a run holding no such frame, a noun phrase, SHALL NOT be tried so. A run SHALL match only where each piece of the expression's headword meets the run's token at its place as the headword writes it: a token, and a headword's piece, being elided when its source span ends on an apostrophe — an elided form of *Italian tokenisation pre-pass*, `dell'` read as `dello`, both halves of `all'`, `coll'`, `dall'`, `nell'` and `sull'`, a truncation (`po'`, a `da'` read as the verb, the article of a truncated articulated preposition) —, except a word whose apostrophe stands for its accent (`perche'` read as `perché`) and the preposition of a truncated articulated preposition (`in` of `ne'`), both written in full, and a truncation written with an accent for its apostrophe (`pò` read as `poco`) being a truncation; a truncation standing for the word the pre-pass or the pack reads it as (`po'` for `poco`, `fa'` for `fai`, `de'` for `dei`), and the article of a truncated articulated preposition for the articulated preposition (`ne'` for `nei`); a piece the headword writes elided SHALL meet only an elided token, an elision only an elision and a truncation only a truncation, and a piece written in full only a token written in full or a truncation standing for another word than the piece (`fare` « fa' », `di` « de' », the article of `nei` the article of « ne' »), never a truncation standing for the piece itself (`poco` « po' » or « pò »), the headword's last piece written in full meeting an elision too; two pieces the headword writes as one articulated preposition SHALL meet only two tokens sharing one source span, and two it writes apart only tokens with spans of their own, the run's first token sharing its span with the token before the run and its last token sharing its span with the token after it meeting a piece the headword writes alone; the headword SHALL be the expression's name the pack carries, or its key, read by Italian's pre-pass, and a name holding neither an apostrophe nor an articulated preposition SHALL be read as written in full and apart. Where several headwords of the pack reach the run's key, the match SHALL be named by the headword, among those whose pieces meet the run, written as the run's tokens at the most places, compared in lowercase with the typographic apostrophe read as the straight one, and between equals by the first in the pack's order; a run no headword meets SHALL NOT match at that length, the shorter runs being tried. A match SHALL report as its key the expression's name, and the reader's status of the expression SHALL be read on that name. The rule of *An expression built on a verb gives its last word to a longer expression* SHALL read Italian as it reads Spanish, its guard on the selection's next token « que » reading « che », once the pack's grammar says what a verb is. The tokens SHALL be what they are without the table, a page's analysis SHALL NOT change, and Italian's analyser version SHALL NOT move for these rules. For an Italian selection, these rules SHALL take the place of the matching rule of *Expression lookup in a phrase gloss* and of the sentences of *French expressions are found on French's reading of a selection*, *Spanish expressions are found on Spanish's reading of a selection*, *A French match never ends inside a written word*, *A French expression's pieces meet the words as its headword writes them*, *A French expression's contractions meet the words as its headword writes them*, *A French match is named by the headword written as the page* and *A Spanish match is named by the headword written as the page* that keep every other language's selections as before, every other rule of those requirements still applying. English, Spanish and French selections SHALL be matched as before.

#### Scenario: An articulated preposition read as two words
- **WHEN** « cotto al dente » is glossed as Italian with a pack keying `al dente` as `a il dente`
- **THEN** the match covers `a`, `il` and `dente`, is named `al dente`, and its status is read on `al dente`

#### Scenario: The article of a contraction is covered
- **WHEN** « vicino al mare » is glossed as Italian with a pack holding `vicino a`
- **THEN** the match covers `vicino`, `a` and the `il` sharing « al »'s span, so a selection of the written words is answered whole

#### Scenario: The di family read as di
- **WHEN** « a causa del ritardo » is glossed as Italian with a pack holding `a causa di` and no key `a causa del`
- **THEN** the match is `a causa di`, covering « del »

#### Scenario: The di family read as di only in a frame
- **WHEN** « ho bisogno del tuo aiuto » and « la bellezza della natura » are glossed as Italian with a pack holding `avere bisogno di` and `la bellezza di`, and no grammar
- **THEN** the first matches `avere bisogno di`, its run built on the auxiliary `avere`, and the second matches nothing: a noun phrase is no frame, the `di` being the next noun's

#### Scenario: An elided piece is not a word in full
- **WHEN** « anch'io » and « anche io » are glossed as Italian with a pack holding `anch'io`, keyed `anche io`
- **THEN** « anch'io » matches and « anche io » does not, `anche` being written in full where the headword writes `anch'`

#### Scenario: A word in full is not an elided piece
- **WHEN** « d'un tratto » and « di un tratto » are glossed as Italian with a pack holding `d'un tratto`
- **THEN** « d'un tratto » matches and « di un tratto » does not

#### Scenario: An accent typed as an apostrophe
- **WHEN** « piu' o meno » is glossed as Italian with a pack holding `più o meno`
- **THEN** it matches, `piu'` being the word `più` written in full, its accent typed as an apostrophe

#### Scenario: A contraction is not two words apart
- **WHEN** « andare a il mare », written apart, and « andare al mare » are glossed as Italian with a pack holding `andare al mare`
- **THEN** « andare al mare » matches and the words apart do not

#### Scenario: The headword written as the page
- **WHEN** « vale la pena » and « valeva la pena » are glossed as Italian with a pack whose key `valere la pena` is reached by `valere la pena` and `vale la pena`
- **THEN** the first match is named `vale la pena` (written as the page at three places) and the second `valere la pena`, the key's winner

#### Scenario: A truncation
- **WHEN** « un po' di pane » and « un poco di pane » are glossed as Italian with a pack holding `un po'` and `un poco`, both keyed `un poco`
- **THEN** « un po' » is named `un po'` and « un poco » `un poco`: the truncation reads as an elided piece by its span, and `un poco`'s last piece in full never meets a truncation

#### Scenario: A truncated imperative
- **WHEN** « fa' attenzione », « va' via », « sta' zitto », « di' la verità » and « fa' presto » are glossed as Italian with a pack holding `fare attenzione`, `andare via`, `stare zitto`, `dire la verità` and `fare presto`
- **THEN** each matches its expression: the truncation stands for `fai`, `vai`, `stai`, `dici`, another word than the headword's piece written in full

#### Scenario: A truncated articulated preposition
- **WHEN** « vicino a' piedi » and « ne' pressi di casa » are glossed as Italian with a pack holding `vicino a` and `nei pressi di`
- **THEN** « vicino a' » matches `vicino a`, its preposition written in full and its article covered, and « ne' pressi di » matches `nei pressi di`, the article of « ne' » standing for `nei`

#### Scenario: A truncation written with an accent
- **WHEN** « un pò di pane » is glossed as Italian with a pack holding `un po'` and `un poco`, and listing `pò` as a form of `poco`
- **THEN** « un pò » is named `un po'`, `pò` being the truncation `po'` written with an accent for its apostrophe

#### Scenario: The overlap rule
- **WHEN** « ha bisogno di tempo » is glossed as Italian with a pack whose grammar reads `avere` as a verb and which holds `avere bisogno` and `bisogno di`
- **THEN** the rule of *An expression built on a verb gives its last word to a longer expression* decides as it does for Spanish

#### Scenario: The other languages do not move
- **WHEN** the six invariance baselines run after this change
- **THEN** every probe is byte for byte the output recorded before, without re-blessing
