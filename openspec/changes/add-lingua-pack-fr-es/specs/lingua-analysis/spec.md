## MODIFIED Requirements

### Requirement: An analysis does not depend on the native language
Two packs of one studied language built from the same studied tables SHALL give, glosses and their senses aside, the same page analysis, the same readings and other dictionary forms of a word, the same levels, ladder and vocabulary estimate, whatever native language each is glossed in.
A test SHALL build, for English, for Spanish and for French, a pack from the first pack's studied
tables glossed in another native language, with other glosses, and SHALL compare every probe of that
language's invariance baseline with its glosses and senses removed.

#### Scenario: English through another native language
- **WHEN** the English invariance baseline is answered with the en-fr pack and with a pack built from its studied tables, glossed in Spanish with fewer glosses
- **THEN** every probe is byte for byte alike once glosses and senses are removed

#### Scenario: Spanish through another native language
- **WHEN** the Spanish invariance baseline is answered with the es-fr pack and with a pack built from its studied tables, glossed in English with glosses for `augusto` and `eugenia`
- **THEN** every probe is byte for byte alike once glosses and senses are removed, every token keeping its class and every page its percentage

#### Scenario: French through another native language
- **WHEN** the French invariance baseline is answered with the fr-en pack and with the fr-es pack, both built from French's committed tables, the second glossed in Spanish with fewer glosses
- **THEN** every probe is byte for byte alike once glosses and senses are removed, the two packs' studied sections are byte for byte alike, and a lemma fr-es glosses that fr-en does not is no dictionary word

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after this change
- **THEN** every probe is byte for byte the one recorded before
