## ADDED Requirements

### Requirement: A Spanish document's names are set aside
The page analysis of a Spanish document SHALL set aside, like an out-of-lexicon proper noun, every occurrence of a form that the document never writes in lowercase, that it capitalises at least once in mid-sentence, and whose dictionary form the pack does not gloss. A form the pack glosses, a form also written in lowercase, a form capitalised only at the head of sentences, and a word whose lemma the reader has marked SHALL keep their classification. The English analysis SHALL NOT change.

#### Scenario: A character of a novel
- **WHEN** a section reads `Augusto miró a Eugenia` and `Entonces Augusto salió`, and the pack glosses neither `augusto` nor `eugenia`
- **THEN** every occurrence of `Augusto` and `Eugenia` is set aside, is not underlined, and does not count in the percentage

#### Scenario: The same form as a word
- **WHEN** the same document also writes `el augusto monarca`
- **THEN** `Augusto` stays a word

#### Scenario: A name with a gloss
- **WHEN** a document capitalises `Dios` in mid-sentence and the pack glosses it
- **THEN** `Dios` stays a word, with its card

#### Scenario: Capitals at the head of sentences only
- **WHEN** a document writes `Augusto calla. Augusto mira el jardín.`
- **THEN** `Augusto` stays a word
