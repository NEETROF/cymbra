## MODIFIED Requirements

### Requirement: A vocabulary size counts dictionary words
The reader's estimated vocabulary and a level's typical vocabulary SHALL be counted over the ranked lemmas that are dictionary words of the pack (lingua-data-packs, *A pack's dictionary words do not depend on its glosses*) or carry a CEFR level, so that neither depends on the native language the pack is glossed in.
A ranked lemma that is neither, mostly a name or noise, SHALL count in no vocabulary size: a name its
studied language's reference pair glosses by a proper noun's senses alone (`london`, `madrid`) counts
in none, and a name a CEFR list levels (`june`) counts as a levelled lemma.

#### Scenario: English
- **WHEN** the vocabulary estimate and the ladder are asked of an engine holding the en-fr pack
- **THEN** the estimate's universe is 22,337 words, and B1's typical vocabulary is 3,074

#### Scenario: English glossed in another language
- **WHEN** they are asked of an engine holding a pack built from the en-fr studied tables, glossed in Spanish with fewer glosses
- **THEN** the universe is still 22,337 words, and B1's typical vocabulary still 3,074

#### Scenario: Spanish glossed in another language
- **WHEN** the vocabulary estimate is asked of an engine holding a pack built from the es-fr studied tables, glossed in English with glosses for more lemmas than es-fr's
- **THEN** its universe is 21,062 words, es-fr's
