## ADDED Requirements

### Requirement: es-en's glosses read a Spanish word's meanings, not its page's layout
The es-en reduction SHALL read a sense the English Wiktionary nests under a sense-group label, a list's introduction or a pointer by the sense's own gloss, SHALL read a shortened or respelled form — an apocope, an apheresis, a syncope, a prepositional form, a pronunciation or eye-dialect spelling — as the meaning its pointer carries or, when it carries none, as its target's senses in the same part of speech, in its place and even when the word has other senses, and SHALL NOT let a proper noun written with a capital open the gloss of a lower-case preposition, conjunction, pronoun, determiner or article.
A parent that is neither a label nor a pointer keeps glossing its nested senses as before, and a form whose
pointer neither carries a meaning nor names a word with one is a pointer, never a meaning. These are
rules of the English Wiktionary's edition, which only es-en reads: they re-pin es-en alone, and its
coverage stays at least its floor.

#### Scenario: Sense-group labels
- **WHEN** the English Wiktionary nests « to come » and « to arrive » under « Senses relating to literal movement. », and venir's other senses under « Figurative senses. »
- **THEN** venir's gloss opens on « to come (…) » and holds neither label

#### Scenario: A list's introduction
- **WHEN** the senses of `cusco` are nested under « places in Peru: »
- **THEN** its gloss holds « a region of Peru » and the places that follow, not « places in Peru »

#### Scenario: A parent that is a meaning
- **WHEN** hacer's « to create, to build, to bring forth » is nested under « to make », which is neither a label nor a pointer
- **THEN** hacer's gloss holds « to make », as before

#### Scenario: An apocope that carries its meaning
- **WHEN** the English Wiktionary glosses `mi` « apocopic form of mío, my » beside the Greek letter mu and the note mi, and `muy` only « apocopic form of mucho; very »
- **THEN** mi's gloss opens on « my », and muy's is « very », not mucho's senses

#### Scenario: An apocope that carries none
- **WHEN** `su` is glossed « apocopic form of suyo », with a sense nested under it
- **THEN** su's gloss opens on suyo's senses as a determiner, holds the nested sense by its own gloss, and holds no « apocopic form of »

#### Scenario: A function word spelled like a place
- **WHEN** `como` has entries as an adverb, a conjunction and a preposition, and `Como`, the Italian city, is a proper noun written with a capital
- **THEN** como's gloss opens on « as (…) », and the city's senses, when the eight senses still hold them, come after every other

#### Scenario: Nothing else moves
- **WHEN** es-en is reduced again from its pinned sources with these rules
- **THEN** en-fr's, es-fr's and en-es's tables and pins are byte for byte unchanged, es-en's pin keeps its snapshot, and its coverage of the 5,000, 10,000 and 20,000 commonest lemmas is at least its floor

### Requirement: es-en's glosses are written in one English typography
The es-en reduction SHALL write a sense that opens on the edition's own description — an article, a determiner, « Used », « Said », « Indicates », « Related », « Pertaining », « To » and the like, followed by a word — in lower case, as the edition writes nearly all of its senses, unless « The » opens a title or a name; SHALL write an ellipsis as the one character « … », spaced on both sides between two words; SHALL pair a sense's straight double quotes as “ ”; and SHALL keep no reference to a numbered sense of the source page and no text after a line break.
No other character of a sense SHALL change: a proper adjective, a month, a language or a name keeps
its capital, a sense with an odd number of straight quotes keeps them, and single quotes stay as
written. The rules hold for the glosses of words and of expressions alike.

#### Scenario: The edition's description in lower case
- **WHEN** the English Wiktionary glosses `ni` « Used when negating two or more elements, … » and `se` « A reflexive or reciprocal pronoun: … »
- **THEN** their glosses read « used when negating two or more elements, … » and « a reflexive or reciprocal pronoun: … »

#### Scenario: A capital that is no description
- **WHEN** a sense reads « The Nutcracker (ballet) », « A (highest grade in testing) » or « July »
- **THEN** it keeps its capital

#### Scenario: One ellipsis
- **WHEN** `nada`, `tanto` and `o` are glossed « not...anything », « both ... and » and « either … or »
- **THEN** their glosses read « not … anything », « both … and » and « either … or »

#### Scenario: Paired quotes
- **WHEN** `otro` is glossed « "Not again!" or "What, again?" (also Otra vez! or Otra vez?) »
- **THEN** its gloss reads « “Not again!” or “What, again?” (also Otra vez! or Otra vez?) »

#### Scenario: A numbered sense of the source
- **WHEN** `ya` is glossed « indicates completion of an action (difference from sense 4 depends on context) » and `jurado` « juror, juryman, juryperson (member of a jury [sense 1]) »
- **THEN** they read « indicates completion of an action » and « juror, juryman, juryperson (member of a jury) »

#### Scenario: An example after a line break
- **WHEN** a sense of `canino` reads « ravenously hungry; hungry as a hog », then a line break and a Spanish example sentence with its translation
- **THEN** its gloss holds « ravenously hungry, hungry as a hog » and no part of the example
