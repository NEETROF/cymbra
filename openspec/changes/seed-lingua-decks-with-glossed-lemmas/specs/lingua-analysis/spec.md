## MODIFIED Requirements

### Requirement: An analysis does not depend on the native language
Two packs of one studied language built from the same studied tables SHALL give, glosses and their senses aside, the same page analysis, the same readings and other dictionary forms of a word, the same levels, ladder and vocabulary estimate, whatever native language each is glossed in.
The cards a level seeds SHALL be the one part of a reader's history that follows the native
language: each pack seeds that level's lemmas, in the order asked, that it glosses (*Level-targeted
deck seeding*), and nothing else differs between the two.
A test SHALL build, for English, for Spanish and for French, a pack from the first pack's studied
tables glossed in another native language, with other glosses, and SHALL compare every probe of that
language's invariance baseline, answered without the reader's level seeding, with its glosses and
senses removed; and SHALL check, through both packs, for every level and both orders, that a seeding
takes that level's lemmas, in that order, that the pack glosses.

#### Scenario: English through another native language
- **WHEN** the English invariance baseline, without the reader's level seeding, is answered with the en-fr pack and with a pack built from its studied tables, glossed in Spanish with fewer glosses
- **THEN** every probe is byte for byte alike once glosses and senses are removed

#### Scenario: Spanish through another native language
- **WHEN** the Spanish invariance baseline, without the reader's level seeding, is answered with the es-fr pack and with a pack built from its studied tables, glossed in English with glosses for `augusto` and `eugenia`
- **THEN** every probe is byte for byte alike once glosses and senses are removed, every token keeping its class and every page its percentage

#### Scenario: French through another native language
- **WHEN** the French invariance baseline, without the reader's level seeding, is answered with the fr-en pack and with the fr-es pack, both built from French's committed tables, the second glossed in Spanish with fewer glosses
- **THEN** every probe is byte for byte alike once glosses and senses are removed, the two packs' studied sections are byte for byte alike (the tag pool up to its pinned prefix), and a lemma fr-es glosses that fr-en does not is no dictionary word

#### Scenario: Seeded cards through another native language
- **WHEN** each level is seeded, commonest first and rarest first, through the en-fr pack and through the en-es pack, then through the fr-en pack and through the fr-es pack
- **THEN** each pack creates its cards from that level's lemmas, in that order, that it glosses, as many as asked — the C1 lemmas en-fr seeds rarest first are not en-es's, and `part`, at A1, is seeded through fr-en and not through fr-es — while the two packs' levels, ladders and vocabulary estimates stay alike

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after this change
- **THEN** every probe before the reader's level seeding is byte for byte the one recorded before; the Spanish baselines (es-fr, es-en) move nowhere, and the English ones (en-fr, en-es) only on the probes that follow the seeding of C1's rarest lemmas, which those packs do not gloss
