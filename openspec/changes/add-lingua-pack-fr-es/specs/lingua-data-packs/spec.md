## ADDED Requirements

### Requirement: French is glossed in Spanish from the Spanish Wiktionary's French section and the French Wiktionary's translation tables
fr-es's glosses, sense runs and expressions SHALL be reduced from the Spanish Wiktionary's French section with the Spanish edition's rules, then, for what that section leaves out, from the Spanish translations the French Wiktionary's French entries list, in the table's order, then from the Spanish entries of the Spanish Wiktionary whose translations list the French word, the commonest Spanish word first — words people wrote in Spanish, with no pivot and no machine translation —, and fr-es SHALL read French's committed studied tables as a pair that is not its studied language's reference does, loading no other pair's reducer.
Before the shared rules read the section, its letters are left out; its senses are read as the
Spanish edition reads en-es's — the senses it marks obsolete or outdated after the others of their
entry, one typography —; the sense of a capitalised proper noun that only says a word is a surname or
a given name glosses no word that has an entry of its own in lower case, other than a proper noun's,
holding a meaning; and an adjective section the edition tags possessive or demonstrative, or whose
every sense is a form of such a word, is read as a determiner. A typographic apostrophe in a French
headword, or in a French word of either table, is read as `'`. A definition whose every sense is the
French headword itself, as written up to case, SHALL give way to the French Wiktionary's Spanish
translations of the word when they hold no word spelled as the headword. In both tables a letter
SHALL gloss no word, and a Spanish word that several parts of speech of one Spanish entry list for a
French word SHALL be listed once, under the first of them French's readings name, or else the first
listed. A pack built from fr-es's tables carries French's dictionary words, fr-en's glossed lemmas:
a lemma fr-es alone glosses is none. The share of the glossed lemmas a definition glosses — of all
of them, and of the 10,000 commonest — SHALL be measured and shown with the tables, and stored in no
pack. The Spanish edition's rules SHALL be read by en-es and fr-es and by no other committed pair.

#### Scenario: A definition first
- **WHEN** the Spanish Wiktionary's French section defines `maison` « Casa. »
- **THEN** the gloss of `maison` is « Casa », as a noun

#### Scenario: A direct-table gloss
- **WHEN** the section has no entry for `intérêt`, and the French Wiktionary's `intérêt` lists « interés » among its Spanish translations
- **THEN** the gloss of `intérêt` is « Interés », at most three Spanish words per part of speech

#### Scenario: An inverted-table gloss
- **WHEN** neither the section nor the French Wiktionary's table glosses `travers`, and the Spanish Wiktionary's `través` lists `travers` as its French translation
- **THEN** the gloss of `travers` is « Través »

#### Scenario: The studied word is no gloss
- **WHEN** the section defines the conjunction `et` « Et. » and the French Wiktionary translates `et` « y » and « e »
- **THEN** the gloss of `et` is « Y, e »

#### Scenario: A cognate keeps its definition
- **WHEN** the section defines `venir` « Venir. » and the French Wiktionary's table lists « venir »
- **THEN** the gloss of `venir` is « Venir »

#### Scenario: A name on a common word's card
- **WHEN** the section has `pierre`, a noun defined « Piedra. », and `Pierre`, a proper noun defined « Nombre de pila de varón, equivalente del español Pedro. »
- **THEN** the gloss of `pierre` is « Piedra »

#### Scenario: A name's own row
- **WHEN** the section has `François`, a proper noun defined « Nombre de pila de varón, equivalente del español Francisco », and no `françois` in lower case
- **THEN** the gloss of `françois` is that note, as before

#### Scenario: Possessives and their forms
- **WHEN** the section has `mon`, an adjective section tagged possessive defined « Mi. », and `mes`, an adjective section whose every sense is a form of `mon`
- **THEN** `mon` and `mes` are glossed « Mi » as determiners

#### Scenario: A letter glosses no word
- **WHEN** French's tables rank `h` and `x` among the 500 commonest lemmas, and the only Spanish word a translation table gives each is the letter itself
- **THEN** `h` and `x` have no gloss in fr-es, and `à` and `y` keep the section's definitions

#### Scenario: A Spanish word listed once
- **WHEN** the Spanish Wiktionary's `este` lists `cet` among its French translations as an adjective and as a pronoun
- **THEN** the gloss of `cet` is « Este », once

#### Scenario: A typographic apostrophe
- **WHEN** the French Wiktionary's `main-d’œuvre` lists « mano de obra » among its Spanish translations
- **THEN** the committed lemma `main-d'œuvre` is glossed « Mano de obra »

#### Scenario: French's dictionary words
- **WHEN** fr-es's pack is built from `tables/fr/` and `tables/fr-es/`
- **THEN** it carries a lexical table listing exactly the lemmas `tables/fr/lexical.tsv` lists, and a lemma fr-es glosses and fr-en does not (`quant` in the prototype) is no dictionary word

#### Scenario: The pack is built, not shipped
- **WHEN** a pull request runs the extension's checks
- **THEN** fr-es's pack is built from `tables/fr/` and `tables/fr-es/`, has the sha256 its pin records and weighs less than 5 MiB, and the extension's list of shipped pairs does not name it

#### Scenario: A rule of the Spanish edition
- **WHEN** `reduce_edition_es.py` changes
- **THEN** en-es's and fr-es's rule digests move, and en-fr's, es-fr's, es-en's and fr-en's do not

#### Scenario: Nothing else moves
- **WHEN** fr-es's native side is committed
- **THEN** en-fr's, es-fr's, es-en's, en-es's and fr-en's tables, pins and packs and every table of `tables/fr/` are byte for byte as before, and the invariance baselines and the extension's snapshots pass without re-blessing

### Requirement: fr-es is held to a coverage floor fixed before its first committed measurement
fr-es SHALL gloss at least 81.4, 68.8 and 54.5 % of the 5,000, 10,000 and 20,000 commonest French lemmas, a floor fixed before the first measurement of the tables it commits, kept in one place and checked by the reduce job on every pull request that reduces fr-es; tables measured under it SHALL NOT be committed, and no package SHALL list fr-es before committed tables measured at or above it.
The floor is the study's figures less two points, as en-es's and fr-en's are. It SHALL NOT be
lowered in the pull request that measured fr-es under it: a lower floor is a decision of the owner,
recorded before the measurement it applies to. When the update that would give fr-es its first
committed tables measures under the floor at any of the three tops, those tables are not committed,
no package lists fr-es — French then ships for English speakers alone — and fr-es is measured again
at a later regeneration of the dumps, against the same floor. Once fr-es is committed, a pull request
that reduces it again — its sources updated, its rules or the Spanish edition's changed, French's
studied tables moved — and measures it under the floor SHALL fail the checks, naming the pair, the
top and the figure, and the committed fr-es stays as it was. Once a package lists fr-es, its
coverage SHALL be published with the other shipped pairs', measured as theirs is.

#### Scenario: The floor
- **WHEN** `gloss_coverage.py --pair fr-es` runs on the committed tables, against its floor of 81.4, 68.8 and 54.5 %
- **THEN** it passes, and the measured figures are shown in the pull request

#### Scenario: One place
- **WHEN** the reduce job measures fr-es
- **THEN** it reads the floor from `gloss_coverage.py`'s `FLOORS`, passes none of its own, and the tests hold that entry to the value this requirement states

#### Scenario: Under the floor at the first measurement
- **WHEN** the update dispatched for fr-es's first tables measures 81.0 % of the 5,000 commonest lemmas glossed
- **THEN** its tables are not committed, no package lists fr-es, and fr-es is measured again at a later regeneration of the dumps, against 81.4, 68.8 and 54.5 %

#### Scenario: A later pull request under the floor
- **WHEN** an update of French's studied tables reduces fr-es again and it measures under the floor at one top
- **THEN** the checks fail, naming fr-es, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["fr-es"]` and this requirement's value is unchanged
- **THEN** the tests fail

#### Scenario: Published once listed
- **WHEN** a package lists fr-es
- **THEN** the published coverage figures hold fr-es's beside the other shipped pairs', measured on the committed tables as theirs are
