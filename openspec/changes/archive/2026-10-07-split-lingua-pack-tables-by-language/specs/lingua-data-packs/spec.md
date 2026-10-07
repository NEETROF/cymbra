## ADDED Requirements

### Requirement: A studied language's tables are kept once
The tables of a studied language — its forms, frequencies, readings and levels, its pinned tag pool and its dictionary words — SHALL be committed once, in that language's folder (`tables/<studied>/`), and every pair studying that language SHALL be built from that folder together with its own.
A pair's own folder (`tables/<pair>/`) SHALL hold what belongs to the pair alone: its glosses, the
parts of speech of their senses and its expressions, with its notice, manifest, pin and README. A
pair's folder SHALL NOT hold a table of its studied language, and the checks SHALL fail, naming the
pair, the file and the studied language's reference pair, when one does. The dictionary words SHALL
be the lemmas the reference pair glosses (*A pack's dictionary words do not depend on its glosses*),
kept as `lexical.tsv` in the studied language's folder, and the checks SHALL fail, naming the
reference pair and a lemma, when they are not, or when the folder lacks its tag pool or its
dictionary words. Moving the tables SHALL leave every shipped pack byte-identical.

#### Scenario: The shipped packs keep their bytes
- **WHEN** the en-fr pack is built from `tables/en/` and `tables/en-fr/`, and the es-fr pack from `tables/es/` and `tables/es-fr/`
- **THEN** each sha256 is the one its `pin.json` recorded before the move, neither pack carries a lexical table, and both invariance baselines pass without being re-blessed

#### Scenario: One copy per studied language
- **WHEN** the committed tables are read
- **THEN** English's `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `tags.tsv` and `lexical.tsv` are in `tables/en/` alone, Spanish's in `tables/es/` alone, and `tables/en-fr/` and `tables/es-fr/` hold only `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md`

#### Scenario: A studied table left in a pair's folder
- **WHEN** a second pair studying Spanish holds its own `grammar.tsv` or `lexical.tsv` in its folder
- **THEN** the checks fail, naming that pair, the file and es-fr

#### Scenario: Dictionary words that are not the reference's
- **WHEN** Spanish's `lexical.tsv` lists a lemma es-fr does not gloss, or leaves out one it glosses
- **THEN** the checks fail, naming es-fr and the lemma

#### Scenario: A pair glossed in another native language copies nothing
- **WHEN** a test pair studying Spanish, glossed in English, is built from `tables/es/` and a folder holding only its glosses, senses, expressions, notice and manifest
- **THEN** it builds, its readings are stored byte for byte as es-fr's, and its dictionary words are es-fr's

### Requirement: Only the reference pair's reduction writes its studied language's tables
A studied language's tables SHALL be written only by the reduction of its reference pair — en-fr for English, es-fr for Spanish, and for a language studied later the first pair built for it — from that pair's pinned sources under that pair's recorded rules, and any other pair's reduction SHALL read them as committed and write only its own folder.
The reference pair's reduction reads its native side too: the French Wiktionary's form links give
English forms, French glosses decide which Spanish lemmas take a level, and the reference's glossed
lemmas are the dictionary words. Its pin and its record of rules SHALL therefore be the record of
its studied language's tables, and no reducer is edited by the move. The pinned tag pool, which no
reduction writes, SHALL be kept by every reduction. A studied language's folder SHALL name its
reference pair, SHALL be no pair, and SHALL never be built, reduced or reported on as one. A run
that reduces several pairs again SHALL reduce each reference pair before the other pairs of its
language. A change to a reference pair's rules SHALL fail the checks of every pair of its studied
language until the reference pair is reduced again.

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

### Requirement: A change to a studied language's tables reaches every pair of that language
A pull request that changes a studied language's tables SHALL, in that same pull request, record again the pack of every pair of that language and reduce again every such pair whose reduction reads them, and a pair whose pack no longer matches its pin SHALL fail the checks.
An update of a reference pair SHALL propose, on one branch, its studied language's tables, its own
tables and every other pair of that language brought to them. Its report SHALL list what the
studied tables change — forms, ranks, levels, readings and dictionary words — and name every pair
whose pack moves.

#### Scenario: Spanish's tables change under a second pair
- **WHEN** es-fr's update changes Spanish's levels, and a second pair studies Spanish
- **THEN** one branch holds `tables/es/`, es-fr's tables and the second pair's, each pin recording its new pack, and the report lists the levels changed and names both pairs

#### Scenario: A pair left behind
- **WHEN** a pull request changes `tables/es/` and does not record a second Spanish pair's new pack
- **THEN** the checks fail, naming that pair

#### Scenario: A rule of the reference's own edition
- **WHEN** a pull request changes the French Wiktionary's rules so that es-fr glosses one more lemma, and reduces es-fr again
- **THEN** Spanish's dictionary words gain that lemma, and the checks of every other pair studying Spanish fail until its pack is recorded again

### Requirement: A pair's committed tables are its own and its studied language's
Wherever a requirement speaks of a pair's committed tables — those its pack is built from and checked against its pin, reduced again, reported on, measured, or packed for a store's reviewer — it SHALL mean the pair's own folder together with its studied language's folder, the studied language being the first side of the pair's name.
The build SHALL fail, naming the pair and what to add, when that language has no folder, and naming
the pair and both languages when the pair's manifest studies another language.

#### Scenario: A listed pair whose studied language has no tables
- **WHEN** the list names a pair whose studied language has no folder under `tables/`
- **THEN** the build fails with a message naming the pair and the folder to add

#### Scenario: A pair whose manifest studies another language
- **WHEN** a pair named `es-en` holds a manifest naming `en` as the language it studies
- **THEN** the build fails, naming the pair and both languages

#### Scenario: The published coverage follows the studied tables
- **WHEN** Spanish's `freq.tsv` changes and the published gloss coverage no longer matches the tables
- **THEN** the coverage check fails until the figures are measured again

#### Scenario: The reviewer's rebuild after the move
- **WHEN** a reviewer rebuilds the packs from the submitted source archive with no network
- **THEN** the archive holds every listed pair's folder and its studied language's, and the reviewer obtains each pack the package carries, byte for byte
