## MODIFIED Requirements

### Requirement: A Spanish document's names are set aside
The page analysis of a Spanish document SHALL set aside, like an out-of-lexicon proper noun, every occurrence of a form that the document never writes in lowercase, that it capitalises at least once in mid-sentence, and whose dictionary form is not one of the pack's dictionary words (lingua-data-packs, *A pack's dictionary words do not depend on its glosses*). A dictionary word, a form also written in lowercase, a form capitalised only at the head of sentences, and a word whose lemma the reader has marked SHALL keep their classification. What is set aside SHALL NOT depend on the pack's glosses. The English analysis SHALL NOT change.

#### Scenario: A character of a novel
- **WHEN** a section reads `Augusto miró a Eugenia` and `Entonces Augusto salió`, and neither `augusto` nor `eugenia` is a dictionary word of the pack
- **THEN** every occurrence of `Augusto` and `Eugenia` is set aside, is not underlined, and does not count in the percentage

#### Scenario: The same form as a word
- **WHEN** the same document also writes `el augusto monarca`
- **THEN** `Augusto` stays a word

#### Scenario: A name with a gloss
- **WHEN** a document capitalises `Dios` in mid-sentence, and the es-fr pack glosses it, so that `dios` is one of its dictionary words
- **THEN** `Dios` stays a word, with its card

#### Scenario: Capitals at the head of sentences only
- **WHEN** a document writes `Augusto calla. Augusto mira el jardín.`
- **THEN** `Augusto` stays a word

#### Scenario: A pack glossed in English
- **WHEN** the section of *A character of a novel* is analysed with a pack built from the es-fr studied tables whose English glosses gloss `augusto` and `eugenia`, and whose dictionary words are es-fr's
- **THEN** every occurrence of `Augusto` and `Eugenia` is set aside, as with the es-fr pack

## ADDED Requirements

### Requirement: An analysis does not depend on the native language
Two packs of one studied language built from the same studied tables SHALL give, glosses and their senses aside, the same page analysis, the same readings and other dictionary forms of a word, the same levels, ladder and vocabulary estimate, whatever native language each is glossed in.
A test SHALL build, for English and for Spanish, a pack from the first pack's studied tables glossed
in another native language, with other glosses, and SHALL compare every probe of that language's
invariance baseline with its glosses and senses removed.

#### Scenario: English through another native language
- **WHEN** the English invariance baseline is answered with the en-fr pack and with a pack built from its studied tables, glossed in Spanish with fewer glosses
- **THEN** every probe is byte for byte alike once glosses and senses are removed

#### Scenario: Spanish through another native language
- **WHEN** the Spanish invariance baseline is answered with the es-fr pack and with a pack built from its studied tables, glossed in English with glosses for `augusto` and `eugenia`
- **THEN** every probe is byte for byte alike once glosses and senses are removed, every token keeping its class and every page its percentage

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after this change
- **THEN** every probe is byte for byte the one recorded before
