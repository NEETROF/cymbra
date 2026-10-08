## ADDED Requirements

### Requirement: en-es's glosses read an English word's meanings, not its page's notes
The en-es reduction SHALL take out of a sense the Spanish Wiktionary's notes to its readers — a maintenance template, a disambiguation note, a reference to its numbered senses, its expansion notice — and a usage note written after the meaning; SHALL NOT let the sense of a capitalised proper noun that only says a word is a surname or a given name gloss a word that has an entry of its own in lower case, other than a proper noun's, that holds a meaning; SHALL read an adjective section the edition tags possessive or demonstrative as a determiner; and SHALL write the senses the edition marks obsolete or outdated after the other senses of their entry.
A usage note is a sentence that opens, after the meaning's period, on one of a closed list of the
edition's openers (« A veces », « Precediendo », « Usado », « Utilizado », « Empleado », « Se usa »,
« Se dice », « Se emplea », « Se utiliza », « A no confundir », « Compárese » and their forms); a
second sentence that carries the meaning stays. A name's note is a sense of a proper noun's entry
glossed « Apellido », « Nombre de pila », « Nombre personal » or « Hipocorístico »: the proper noun's
other senses stay, and a word glossed only by names keeps its notes. Only the possessive and
demonstrative sections become determiners; the edition's other adjective sections stay adjectives.
Nothing is left out by the order: a word whose senses in an entry are all obsolete keeps them as
written. These are rules of the Spanish Wiktionary's edition and of en-es's reducer, which only en-es
reads: they re-pin en-es alone, and its coverage stays at least its floor.

#### Scenario: The edition's notes to its readers
- **WHEN** the Spanish Wiktionary glosses a sense of `stage` « Identificar la fase de un proceso.^([cita requerida]) », a sense of `favor` « Hacer un favor [sentido del sustantivo] para; mostrar beneficencia hacia », and `hardcore`'s noun and adjective « Hardcore (definiciones [1,2]) » and « Hardcore (definiciones [4,5]) »
- **THEN** those senses read « Identificar la fase de un proceso », « Hacer un favor para, mostrar beneficencia hacia » and « Hardcore », the last held once

#### Scenario: A usage note after the meaning
- **WHEN** the Spanish Wiktionary glosses `a` « Un, una. A veces se omite en la traducción. »
- **THEN** the gloss of `a` opens on « Un, una »

#### Scenario: A second sentence that carries the meaning
- **WHEN** the Spanish Wiktionary glosses `isn't` « Contracción de el verbo is y el adverbio not. Traducida como "no es" o "no está". »
- **THEN** its gloss keeps the second sentence

#### Scenario: A name on a common word's card
- **WHEN** the Spanish Wiktionary has `will` as a noun and a verb, and `Will`, a proper noun, glossed « Apellido » and « Hipocorístico de William »; and `smith` as a noun « Herrero », and `Smith` glossed « Apellido »
- **THEN** the gloss of `will` holds neither name's note, and the gloss of `smith` is « Herrero »

#### Scenario: A name's own row
- **WHEN** the Spanish Wiktionary has only `Wayne`, a proper noun glossed « Apellido », and no `wayne` in lower case
- **THEN** the gloss of `wayne` is « Apellido », as before

#### Scenario: Possessives and demonstratives
- **WHEN** the Spanish Wiktionary has `her` as a personal pronoun and as an adjective section tagged possessive glossed « Su (de ella) », and `that` as an adjective section tagged demonstrative
- **THEN** « Su (de ella) » is a determiner's sense of `her`, and `that`'s « Ese » and « Aquel » are a determiner's

#### Scenario: Current senses first
- **WHEN** the Spanish Wiktionary's first verb section of `go` opens on « Andar, marchar, caminar », tagged obsolete, before « Ir »
- **THEN** the gloss of `go` opens on « Ir », and « Andar, marchar, caminar » stays among `go`'s senses when the eight still hold it

#### Scenario: Nothing else moves
- **WHEN** en-es is reduced again from its pinned sources with these rules
- **THEN** en-fr's, es-fr's and es-en's tables and pins are byte for byte unchanged, en-es's pin keeps its snapshot, its studied record and its sources, and its coverage of the 5,000, 10,000 and 20,000 commonest lemmas is at least its floor

### Requirement: en-es's glosses are written in one Spanish typography
The en-es reduction SHALL write an ellipsis as the one character « … », spaced on both sides between two words; SHALL pair a sense's straight double quotes as « »; and SHALL name the English -ing form « forma en -ing » where a sense of an English entry names it « participio presente ».
An ellipsis that is not between two words keeps the spacing written. Quotes are paired in order, with
no space inside them, when a sense holds an even number of them; a sense holding an odd number keeps
them as written, and single quotes stay as written, since they are also the apostrophes of the
English words a sense names. No other character of a sense SHALL change. The rules hold for the
Spanish Wiktionary's senses and for the translation tables' words, for words and for expressions
alike.

#### Scenario: One ellipsis
- **WHEN** the Spanish Wiktionary glosses a sense of `on` « Relacionado con el tema ..., sobre el tema ... » and a sense of `neither` « (neither ... nor) Ni »
- **THEN** they read « Relacionado con el tema …, sobre el tema … » and « (neither … nor) Ni »

#### Scenario: Angular quotes
- **WHEN** the Spanish Wiktionary glosses `it's` « Contracción de el pronombre it ("ello") y el verbo is ("es") »
- **THEN** its gloss reads « Contracción de el pronombre it («ello») y el verbo is («es») »

#### Scenario: Single quotes
- **WHEN** the Spanish Wiktionary glosses `that's` « Contracción de 'that is', ese es, esa es, aquel es, aquella es »
- **THEN** its gloss keeps the single quotes as written

#### Scenario: The -ing form named as the card names it
- **WHEN** the Spanish Wiktionary glosses a sense of `be` « Estar (be + participio presente) »
- **THEN** that sense reads « Estar (be + forma en -ing) »

### Requirement: en-es's translation-table glosses hold the translators' words, not their notes
The en-es reduction SHALL NOT gloss with a Spanish word that the English Wiktionary's translation table labels disused; SHALL take out of a translation the note its translator wrote inside it — a loanword's respelling, a label holding no Spanish word, a sense number, an English usage note — and keep the word; and SHALL list a Spanish word once for an English word that the Spanish Wiktionary's translations give it, read backwards, under the first of the Spanish word's parts of speech that the English word's readings name, or else the first listed.
A respelling is a note that follows the English headword, or one of its words, written as the
translation (« hall (hol) »). A label holds no word that wordfreq's Spanish list rates at 1.0 Zipf or
more; a usage note opens on « with »; a note holding a Spanish word stays. A word whose only
translations are labelled disused takes no gloss from that table. A word the English Wiktionary
translates under several of its own parts of speech keeps each: those are the English word's. The
English word's readings are the studied tables the pin records.

#### Scenario: A disused word
- **WHEN** the English Wiktionary lists « orquesta » and « orquestra (disused) » for `orchestra`
- **THEN** the gloss of `orchestra` is « Orquesta »

#### Scenario: A translator's note
- **WHEN** the English Wiktionary lists « pasillo », « hall (hol) » and « jol » for `hall`, and « pueblucho », « villorrio (despective) » and « poblacho (despective) » for `backwater`
- **THEN** the gloss of `hall` reads « Pasillo, hall, jol » and that of `backwater` « Pueblucho, villorrio, poblacho »

#### Scenario: A Spanish note stays
- **WHEN** the English Wiktionary lists « guardería (infantil) » for `daycare`
- **THEN** the gloss of `daycare` keeps « (infantil) »

#### Scenario: A Spanish word listed once
- **WHEN** the Spanish Wiktionary's `largo` lists `lengthy` as its translation as an adjective, a noun, an interjection and a verb, and its `marinero` lists `seaman` as an adjective and a noun, and the English readings name `seaman` a noun
- **THEN** the gloss of `lengthy` is « Largo », once, and the gloss of `seaman` is « Marinero » under a noun

#### Scenario: The English word's own parts of speech
- **WHEN** the English Wiktionary lists « israelí » for `israeli` as a noun and as an adjective
- **THEN** the gloss of `israeli` holds « Israelí » under each, as before
