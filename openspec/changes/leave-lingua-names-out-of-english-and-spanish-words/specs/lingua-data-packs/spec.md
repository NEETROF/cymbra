## MODIFIED Requirements

### Requirement: A pack's dictionary words do not depend on its glosses
A pack SHALL hold the same dictionary words — the lemmas its vocabulary sizes count and the names rules keep as words — whatever native language it is glossed in: the lemmas its studied language's reference pack glosses — en-fr for English, es-fr for Spanish, fr-en for French, and for a language studied later the first pack built for it —, less those the reference glosses by a proper noun's senses alone, which are names, not words to learn.
A lemma glossed by a proper noun's senses alone is one every sense run of whose gloss is a proper
noun's, as the reference's Wiktionary edition files it; a word with a common sense beside a name's
stays a word, and a name keeps its gloss. A lemma a CEFR list levels keeps its level, whether or not it
is a dictionary word, and the vocabulary sizes count it. The rule is the reference's reduction's: it
reads the reference's own glosses and sense runs, never another pair's.
A pack whose glossed lemmas are not its dictionary words SHALL name them in a lexical table, an
optional section that a core which does not read it ignores. A pack without one SHALL read its
glossed lemmas as its dictionary words; the reference packs, whose dictionary words leave out the
lemmas they gloss as names alone, carry one. The builder SHALL read a pack's dictionary words from an
optional `lexical.tsv` beside its tables and SHALL write the lexical table only when they differ from
the lemmas the pack glosses. When it writes a lexical table, it SHALL refuse, naming the lemma, a
dictionary word or a glossed lemma that is neither the lemma of a form nor a ranked lemma, so that no
native language's glosses can add a lemma to the lexicon.

The checks SHALL fail when two packs of one studied language, built from committed tables, hold
different dictionary words.

#### Scenario: The shipped packs carry no lexical table
- **WHEN** a pack is built from tables whose dictionary words are exactly the lemmas it glosses, as en-fr's and es-fr's were until their references' names left English's and Spanish's dictionary words
- **THEN** it carries no lexical table, and keeps the bytes it had before the lexical table existed

#### Scenario: The reference packs carry a lexical table
- **WHEN** the en-fr, es-fr and fr-en packs are built from their committed tables
- **THEN** each carries a lexical table listing its studied language's dictionary words, and each sha256 is the one its pin records

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

#### Scenario: French's names
- **WHEN** fr-en glosses `paris` and `durand` only by proper nouns' senses, and `marche` by a common noun's beside a department's
- **THEN** `paris` and `durand` are no dictionary words of French, `marche` is, fr-en's pack carries a lexical table, and a French document that capitalises `Paris` in mid-sentence and never writes it in lowercase sets it aside as a name

#### Scenario: English's names
- **WHEN** en-fr glosses `london` « Londres; … » and `margaret` « Marguerite, … » only by proper nouns' senses, and `bill` by common nouns' and verbs' senses beside a name's
- **THEN** `london` and `margaret` are no dictionary words of English, `bill` is, en-fr still glosses all three, and the en-es pack holds the same dictionary words as the en-fr pack

#### Scenario: Spanish's names
- **WHEN** es-fr glosses `madrid` « Madrid » and `maría` « Marie; María » only by proper nouns' senses, `dios` « Dieu » by a common noun's, and `luna` by a common noun's beside a name's
- **THEN** `madrid` and `maría` are no dictionary words of Spanish, `dios` and `luna` are, es-fr still glosses all four, and the es-en pack holds the same dictionary words as the es-fr pack

#### Scenario: A month a CEFR list levels
- **WHEN** en-fr glosses `june` « Juin; Prénom épicène… » only by proper nouns' senses, and CEFR-J gives it A1
- **THEN** `june` is no dictionary word of English, keeps its level A1, and the vocabulary estimate's universe counts it

### Requirement: A studied language's tables are kept once
The tables of a studied language — its forms, frequencies, readings and levels, its pinned tag pool and its dictionary words — SHALL be committed once, in that language's folder (`tables/<studied>/`), and every pair studying that language SHALL be built from that folder together with its own.
A pair's own folder (`tables/<pair>/`) SHALL hold what belongs to the pair alone: its glosses, the
parts of speech of their senses and its expressions, with its notice, manifest, pin and README. A
pair's folder SHALL NOT hold a table of its studied language, and the checks SHALL fail, naming the
pair, the file and the studied language's reference pair, when one does. The dictionary words SHALL
be those *A pack's dictionary words do not depend on its glosses* names, as the reference pair's
reduction writes them, kept as `lexical.tsv` in the studied language's folder, and the checks SHALL
fail, naming the reference pair and a lemma, when they are neither every lemma the reference glosses
nor every one but those it glosses by a proper noun's senses alone, or when the folder lacks its tag
pool or its dictionary words. Moving the tables SHALL leave every shipped pack byte-identical.

#### Scenario: The shipped packs keep their bytes
- **WHEN** the en-fr pack is built from `tables/en/` and `tables/en-fr/`, and the es-fr pack from `tables/es/` and `tables/es-fr/`
- **THEN** each sha256 is the one its `pin.json` records, each carries a lexical table listing its studied language's dictionary words, and both invariance baselines pass as committed

#### Scenario: One copy per studied language
- **WHEN** the committed tables are read
- **THEN** English's `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `tags.tsv` and `lexical.tsv` are in `tables/en/` alone, Spanish's in `tables/es/` alone, and `tables/en-fr/` and `tables/es-fr/` hold only `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md`

#### Scenario: A studied table left in a pair's folder
- **WHEN** a second pair studying Spanish holds its own `grammar.tsv` or `lexical.tsv` in its folder
- **THEN** the checks fail, naming that pair, the file and es-fr

#### Scenario: Dictionary words that are not the reference's
- **WHEN** Spanish's `lexical.tsv` lists a lemma es-fr does not gloss, or leaves out one es-fr glosses by a common noun's sense (`casa`)
- **THEN** the checks fail, naming es-fr and the lemma

#### Scenario: A pair glossed in another native language copies nothing
- **WHEN** a test pair studying Spanish, glossed in English, is built from `tables/es/` and a folder holding only its glosses, senses, expressions, notice and manifest
- **THEN** it builds, its readings are stored byte for byte as es-fr's, and its dictionary words are es-fr's

#### Scenario: Names left out by halves
- **WHEN** French's `lexical.tsv` leaves out `paris` but lists `lyon`, both glossed by fr-en with a proper noun's senses alone
- **THEN** the checks fail, naming fr-en and `lyon`

### Requirement: Only the reference pair's reduction writes its studied language's tables
A studied language's tables SHALL be written only by the reduction of its reference pair — en-fr for English, es-fr for Spanish, and for a language studied later the first pair built for it — from that pair's pinned sources under that pair's recorded rules, and any other pair's reduction SHALL read them as committed and write only its own folder.
The reference pair's reduction reads its native side too: the French Wiktionary's form links give
English forms, French glosses decide which Spanish lemmas take a level, and the reference's glossed
lemmas, less those it glosses by a proper noun's senses alone, are the dictionary words. Its pin and
its record of rules SHALL therefore be the record of its studied language's tables, and no reducer is
edited by the move. The pinned tag pool, which no reduction writes, SHALL be kept by every reduction.
A studied language's folder SHALL name its reference pair, SHALL be no pair, and SHALL never be built,
reduced or reported on as one. A run that reduces several pairs again SHALL reduce each reference pair
before the other pairs of its language. A change to a reference pair's rules SHALL fail the checks of
every pair of its studied language until the reference pair is reduced again.

#### Scenario: A reference pair reduced again
- **WHEN** es-fr is reduced again from its pinned sources
- **THEN** it writes `tables/es/` and `tables/es-fr/` byte for byte as committed, keeps `tables/es/tags.tsv`, and leaves its `pin.json` unchanged

#### Scenario: Another pair of the language reduced again
- **WHEN** a pair studying Spanish other than es-fr is reduced again or updated
- **THEN** it reads Spanish's committed tables and writes nothing in `tables/es/`

#### Scenario: A studied table edited by hand
- **WHEN** a pull request edits a line of `tables/en/forms.tsv` and records the pack en-fr builds from it
- **THEN** the checks that reduce every pair again fail, naming en-fr and `en/forms.tsv`

#### Scenario: A studied table meant not to move
- **WHEN** every pair is reduced again expecting no change, and one byte of Spanish's `level.tsv` differs
- **THEN** the run fails naming es-fr and `es/level.tsv`, and proposes no branch

#### Scenario: A studied language's folder is not a pair
- **WHEN** the checks, the reduction of every pair or the update of every pair walk the committed tables
- **THEN** they build and reduce en-fr and es-fr, and never take `en` or `es` for a pair

#### Scenario: A reference pair's rules changed and not applied
- **WHEN** a pull request edits only es-fr's reducer without reducing its tables again
- **THEN** the checks of every pair studying Spanish fail, naming es-fr's rule files, and en-fr's still pass
