## MODIFIED Requirements

### Requirement: A pair's reduction rules include the rules it shares
The reduction rules of a pair SHALL be its own reducer together with every module of rules shared between pairs that its reducer loads — the rules every pair applies and the rules of the Wiktionary edition its glosses come from — and the record committed with a pair's tables SHALL name those files and carry one digest over all of them. A change to any of them SHALL fail the checks of every pair whose tables were reduced by the previous rules, until those tables are reduced again from their pinned sources; the version of a pack reduced again SHALL name that digest. A rule module a pair's reducer loads SHALL never escape the digest, and the rules of an edition a pair does not read SHALL not be part of its rules. Every pair SHALL be reducible again in one run, which proposes one branch holding every pair's tables with the invariance baselines re-blessed once, and which, asked to expect no change, fails naming the pair and the file whose bytes differ.

#### Scenario: A shared rule changes
- **WHEN** a pull request edits a rule module shared by the en-fr and es-fr reducers without reducing their tables again
- **THEN** the checks of both pairs fail, naming the rule files that changed

#### Scenario: Another pair's reducer changes
- **WHEN** a pull request edits only the es-fr reducer
- **THEN** the en-fr tables still pass their check

#### Scenario: Moving rules into a shared module
- **WHEN** rules are moved out of a pair's reducer into a shared module without changing what they do
- **THEN** the pair's tables reduced again from the same pinned sources are byte-identical, except for the pack version that names the new rule digest

#### Scenario: The rules of an edition no French-native pair reads
- **WHEN** a pull request edits only the English Wiktionary's rules
- **THEN** the en-fr and es-fr tables still pass their checks

#### Scenario: A rule the digest would miss
- **WHEN** a pair's reducer loads a rule module its recorded rules do not name
- **THEN** the checks fail, naming the module

#### Scenario: Every pair in one proposal
- **WHEN** the shared rules change and every pair is reduced again in one run
- **THEN** one branch holds every pair's tables, manifests and pins, with both invariance baselines re-blessed once

#### Scenario: A change meant to move no table
- **WHEN** every pair is reduced again expecting no change, and one byte of es-fr's `gloss.tsv` differs
- **THEN** the run fails naming es-fr and `gloss.tsv`, and proposes no branch

## ADDED Requirements

### Requirement: The committed tables are what the rules make of the pinned sources
The checks of a pull request that changes a pair's tables, its pinned sources, or the rules and tools that reduce them SHALL reduce every committed pair again from its pinned sources, with the pinned interpreter and dependencies, and SHALL fail when the result differs in any byte from the committed tables, `manifest.json` or `pin.json`.

#### Scenario: Rules that move no table
- **WHEN** a pull request generalises the shared gloss rules and commits the en-fr and es-fr tables reduced again
- **THEN** the checks obtain the committed bytes and pass, and under `tables/` the pull request changes only each pair's `manifest.json` (its `pack_version`) and `pin.json`

#### Scenario: A digest written by hand
- **WHEN** a pull request edits a shared rule that changes es-fr's glosses and writes the new digest into `pin.json` without reducing the tables again
- **THEN** the checks fail, naming es-fr and `gloss.tsv`

#### Scenario: A table edited by hand
- **WHEN** a pull request edits a line of en-fr's `gloss.tsv` and records the pack it builds
- **THEN** the checks fail, naming en-fr and `gloss.tsv`

### Requirement: A gloss is in the reader's language, written by a person
A pack's glosses SHALL be written by a person in the pack's native language: never in the studied language, never in a third language, never machine-translated, and never pivoted through a third language. A word a person wrote into a Wiktionary translation table SHALL qualify when the table pairs the studied and the native language directly: the native words a studied entry lists, or the native entry whose table lists the studied word.

#### Scenario: An English gloss is not a French one
- **WHEN** neither the French Wiktionary nor a French–Spanish translation table glosses a Spanish lemma that the English Wiktionary's Spanish section glosses in English
- **THEN** the es-fr pack gives that lemma no gloss

#### Scenario: No pivot through English
- **WHEN** one sense of the English Wiktionary's `house` lists French *maison* and Spanish *casa* among its translations
- **THEN** neither glosses the other in any pack

#### Scenario: A translation table qualifies
- **WHEN** a sense of the English Wiktionary's English entry `house` lists Spanish *casa* among its translations
- **THEN** a pack studying English and glossed in Spanish may gloss `house` « casa »

#### Scenario: Nothing written, no gloss
- **WHEN** no person-written source glosses a lemma in the pack's native language
- **THEN** the lemma has no gloss, and no translation engine fills it

### Requirement: A gloss is cleaned by the rules of the Wiktionary edition that wrote it
The reducer SHALL clean each gloss by the rules of the Wiktionary edition it comes from — French, English or Spanish: which senses only name the word they are a form of, which notes and placeholders the edition writes for its own readers, and which coordinators a sense can be left hanging on. One edition's rules SHALL NOT change a gloss taken from another, and the French Wiktionary's rules SHALL clean as they did before.

#### Scenario: The French Wiktionary's glosses do not move
- **WHEN** the en-fr and es-fr tables are reduced again from their pinned sources with every edition's rules in place
- **THEN** their seven data tables and NOTICE are byte for byte as committed

#### Scenario: An English form-of sense
- **WHEN** the English Wiktionary glosses the noun `casas` only « plural of casa »
- **THEN** that sense is read as a form of `casa`, not as a meaning

#### Scenario: A Spanish form-of sense
- **WHEN** the Spanish Wiktionary glosses `chips` « Forma del plural de chip. » and `read` « Participio pasado del verbo (to) read. »
- **THEN** each sense is read as a form of `chip` and of `read`, not as a meaning

#### Scenario: An English sense with no gloss
- **WHEN** the English Wiktionary marks a sense as having no gloss
- **THEN** the sense is left out, and a word left with no sense has no gloss

### Requirement: A translation table is read wherever the edition writes it
A file derived from a dump's translations SHALL hold every translation into its target language that an entry lists, whether the edition writes the table for the whole entry or under one of its senses, with the sense the table names when it names one.

#### Scenario: The English Wiktionary's tables under senses
- **WHEN** the Spanish translations of the English Wiktionary's English entries are derived
- **THEN** the entries whose tables sit under a sense are kept with them, about 68,579 entries against 5,080 with a table for the whole entry

#### Scenario: An edition that writes tables per entry
- **WHEN** an entry lists its translations only for the whole entry, as every French and Spanish Wiktionary entry es-fr reads does
- **THEN** its derived line is byte for byte the one derived before this change
