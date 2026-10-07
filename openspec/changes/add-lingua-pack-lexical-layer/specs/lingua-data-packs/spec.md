## ADDED Requirements

### Requirement: A pack's dictionary words do not depend on its glosses
A pack SHALL hold the same dictionary words — the lemmas its vocabulary sizes count and the Spanish names rule keeps as words — whatever native language it is glossed in: the lemmas its studied language's reference pack glosses — en-fr for English, es-fr for Spanish, and for a language studied later the first pack built for it.
A pack whose glossed lemmas are not its dictionary words SHALL name them in a lexical table, an
optional section that a core which does not read it ignores. A pack without one SHALL read its
glossed lemmas as its dictionary words, so the en-fr and es-fr packs carry no lexical table and keep
their bytes. The builder SHALL read a pack's dictionary words from an optional `lexical.tsv` beside
its tables and SHALL write the lexical table only when they differ from the lemmas the pack glosses.
When it writes a lexical table, it SHALL refuse, naming the lemma, a dictionary word or a glossed
lemma that is neither the lemma of a form nor a ranked lemma, so that no native language's glosses
can add a lemma to the lexicon.

The checks SHALL fail when two packs of one studied language, built from committed tables, hold
different dictionary words.

#### Scenario: The shipped packs carry no lexical table
- **WHEN** the en-fr and es-fr packs are built from their committed tables after this change
- **THEN** neither carries a lexical table, and each sha256 is the one its pin records

#### Scenario: A pack glossed in English
- **WHEN** a pack studying Spanish is built from the es-fr studied tables, with English glosses that gloss `augusto`, and a `lexical.tsv` listing the lemmas es-fr glosses
- **THEN** it carries a lexical table, `casa` is one of its dictionary words, and `augusto`, which es-fr does not gloss, is not

#### Scenario: A lexical table that says what the glosses say
- **WHEN** a pack's `lexical.tsv` lists exactly the lemmas it glosses
- **THEN** the pack is byte for byte the one built without that file

#### Scenario: A gloss outside the lexicon
- **WHEN** a pack with a `lexical.tsv` glosses a lemma that no form maps to and no rank lists
- **THEN** the build fails and names the lemma

#### Scenario: Two packs of one language disagree
- **WHEN** the committed tables of a second pair studying Spanish name other dictionary words than es-fr's
- **THEN** the checks fail, naming both pairs

#### Scenario: An older core
- **WHEN** a core built before this change loads a pack carrying a lexical table
- **THEN** the pack loads, and that core reads its glossed lemmas as its dictionary words

### Requirement: A form's readings do not depend on the pack's native language
Two packs of one studied language built from the same forms, frequencies and readings SHALL store those readings byte for byte alike, whatever tags their glosses' senses carry.
A pack built with its studied language's pinned tag pool SHALL hold, in this order, that pool, then
the tags of its readings outside it, then the tags only its senses carry, each part sorted. English's
and Spanish's pinned pools SHALL be the pools the en-fr and es-fr packs carry, so that both keep their
bytes, and every pair built from committed tables SHALL be built with its studied language's pinned
pool. A pack built without one, such as a test pack, SHALL keep a single sorted pool, as before. The
checks SHALL fail when two packs of one studied language, built from committed tables, store a form's
readings differently.

#### Scenario: The shipped packs keep their pools
- **WHEN** the en-fr and es-fr packs are built from their committed tables after this change
- **THEN** each carries the tag pool it carried before, and each sha256 is the one its pin records

#### Scenario: A sense part of speech the first pack never used
- **WHEN** a pack studying English is built from the en-fr studied tables with a sense run tagged `NUM`, which no en-fr sense carries
- **THEN** it builds, its readings are stored byte for byte as en-fr's, and the core reads that run as `NUM`

#### Scenario: Fewer sense tags
- **WHEN** a pack studying Spanish is built from the es-fr studied tables with no sense run tagged `INTJ`, `SYM` or `X`
- **THEN** its readings are stored byte for byte as es-fr's

#### Scenario: A reading outside the pinned pool
- **WHEN** a test pack studying Spanish gives `me` a reading whose tag Spanish's pinned pool does not hold
- **THEN** it builds, and the core reads that reading back

### Requirement: A noun's gender comes from its readings
The builder SHALL give a noun's sense runs the gender its dictionary form is read with, when the readings of that form as itself give exactly one, whatever native language the gloss is written in, and SHALL refuse, naming the word, a noun run whose gender those readings contradict.
A noun those readings give both genders, or none, SHALL keep a run without a gender.

#### Scenario: The es-fr pack does not change
- **WHEN** the es-fr pack is built from its committed tables after this change
- **THEN** every noun run carries the gender it carried before, and its sha256 is the one its pin records

#### Scenario: Glosses that say no gender
- **WHEN** a pack studying Spanish is built from the es-fr studied tables with English glosses whose noun runs carry no gender
- **THEN** the noun run of `casa` reads `NOUN|Gender=Fem`, as es-fr's does

#### Scenario: A noun of both genders
- **WHEN** such a pack is built and `estudiante` is read as masculine and as feminine
- **THEN** its noun run carries no gender

#### Scenario: A run its readings contradict
- **WHEN** a pair's sense table tags a noun's run `NOUN|Gender=Masc` and the noun's readings give it only the feminine
- **THEN** the build fails and names the noun
