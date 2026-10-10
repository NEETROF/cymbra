## ADDED Requirements

### Requirement: An expression built on a verb gives its last word to a longer expression
The phrase gloss SHALL NOT keep a match, in French, Spanish or English, at a length where two things hold: the match is built on a verb — one of its tokens, its last included, is a form the pack's grammar reads, as the dictionary form the analysis gave it, as a verb or an auxiliary (*French's word grammar*, *A word's grammar, from the pack*, *A dictionary form is read as itself*), and no token before that one is a preposition of the studied language's closed classes —; and the longest run starting on the match's last token that is a key and meets its headword, as *Expression lookup in a phrase gloss*, *French expressions are found on French's reading of a selection*, *Spanish expressions are found on Spanish's reading of a selection* and the requirements holding French pieces as written find runs, holds more tokens than the match, or, in French and Spanish, starts on the `à` of a written « au » or « aux », or on the `a` or `de` of a written « al » or « del », the token after the match sharing its source span. The match SHALL then give up its last token to that expression: the shorter runs SHALL be tried as for a run that does not meet its headword, else the next token, and the expression SHALL be found on that token as the scan finds any. The match SHALL keep its last token, as before, when that token is a preposition the selection's next token, `que`, follows (French and Spanish), or when the expression that would take it ends inside the run the scan would take next after the match, and that run holds a token that is not a function word. A match that is not built on a verb, or after whose last token no such expression starts, SHALL be kept as before, every other rule of the requirements above and of *A French match never ends inside a written word* still applying. A match's key, gloss and status SHALL be what they are without this rule, the tokens SHALL be what they are without the table, and a page's analysis, every analyser version and the packs SHALL NOT change.

#### Scenario: A verb's expression gives the « au » to the expression starting on it
- **WHEN** « Elles sont au nombre de sept. » is glossed with a French pack holding `être à`, `nombre de` and `au nombre de`, its grammar reading « sont » as a form of the verb `être`
- **THEN** one match covers `à`, `le`, `nombre` and `de`, its key is `au nombre de`, and `sont` is in no match

#### Scenario: A preposition written apart goes to a longer expression
- **WHEN** « Il est à la fois sévère et amical. » is glossed with a French pack holding `être à` and `à la fois`, its grammar reading « est » as a form of `être`
- **THEN** one match covers `à`, `la` and `fois`, and its key is `à la fois`

#### Scenario: The giving expression is tried shorter
- **WHEN** « Elle se mit au lit. » is glossed with a French pack holding `se mettre`, `se mettre à` and `au lit`, its grammar reading « mit » as a form of `mettre`
- **THEN** one match covers `se` and `mit`, its key `se mettre`, and another covers `à`, `le` and `lit`, its key `au lit`

#### Scenario: The verb itself goes to a longer expression
- **WHEN** « J'ai besoin de toi. » is glossed with a French pack holding `j'ai` and `avoir besoin de`, its grammar reading « ai » as a form of `avoir`
- **THEN** one match covers `ai`, `besoin` and `de`, and its key is `avoir besoin de`

#### Scenario: A compound preposition keeps its à
- **WHEN** « Il resta jusqu'au début de la nuit. » is glossed with a French pack holding `jusqu'à` and `au début`
- **THEN** one match covers `jusque`, `à` and `le`, its key is `jusqu'à`, and no match is `au début`

#### Scenario: An expression no longer than the verb's takes nothing
- **WHEN** « Il venait de l'étranger. » is glossed with a French pack holding `venir de` and `de l'`, its grammar reading « venait » as a form of `venir`
- **THEN** one match covers `venait` and `de`, and its key is `venir de`

#### Scenario: A Spanish verb's preposition
- **WHEN** « Lo hizo de todos modos. » is glossed with a Spanish pack holding `hacer de` and `de todos modos`, its grammar reading « hizo » as a form of `hacer`
- **THEN** one match covers `de`, `todos` and `modos`, and its key is `de todos modos`

#### Scenario: A preposition followed by que stays
- **WHEN** « Pese a que no tengo dinero » is glossed with a Spanish pack holding `pese a` and `a que no`, its grammar reading « pese » as a form of `pesar`
- **THEN** one match covers `Pese` and `a`, and its key is `pese a`

#### Scenario: The expression next is not cut
- **WHEN** « Ella se iba al día siguiente. » is glossed with a Spanish pack holding `ir a`, `al día` and `día siguiente`, its grammar reading « iba » as a form of `ir`
- **THEN** one match covers `iba`, `a` and `el`, its key `ir a`, and another covers `día` and `siguiente`, its key `día siguiente`

#### Scenario: An English verb's particle
- **WHEN** « The whole thing went over the top. » is glossed with an English pack holding `go over` and `over the top`, its grammar reading « went » as a form of `go`
- **THEN** one match covers `over`, `the` and `top`, and its key is `over the top`

#### Scenario: What the invariance baselines show
- **WHEN** the French, Spanish, es-en, English and en-es invariance baselines are re-blessed with the phrase probes this change adds
- **THEN** over fr-en « Elles sont au nombre de sept. » answers `au nombre de`, « Il est à la fois sévère et amical. » `à la fois`, « Il est au courant de tout. » `au courant`, « Elle se mit au lit. » `se mettre` and `au lit`, « grâce au travail du monteur » `grâce à`, « J'ai besoin de toi. » `avoir besoin de`, « C'est ce que tu veux. » `c'est` and `ce que`, « À l'est, il fait froid. » `faire froid` alone; over es-fr « Lo hizo de todos modos. » answers `de todos modos`, « Van al final del álbum. » `al final de`, « Pese a que no tengo dinero, compré el libro. » `pese a`, « Ella se iba al día siguiente. » `ir a` and `día siguiente`; over en-fr and en-es « The whole thing went over the top. » answers `over the top` and « He was standing in front of the house. » `in front of`; and every probe recorded before is byte for byte

### Requirement: A French expression's hyphens meet the words as its headword writes them
The phrase gloss SHALL match a French expression on a run of a selection only where, at each place where the expression's headword joins two pieces with a hyphen — read as French's pre-pass reads the headword, a hyphen with the euphonic `t` between (`a-t-il`) counted as one —, the selection's two tokens are joined by a hyphen too, read without soft hyphens as *A soft hyphen is not part of a word* reads one. The headword SHALL be the expression's name the pack carries, or its key where it carries none; a name holding no hyphen SHALL constrain nothing. A run whose tokens do not meet SHALL NOT match at that length, and the shorter runs SHALL be tried as *French expressions are found on French's reading of a selection* tries them, every other rule of that requirement and of *A French expression's pieces meet the words as its headword writes them* and *A French expression's contractions meet the words as its headword writes them* still applying. English and Spanish selections, the tokens, a page's analysis, French's analyser version and the packs SHALL NOT change.

#### Scenario: The question marker needs its hyphen
- **WHEN** « C'est ce que tu veux. » is glossed with a French pack holding `c'est`, `ce que` and `est-ce que`
- **THEN** one match covers `C'` and `est`, its key `c'est`, another covers `ce` and `que`, its key `ce que`, and no match is `est-ce que`

#### Scenario: Written with its hyphen
- **WHEN** « Est-ce que tu viens ? » is glossed with the same pack
- **THEN** one match covers `Est`, `ce` and `que`, and its key is `est-ce que`

#### Scenario: Across a comma
- **WHEN** « À l'est, il fait froid. » is glossed with a French pack holding `est-il`
- **THEN** no match is `est-il`
