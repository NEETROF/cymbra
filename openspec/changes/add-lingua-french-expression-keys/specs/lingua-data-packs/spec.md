## ADDED Requirements

### Requirement: A French pack keys its expressions as French is read
A pack studying French SHALL key each expression by what the core's French analysis reads in its headword — the headword tokenised by French's own pre-pass and each token resolved as a page's token is — writing each token as its dictionary form, except `le`, `la`, `les`, `un`, `une`, `ce`, `cet`, `cette`, `ces`, `mon`, `ma`, `mes`, `ton`, `ta`, `tes`, `son`, `sa`, `ses`, `notre`, `nos`, `votre`, `vos`, `leur` and `leurs`, each written in lowercase as the pre-pass gives it, the tokens joined by single spaces. The key SHALL be built by the same code that reads a page, at build time, never by a copy of French's rules in the reducers. An entry SHALL be kept only when its headword reads as at least two and at most seven tokens, the dictionary form of every token is a lemma of the pack's lexicon, and every word of the headword gives at least one token: a headword without a space that the pre-pass splits is an expression, and one read as a single token is not. Each kept expression SHALL be named by its headword as the dictionary writes it, and the pack SHALL carry that name, in an optional section of its own, wherever it differs from the key. Where several headwords reach one key, the headword written as the key SHALL keep it, then the one with the most tokens written as their key piece, then the first in byte order. For a pack studying French, these rules SHALL take the place of the key rule of *Multi-word expression table* (the dictionary forms of its words, by the lemmatiser), every other rule of that requirement still applying. A pack studying English or Spanish SHALL be keyed as before and carry no such section, byte for byte.

#### Scenario: An expression holding a contracted article
- **WHEN** a French pack is built from a source holding the expression `au revoir`
- **THEN** it is keyed `à le revoir` and named `au revoir`

#### Scenario: An expression holding an elision
- **WHEN** a French pack is built from a source holding `coup d'œil`
- **THEN** it is keyed `coup de œil` and named `coup d'œil`

#### Scenario: A word the pre-pass splits
- **WHEN** a French pack is built from a source holding `d'abord` and `peut-être`, its forms listing `peut-être` whole
- **THEN** `d'abord` is an expression keyed `de abord` and named `d'abord`, and `peut-être` is no expression

#### Scenario: An inflected word
- **WHEN** a French pack is built from a source holding `il y a`
- **THEN** it is keyed `il y avoir` and named `il y a`

#### Scenario: Determiners keep their written form
- **WHEN** a French pack is built from a source holding `à la`, `haut la main` and `haut les mains`
- **THEN** `à la` is keyed `à la`, and `haut la main` and `haut les mains` are two expressions with keys of their own

#### Scenario: A key longer than the window
- **WHEN** a French headword reads as more than seven tokens
- **THEN** the pack leaves it out

#### Scenario: A word the analysis drops
- **WHEN** a French headword holds a word that gives no token, such as the single letter of `compte en t` that the lexicon does not list
- **THEN** the pack leaves it out, rather than keying the words that remain

#### Scenario: Two spellings reach one key
- **WHEN** a French source holds both `boîte à gants` and `boite à gants`, `boite` being a spelling the forms table files under `boîte`
- **THEN** the pack holds one expression, named `boîte à gants`

#### Scenario: English and Spanish packs do not move
- **WHEN** the en-fr, es-fr, es-en and en-es packs are built from their committed tables
- **THEN** each has the sha256 its pin records, and none carries an expression name
