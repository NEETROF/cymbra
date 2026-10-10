## ADDED Requirements

### Requirement: An English document's names are set aside
The page analysis of an English document SHALL set aside, like an out-of-lexicon proper noun, every occurrence of a form that the document never writes in lowercase, that it capitalises at least once in mid-sentence, and whose dictionary form is neither one of the pack's dictionary words (lingua-data-packs, *A pack's dictionary words do not depend on its glosses*) nor a lemma that carries a CEFR level, and the pronoun `I` and its contractions SHALL give no evidence.
A dictionary word, a lemma a CEFR list levels, a form also written in lowercase, a form capitalised
only at the head of sentences, and a word whose lemma the reader has marked SHALL keep their
classification. A form that is `i`, or opens on `i'` or `i’` (`I'm`, `I'll`, `I'd`, `I've`), is
capitalised wherever it stands and is never a name. A capital after an apostrophe gives no evidence of
its own, and a hyphenated run the pack does not list is judged by its parts, as in a Spanish document.
What is set aside SHALL NOT depend on the pack's glosses, so a pack studying English glossed in another
language sets aside what the en-fr pack does. The rule changes English's output, so English's analyser
version is bumped (*An analyser version per studied language*); Spanish's and French's names rules and
analyser versions do not change.

#### Scenario: A character of a story
- **WHEN** a section reads « When the train pulled into the station, Margaret grabbed her brother's coat », and `margaret` is a lemma of the pack but not one of its dictionary words
- **THEN** `Margaret` is set aside, is not underlined, and does not count in the percentage

#### Scenario: A city
- **WHEN** a document reads « We flew from Paris to London. London was cold. », and `london` is a lemma of the pack but not one of its dictionary words
- **THEN** both occurrences of `London` are set aside

#### Scenario: The pronoun I
- **WHEN** a document reads « Tell Mum I'll call, and I'm sure she knows », and neither `i'll` nor `i'm` is a dictionary word of the pack
- **THEN** `I'll` and `I'm` stay words

#### Scenario: A word a CEFR list levels
- **WHEN** a document reads « The lanes open in June, said Mr Brown », `june` and `mr` carry a CEFR level, and neither is a dictionary word of the pack
- **THEN** `June` and `Mr` stay words

#### Scenario: A dictionary word
- **WHEN** a document capitalises `Bill` in mid-sentence, and `bill` is one of the pack's dictionary words
- **THEN** `Bill` stays a word, with its card

#### Scenario: Capitals at the head of sentences only
- **WHEN** a document writes « Margaret ran. Margaret stopped. »
- **THEN** `Margaret` stays a word

#### Scenario: The same form as a word
- **WHEN** a document writes « said Sam » and also « sam », in lowercase, in a user name
- **THEN** `Sam` stays a word

#### Scenario: English glossed in Spanish
- **WHEN** the English baseline's `fiction` page is analysed with the en-es pack, whose dictionary words are en-fr's
- **THEN** `Margaret` is set aside, as with the en-fr pack

## MODIFIED Requirements

### Requirement: A Spanish document's names are set aside
The page analysis of a Spanish document SHALL set aside, like an out-of-lexicon proper noun, every occurrence of a form that the document never writes in lowercase, that it capitalises at least once in mid-sentence, and whose dictionary form is not one of the pack's dictionary words (lingua-data-packs, *A pack's dictionary words do not depend on its glosses*). A dictionary word, a form also written in lowercase, a form capitalised only at the head of sentences, and a word whose lemma the reader has marked SHALL keep their classification. What is set aside SHALL NOT depend on the pack's glosses.

#### Scenario: A character of a novel
- **WHEN** a section reads `Augusto miró a Eugenia` and `Entonces Augusto salió`, and neither `augusto` nor `eugenia` is a dictionary word of the pack
- **THEN** every occurrence of `Augusto` and `Eugenia` is set aside, is not underlined, and does not count in the percentage

#### Scenario: The same form as a word
- **WHEN** the same document also writes `el augusto monarca`
- **THEN** `Augusto` stays a word

#### Scenario: A name with a gloss
- **WHEN** a document capitalises `Dios` in mid-sentence, and the es-fr pack glosses it by a common noun's sense, so that `dios` is one of its dictionary words
- **THEN** `Dios` stays a word, with its card

#### Scenario: A name glossed as a name
- **WHEN** a document reads « Cuando Pedro llegó a Sevilla, María ya se había ido a Madrid. », and es-fr glosses `pedro`, `sevilla`, `maría` and `madrid` by proper nouns' senses alone, so that none of them is a dictionary word
- **THEN** `Pedro`, `Sevilla`, `María` and `Madrid` are set aside, and a selection of one of them still shows its gloss

#### Scenario: Capitals at the head of sentences only
- **WHEN** a document writes `Augusto calla. Augusto mira el jardín.`
- **THEN** `Augusto` stays a word

#### Scenario: A pack glossed in English
- **WHEN** the section of *A character of a novel* is analysed with a pack built from the es-fr studied tables whose English glosses gloss `augusto` and `eugenia`, and whose dictionary words are es-fr's
- **THEN** every occurrence of `Augusto` and `Eugenia` is set aside, as with the es-fr pack

### Requirement: A French document's names are set aside
The page analysis of a French document SHALL set aside, like an out-of-lexicon proper noun, every occurrence of a form — a word, or a hyphenated run the pack does not list whole — that the document never writes in lowercase, that it capitalises at least once in mid-sentence, and whose dictionary form is not one of the pack's dictionary words (lingua-data-packs, *A pack's dictionary words do not depend on its glosses*); a capital right after an elided piece (`l'`, `d'`, `qu'`…) SHALL count as one in mid-sentence. A dictionary word, a form also written in lowercase, a form capitalised only at the head of sentences, and a word whose lemma the reader has marked SHALL keep their classification. What is set aside SHALL NOT depend on the pack's glosses, and French's readings SHALL NOT change Spanish's or English's names rule.

#### Scenario: A city
- **WHEN** a document reads « Nous partons de Paris. Paris est loin. », and `paris` is a lemma of the pack but not one of its dictionary words
- **THEN** both occurrences of `Paris` are set aside, are not underlined, and do not count in the percentage

#### Scenario: After an elided piece
- **WHEN** a document's only capitalised `Aube` follows `l'` (« l'Aube rejoint la Seine »), and `aube` is a lemma of the pack but not one of its dictionary words
- **THEN** `Aube` is set aside

#### Scenario: A hyphenated name
- **WHEN** a document reads « ont retrouvé Jean-Pierre à Saint-Étienne », and the pack lists `pierre` and `saint` but neither run
- **THEN** `Jean-Pierre` and `Saint-Étienne` are set aside

#### Scenario: A dictionary word
- **WHEN** a document reads « entre Orange et Vienne », `orange` is a dictionary word of the pack, and the pack lists `vienne` as a form of the dictionary word `venir`
- **THEN** `Orange` and `Vienne` stay words, with their cards

#### Scenario: Capitals at the head of sentences only
- **WHEN** a document writes `Mme` only at the head of its blocks
- **THEN** `Mme` stays a word

#### Scenario: The same form as a word
- **WHEN** a document writes « le Lot » in mid-sentence and also « un lot de livres »
- **THEN** `Lot` stays a word

#### Scenario: Spanish's rule does not change
- **WHEN** a Spanish document is analysed
- **THEN** a Spanish capital after an apostrophe still gives no evidence and a Spanish hyphenated run is still judged by its parts
