## ADDED Requirements

### Requirement: A word gloss written by a person replaces the source's
A pair's reduction rules SHALL be able to hold a short, closed table of word glosses written and reviewed by a person, each entry giving a lemma's senses with the part of speech of each, and an entry SHALL replace that lemma's gloss and its part-of-speech runs together, and nothing else.
The table is for the words whose source gloss names a form, gives a fringe sense or hides the
meaning behind another part of speech, and that no cleaning rule can mend — in the en-fr pack, the
English modal auxiliaries `would`, `could`, `should`, `might`, `must`, `shall`, `shalt`, `can`,
`may` and `will`. It wins over the senses the requirements above pick, as a pair's curated
expressions win over their source. An entry SHALL change no lemma, form, frequency, level, reading,
expression or dictionary word, and SHALL NOT add a lemma to those the pack glosses: the reduction
SHALL fail, naming the lemma, when an entry names a lemma the source does not gloss. An entry SHALL
obey the limits every word gloss obeys — at most eight senses, each within 300 characters and all
within 800, no sense separator inside a sense, the senses of one part of speech adjacent — and SHALL
NOT be made of proper-noun senses alone; the reduction SHALL fail, naming the lemma, otherwise. Every
gloss the table does not name SHALL be the one the rules made before it. Editing the table is a rule
change: the pair's tables are reduced again.

#### Scenario: `would` reads as the conditional
- **WHEN** the en-fr tables are reduced with the table in place
- **THEN** the gloss of `would` opens on « Conditionnel : would come = viendrait », holds no sense about finding someone attractive, and its part-of-speech table records four verb senses

#### Scenario: A noun kept after the modal
- **WHEN** the table glosses `must` with two verb senses followed by « Incontournable, chose à ne pas manquer », « Moût » and « Moisi, moisissure »
- **THEN** the card of `must` reads its verb senses first, and its part-of-speech table records two verb senses followed by three noun senses

#### Scenario: Only the named glosses move
- **WHEN** the en-fr tables are reduced again from their pinned sources with the table in place
- **THEN** only the ten named lemmas' lines of `gloss.tsv` and `senses.tsv` differ from the tables reduced without it, every other en-fr table and every table of the English studied side are byte for byte the same, and the pack version names the new rule digest

#### Scenario: An entry for a word the source does not gloss
- **WHEN** the table holds an entry for a lemma the source leaves without a gloss
- **THEN** the reduction fails and names the lemma

#### Scenario: An entry past the gloss limits
- **WHEN** an entry holds a sense of 301 characters, or two verb senses separated by a noun sense
- **THEN** the reduction fails and names the lemma

#### Scenario: Another pair does not move
- **WHEN** the en-fr table is edited
- **THEN** the en-es, es-fr and every other pair's tables still pass their checks
