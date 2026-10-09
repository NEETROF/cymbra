## ADDED Requirements

### Requirement: A pack's lexicon reads every lemma as itself
The pack builder SHALL refuse, naming them, a forms table that lists one form with two lemmas, and a lemma — of the forms table, the ranks or the glosses — whose own spelling the forms table reads as another lemma; and it SHALL file every lemma's rank, gloss, level, dictionary-word mark, readings, other dictionary forms and sense runs at that lemma's own place in the pack's lemma list.
A level, a dictionary word, a reading or a sense run written for a string that is no lemma of the
pack SHALL be filed under no lemma, and never under the lemma that string is a form of: what the
builder files under a lemma is what the core reads for that lemma (*A dictionary form is read as
itself*). The pack's lemma list SHALL be in strictly ascending byte order, as the builder writes it,
and the core SHALL refuse to load a pack whose list is not, as it refuses one whose forms point
outside the list. Every pair's committed tables already meet these rules: their packs keep their
bytes.

#### Scenario: The committed pairs
- **WHEN** en-fr, es-fr, es-en and en-es are built from their committed tables
- **THEN** each builds, to the sha256 its pin records: no form of theirs has two lemmas, every lemma reads as itself, and nothing was filed under another lemma

#### Scenario: A lemma whose own spelling reads as another
- **WHEN** a pair's tables rank `venue`, and its forms table maps the form `venue` to `venir`
- **THEN** the build fails, naming `venue` and `venir`, where it gave *venir* the rank of `venue`

#### Scenario: A form listed with two lemmas
- **WHEN** a forms table maps the form `porte` once to `porte` and once to `porter`
- **THEN** the build fails, naming `porte` and both lemmas, where it read the form as whichever lemma sorts first

#### Scenario: A level written for a form
- **WHEN** a level table gives `donnée` B1 and `donner` A1, `donnée` is no lemma of the pack, and the forms table maps the form `donnée` to `donner`
- **THEN** no lemma of the pack takes the B1 written for `donnée`, and `donner` is A1

#### Scenario: A lemma list out of order
- **WHEN** a pack's lemma list is not in strictly ascending byte order, or names a lemma twice
- **THEN** the core refuses to load it, with an explicit error, and produces no partial analysis
