## ADDED Requirements

### Requirement: es-en's glosses read a Spanish word's meanings, not its page's layout
The es-en reduction SHALL read a sense the English Wiktionary nests under a sense-group label, a list's introduction or a pointer by the sense's own gloss, SHALL read a shortened or respelled form — a sense worded as an apocope, an apheresis, a syncope, a prepositional form, or a pronunciation or eye-dialect spelling — as the meaning its pointer carries or, when it carries none, as its target's senses in the same part of speech, in its place and even when the word has other senses, SHALL read a pronoun's case form whose pointer carries its meaning after a colon or a semicolon as that meaning, and SHALL NOT let a proper noun whose headword has an initial capital, and is not written all in capitals, open the gloss of a lower-case preposition, conjunction, pronoun, determiner or article.
A parent that is neither a label nor a pointer keeps glossing its nested senses as before. A form is
known by its pointer's wording, never by a tag alone: a sense tagged apocopic or prepositional and
worded as a meaning stays a meaning. A form's target is the word its pointer names, whole, and its
meaning what the pointer records for that word or writes after it. A form whose pointer neither
carries a meaning nor names a word of three letters or more with one in its part of speech is a
pointer, never a meaning: the bound of *A word that is only a form of another takes that word's
gloss* holds for these forms too. A headword written all in capitals is an acronym's, not a place's,
and its lines keep their place. These are rules of the English Wiktionary's edition, which only
es-en reads: they re-pin es-en alone, and its coverage stays at least its floor.

#### Scenario: Sense-group labels
- **WHEN** the English Wiktionary nests « to come » and « to arrive » under « Senses relating to literal movement. », and venir's other senses under « Figurative senses. »
- **THEN** venir's gloss opens on « to come (…) » and holds neither label

#### Scenario: A list's introduction
- **WHEN** the senses of `cusco` are nested under « places in Peru: »
- **THEN** its gloss holds « a region of Peru » and the places that follow, not « places in Peru »

#### Scenario: A parent that is a meaning
- **WHEN** hacer's « to create, to build, to bring forth » is nested under « to make », which is neither a label nor a pointer
- **THEN** hacer's gloss holds « to make », as before

#### Scenario: A meaning nested under a diminutive
- **WHEN** `casita`'s « small house » and « house » are nested under « diminutive of casa », and every sense of `cosita` under « diminutive of cosa »
- **THEN** casita's gloss opens on « small house », and cosita, unglossed before, is glossed « small thing; thingy »

#### Scenario: An apocope that carries its meaning
- **WHEN** the English Wiktionary glosses `mi` « apocopic form of mío, my » beside the Greek letter mu and the note mi, and `muy` only « apocopic form of mucho; very »
- **THEN** mi's gloss opens on « my », and muy's is « very », not mucho's senses

#### Scenario: A form named by its pointer's fields
- **WHEN** the adjective `mal` is glossed « apocopic form of malo bad; evil », its pointer naming « malo bad » with the meaning « evil », and the expression `cincuenta y un` « apocopic form of cincuenta y uno (“fifty-one”) »
- **THEN** mal's gloss holds « evil » and no « apocopic form of », and cincuenta y un's gloss is « fifty-one », not « fifty »

#### Scenario: A tag is no pointer
- **WHEN** mal's adjective « amiss, awry, off, wrong » is tagged apocopic, and nos's « first person nominative, prepositional and vocative plural pronoun » prepositional, neither worded as a pointer
- **THEN** both stay in their glosses as meanings

#### Scenario: An apocope that carries none
- **WHEN** `su` is glossed « apocopic form of suyo », with a sense nested under it
- **THEN** su's gloss opens on suyo's senses as a determiner, holds the nested sense by its own gloss, and holds no « apocopic form of »

#### Scenario: A target of two letters
- **WHEN** `er` is glossed only « pronunciation spelling of el » and « eye dialect spelling of el »
- **THEN** er has no gloss, as before

#### Scenario: A pronoun's case form
- **WHEN** `lo` is glossed « accusative of él and usted (when referring to a man), and a variant of ello in many constructions; him, you (formal), it, that » beside its article, and `les` « dative of ellos and ellas; to them, for them » beside « the (plural) »
- **THEN** lo's gloss opens on « him, you (formal), it, that », and les's on « to them, for them », before « the (plural) »

#### Scenario: A function word spelled like a place
- **WHEN** `como` has entries as an adverb, a conjunction and a preposition, and `Como`, the Italian city, is a proper noun written with an initial capital
- **THEN** como's gloss opens on « as (…) », and the city's senses, when the eight senses still hold them, come after every other

#### Scenario: An acronym keeps its place
- **WHEN** `pr` is only an abbreviation of por, para and pero, a preposition and a conjunction, and `PR`, written all in capitals, an initialism of Puerto Rico
- **THEN** pr's gloss is « Puerto Rico (…) », as before

#### Scenario: Nothing else moves
- **WHEN** es-en is reduced again from its pinned sources with these rules
- **THEN** en-fr's, es-fr's and en-es's tables and pins are byte for byte unchanged, es-en's pin keeps its snapshot, its studied record and its sources, and its coverage of the 5,000, 10,000 and 20,000 commonest lemmas is at least its floor

### Requirement: es-en's glosses are written in one English typography
The es-en reduction SHALL write a sense that opens on one of the edition's description openers, followed by a space and a letter, in lower case, unless « The » precedes a capital; SHALL write an ellipsis as the one character « … », spaced on both sides between two words; SHALL pair a sense's straight double quotes as “ ”; and SHALL keep no reference to a numbered sense of the source page and no text after a line break.
The openers are a closed list: « A », « An », « The », « Any », « One », « Some », « Certain »,
« Various », « Either », « Used », « Said », « Indicate », « Indicates », « Expresses », « Denotes »,
« Forms », « Replaces », « Introduces », « Refers », « Related », « Relating », « Pertaining », « Of »,
« Having », « In », « To », « Someone », « Something », « Term », « Expression » and « Interjection ».
« The » before a capital stays, whether it opens a title, a name or a species. An ellipsis that is
not between two words keeps the spacing written. A bracketed sense number goes first, then a
parenthesis still naming a numbered sense goes whole, so a parenthesis that says more stays. No other
character of a sense SHALL change: a proper adjective, a month, a language or a name keeps its
capital, a sense with an odd number of straight quotes keeps them, and single quotes stay as
written. The rules hold for the glosses of words and of expressions alike.

#### Scenario: The edition's description in lower case
- **WHEN** the English Wiktionary glosses `ni` « Used when negating two or more elements, … » and `se` « A reflexive or reciprocal pronoun: … »
- **THEN** their glosses read « used when negating two or more elements, … » and « a reflexive or reciprocal pronoun: … »

#### Scenario: A capital that is no description
- **WHEN** a sense reads « The Nutcracker (ballet) », « The Eurasian treecreeper », « A (highest grade in testing) » or « July »
- **THEN** it keeps its capital

#### Scenario: One ellipsis
- **WHEN** `nada`, `tanto`, `o` and `ahora` are glossed « not...anything », « both ... and », « either … or » and « whether...or... »
- **THEN** their glosses read « not … anything », « both … and », « either … or » and « whether … or… »

#### Scenario: Paired quotes
- **WHEN** `otro` is glossed « "Not again!" or "What, again?" (also Otra vez! or Otra vez?) »
- **THEN** its gloss reads « “Not again!” or “What, again?” (also Otra vez! or Otra vez?) »

#### Scenario: A numbered sense of the source
- **WHEN** `ya` is glossed « indicates completion of an action (difference from sense 4 depends on context) », and `jurado` « juror, juryman, juryperson (member of a jury [sense 1]) » and « judge (member of a jury [sense 2]; officiator of a competitive event) »
- **THEN** they read « indicates completion of an action », « juror, juryman, juryperson (member of a jury) » and « judge (member of a jury, officiator of a competitive event) »

#### Scenario: An example after a line break
- **WHEN** a sense of `canino` reads « ravenously hungry; hungry as a hog », then a line break and a Spanish example sentence with its translation
- **THEN** its gloss holds « ravenously hungry, hungry as a hog » and no part of the example
