## ADDED Requirements

### Requirement: A Spanish pack keys its expressions as Spanish is read
A pack studying Spanish SHALL key each expression by what the core's Spanish analysis reads in its headword — the headword tokenised by Spanish's own pre-pass, `al` and `del` read as `a` + `el` and `de` + `el`, and each token resolved as a page's token is — writing each token as its dictionary form, except `el`, `la`, `los`, `las`, `lo`, `un`, `una`, `unos`, `unas`, `mi`, `mis`, `tu`, `tus`, `su`, `sus`, `nuestro`, `nuestra`, `nuestros`, `nuestras`, `vuestro`, `vuestra`, `vuestros`, `vuestras`, `este`, `esta`, `estos`, `estas`, `ese`, `esa`, `esos`, `esas`, `aquel`, `aquella`, `aquellos` and `aquellas`, each written in lowercase as the pre-pass gives it, the tokens joined by single spaces. The key SHALL be built by the same code that reads a page, the one that keys a French pack, at build time. An entry SHALL be kept only when its headword reads as at least two and at most seven tokens, the dictionary form of every token is a lemma of the pack's lexicon, and every word of the headword gives at least one token. Each kept expression SHALL be named by its headword as the dictionary writes it, and the pack SHALL carry that name, in the optional section a French pack carries its names in, wherever it differs from the key. Where several headwords reach one key, the headword written as the key SHALL keep it, then the one with the most tokens written as their key piece, then the first in byte order. For a pack studying Spanish, these rules SHALL take the place of the key rule of *Multi-word expression table* (the dictionary forms of its words, by the lemmatiser) and of the sentence of *A French pack keys its expressions as French is read* that keeps a Spanish pack keyed as before with no names, every other rule of those requirements still applying. A pack studying English SHALL be keyed as before and carry no names, byte for byte, and a Spanish pack's tables and `pack_version` SHALL NOT move for these rules.

#### Scenario: A contraction
- **WHEN** a Spanish pack is built from a source holding the expression `al menos`
- **THEN** it is keyed `a el menos` and named `al menos`

#### Scenario: An inflected word
- **WHEN** a Spanish pack is built from a source holding `tener en cuenta`
- **THEN** it is keyed `tener en contar` and named `tener en cuenta`

#### Scenario: Determiners keep their written form
- **WHEN** a Spanish pack is built from a source holding `a la vez`, `a las armas` and `al arma`
- **THEN** `a la vez` is keyed `a la vez` and carries no name, and `a las armas` and `al arma` are two expressions with keys of their own

#### Scenario: Seven tokens
- **WHEN** a Spanish pack is built from a source holding `al fin y al cabo`
- **THEN** it is keyed `a el fin y a el cabo`, seven tokens, and kept

#### Scenario: A key longer than the window
- **WHEN** a Spanish headword reads as more than seven tokens
- **THEN** the pack leaves it out

#### Scenario: Two headwords reach one key
- **WHEN** a Spanish source holds both `soy inglés` and `soy inglesa`, `inglesa` being a form the forms table files under `inglés`
- **THEN** the pack holds one expression, keyed `ser inglés` and named `soy inglés`

#### Scenario: The committed packs
- **WHEN** the en-fr, en-es, es-fr and es-en packs are built from their committed tables
- **THEN** each has the sha256 its pin records, the en-fr and en-es pins as they were, and the es-fr and es-en packs carry the names section, their tables and `pack_version` unchanged
