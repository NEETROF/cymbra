# Spec Delta

## ADDED Requirements

### Requirement: An Italian pack keys its expressions as Italian is read
A pack studying Italian SHALL key each expression by what the core's Italian analysis reads in its headword — the headword tokenised by Italian's own pre-pass, its elided pieces read as the words they stand for, its articulated prepositions of the `a`, `da`, `in`, `su` and `con` families read as preposition + article and those of the `di` family kept whole, and each token resolved as a page's token is — writing each token as its dictionary form, except `il`, `lo`, `la`, `i`, `gli`, `le`, `un`, `uno`, `una`, `mio`, `mia`, `miei`, `mie`, `tuo`, `tua`, `tuoi`, `tue`, `suo`, `sua`, `suoi`, `sue`, `nostro`, `nostra`, `nostri`, `nostre`, `vostro`, `vostra`, `vostri`, `vostre`, `loro`, `questo`, `questa`, `questi`, `queste`, `quello`, `quella`, `quelli`, `quelle`, `quel`, `quei` and `quegli`, each written in lowercase as the pre-pass gives it, the tokens joined by single spaces. The key SHALL be built by the same code that reads a page, the one that keys a French or a Spanish pack, at build time. An entry SHALL be kept only when its headword reads as at least two and at most seven tokens, the dictionary form of every token is a lemma of the pack's lexicon, and every word of the headword gives at least one token — a word without a space that the pre-pass splits being an expression, a word read as one token never. Each kept expression SHALL be named by its headword as the dictionary writes it, and the pack SHALL carry that name, in the optional section a French pack carries its names in, wherever it differs from the key. Where several headwords reach one key, the headword written as the key SHALL keep it, then the one with the most tokens written as their key piece, then the first in byte order; every other headword of the key SHALL be kept as an expression of its own, in the optional section a French pack lists them in. For a pack studying Italian, these rules SHALL take the place of the key rule of *Multi-word expression table* (the dictionary forms of its words, by the lemmatiser), of the sentences of *A French pack keys its expressions as French is read* and *A Spanish pack keys its expressions as Spanish is read* that keep every other pack keyed as before with no names, and of the sentences of *A French pack keeps every headword of a key several reach* and *A Spanish pack keeps every headword of a key several reach* that keep every other pack without the section, every other rule of those requirements still applying. A pack studying English, Spanish or French SHALL be keyed as before, byte for byte, and an Italian pack's tables and `pack_version` SHALL NOT move for these rules.

#### Scenario: An articulated preposition
- **WHEN** an Italian pack is built with the expressions `al dente` and `alla romana` and a lexicon listing `a`, `il`, `la`, `dente` and `romano`
- **THEN** `al dente` is keyed `a il dente` and `alla romana` `a la romano`, the two names carried, and neither answers the other's article

#### Scenario: An elision
- **WHEN** an Italian pack is built with the expression `fare l'amore` and a lexicon listing `fare`, `lo` and `amore`
- **THEN** it is keyed `fare lo amore` and named `fare l'amore`

#### Scenario: The di family stays whole
- **WHEN** an Italian pack is built with the expression `amico del cuore` and a lexicon listing `amico`, `del` and `cuore`
- **THEN** it is keyed `amico del cuore`, which is its own headword, so no name is carried

#### Scenario: A word the pre-pass splits
- **WHEN** an Italian pack is built with the headwords `d'accordo` and `perché`
- **THEN** `d'accordo` is an expression keyed `di accordo`, and `perché`, one token, is not

#### Scenario: Seven tokens
- **WHEN** an Italian pack is built with the expressions `al fin fine` (four tokens) and a proverb of nine tokens
- **THEN** the first is keyed and the proverb left out

#### Scenario: Two headwords on one key
- **WHEN** an Italian pack is built with `quel che sia` and `quello che sia` reaching one key
- **THEN** the headword written as the key keeps it, the other is kept as an expression of its own in the section listing it, and both are named

#### Scenario: Nothing else moves
- **WHEN** the six committed packs are built after this change
- **THEN** each builds to the sha256 its pin records
