## ADDED Requirements

### Requirement: en-it's glosses open on the Italian word of the English word's commonest use
en-it's rows SHALL open on the sense of an English word's commonest use, by rules of en-it's own modules, which no other pair loads. A row the Italian Wiktionary's English section glosses, of two parts of speech or more, SHALL open on the part of speech the English Wiktionary's Italian table lists first for the word when that part of speech's words, in either translation table, are also the ones Italian writes most — at least 0.5 Zipf above the row's first part of speech's on wordfreq's Italian list —, its senses moved together and the others following in their order, and a verb SHALL NOT yield to a noun. When that part of speech is missing from the row, or the row's first sense describes the word rather than translating it and no sense of the row shares a word with either table, the row SHALL open on the table's first sense: at most three Italian words, none English, for a lemma of three letters or more. A reviewed table (`FIRST_RUN`) SHALL name, for the commonest grammatical words and the rows these rules cannot tell apart, the sense a row opens on — a sense the section writes, or the words a table gives —, each with its reason and source, failing its test when the section no longer writes the sense it names.

#### Scenario: The part of speech two signals agree on
- **WHEN** the section glosses `make` « marca; fare, creare, fabbricare; … », the English Wiktionary lists its verb first, and « fare » rates above « marca »
- **THEN** en-it's row reads « fare, creare, fabbricare; rendere; …; marca »

#### Scenario: A verb does not yield to a noun
- **WHEN** a row opens on a verb and the table's noun words rate higher (`check` « assegno »)
- **THEN** the row's order is unchanged

#### Scenario: The missing main sense
- **WHEN** the section glosses `prove` « prove » and the English Wiktionary's table lists « dimostrare, provare » under the verb it lists first
- **THEN** en-it's row opens on « dimostrare, provare »

#### Scenario: A row of descriptions
- **WHEN** the section glosses `its` « possessivo di it » alone and the table's first sense is « suo »
- **THEN** en-it's row reads « suo; possessivo di it »

#### Scenario: A grammatical word, reviewed
- **WHEN** the section glosses `that` « così; introduce una proposizione…; quello, quella; che, il quale… »
- **THEN** en-it's row opens on « quello, quella; che, il quale, la quale, i quali, le quali », as `FIRST_RUN` names it

#### Scenario: A reviewed sense no longer written
- **WHEN** `FIRST_RUN` names a sense of the section that the section no longer writes
- **THEN** the reducer's tests fail, naming the lemma

### Requirement: en-it's glosses hold Italian words, without frames, notes, pointers, heraldry or repeats
en-it SHALL take out of its rows what is not a translation, when the word keeps a sense that is: a frame before the translation (« forma possessiva di I usata come aggettivo possessivo, … », « in funzione di soggetto: … ») when what follows is items of three words or fewer, none opening on a preposition; a proper noun's note on a word with a common entry; a pointer's wording (« vedi », « si veda », « cfr », « più su », « variante di »); a heraldic sense, wherever it stands, and a tincture's; an abbreviation's English expansion, unless the word keeps no sense holding an Italian word or the expansions are four or more; a wiki's text; a state's code on a word another sense glosses; a sense every item of which an earlier sense holds; an item holding « ə ». A given name's note naming its Italian equivalent SHALL read as that name, short (« Giovanni (nome) »), and a name alone SHALL keep its note. No rule SHALL take a lemma's only gloss.

#### Scenario: A frame
- **WHEN** the section glosses `my` « forma possessiva di I usata come aggettivo possessivo, il mio, la mia, i miei, le mie »
- **THEN** en-it's row reads « il mio, la mia, i miei, le mie »

#### Scenario: A frame kept
- **WHEN** what follows a frame opens on a preposition (`audio` « relativo all'ascolto, alla riproduzione sonora »)
- **THEN** the sense is unchanged

#### Scenario: A name on a common word
- **WHEN** the section glosses `summer` « estate » and the proper noun *Summer* « nome proprio di persona inglese femminile »
- **THEN** en-it's row reads « estate »

#### Scenario: A given name
- **WHEN** the section glosses `john` « nome proprio di persona maschile, corrispondente all'italiano Giovanni »
- **THEN** en-it's row reads « Giovanni (nome) »

#### Scenario: Heraldry inside a row
- **WHEN** the section glosses `side` « lato; faccia di una medaglia; fianco: porzione di uno scudo…; allearsi; … »
- **THEN** en-it's row reads « lato; faccia di una medaglia; allearsi; schierarsi; sgrossare »

#### Scenario: An expansion
- **WHEN** the section glosses `uk` « United Kingdom : Regno Unito »
- **THEN** en-it's row reads « Regno Unito »

#### Scenario: A list of expansions
- **WHEN** every sense of `csi` is an English expansion
- **THEN** its row is unchanged

#### Scenario: A repeat
- **WHEN** a row reads `national` « nazionale; nazionale »
- **THEN** en-it's row reads « nazionale »

### Requirement: en-it corrects, by name, what its sources write wrong
A reviewed table (`CORRECTIONS`) SHALL correct, by lemma, a false friend's first place (`prove`, `disposable`, `editor`, `fabric`), a page a source copied from another word (`some`), English text where Italian is due (`media`) and a misspelling (« squaglia », « sù », « grupo »), each with its reason and source, naming itself when the source no longer writes what it corrects; its words SHALL be a source's, or be marked as written by a person and approved by the owner. A contraction no source glosses SHALL read as the pointer the section writes for the contractions it glosses — « contrazione di » and its full form —, and a contraction the section glosses through another's entry SHALL read its own entry's full form.

#### Scenario: A false friend
- **WHEN** the section glosses `disposable` « disponibile » and the English Wiktionary's table « monouso, usa e getta »
- **THEN** en-it's row reads « monouso, usa e getta »

#### Scenario: A copied page
- **WHEN** the section glosses `some` with `many`'s senses (« molti, tanti; la folla; molti, molte »)
- **THEN** en-it's row reads « qualche, alcuni, alcune; del, della »

#### Scenario: A correction that finds nothing
- **WHEN** a row of `CORRECTIONS` names a sense or an item the source no longer writes
- **THEN** the reduction's summary names it and the reducer's tests fail

#### Scenario: A contraction no source glosses
- **WHEN** no source glosses `you're`
- **THEN** en-it's row reads « contrazione di you are »

#### Scenario: A contraction read through another's entry
- **WHEN** the section's entry for `doesn't` points at `don't` and writes « contrazione di does not »
- **THEN** en-it's row reads « contrazione di does not »

#### Scenario: A gloss written by a person
- **WHEN** a row of `CORRECTIONS` holds a word no source gives
- **THEN** the row is marked as written by a person, and a test lists every such gloss

## MODIFIED Requirements

### Requirement: English is glossed in Italian from the Italian Wiktionary's English section and the two Wiktionaries' translation tables
en-it's glosses SHALL be reduced from the Italian Wiktionary's English section with the Italian edition's rules, then, for what that section leaves out, from the English Wiktionary's Italian translation tables, then from the Italian Wiktionary's English translations read backwards — words people wrote in Italian, at most three per part of speech from each table, with no pivot and no machine translation —, reading the committed English studied tables as a reader pair does and loading no other pair's reducer; the first source that glosses a lemma glosses it, but where en-it's own rules reorder a row's senses, open it on a table's words or take out what is not a translation (*en-it's glosses open on the Italian word of the English word's commonest use*, *en-it's glosses hold Italian words, without frames, notes, pointers, heraldry or repeats*), and a gloss a reviewed table writes SHALL be a source's words or be marked as written by a person. No translation table SHALL gloss a letter: a letter's entry, a one-letter word's noun entry and an entry whose every translation is its own letter are left out of both tables, and, read backwards, every one-letter English word. In the direct table a word labelled disused SHALL NOT be listed, and a translator's note inside a word — a sense number, a loanword's respelling, a label holding no word wordfreq's Italian list rates at 1.0 Zipf or more, a usage note opening on « with » — SHALL be taken out and the word kept. Read backwards, an Italian word SHALL be listed once per English word, under the first of its parts of speech English's readings name, or else the first listed, the words ordered by Italian frequency. Every table word SHALL be written in the Italian edition's typography, for words and expressions alike. Its expressions SHALL be keyed as en-es's are. The share of glossed lemmas among the 10,000 commonest whose gloss came from a translation table SHALL be measured and shown. The Italian edition's rules SHALL be read by en-it and by no other committed pair.

#### Scenario: A definition first
- **WHEN** the Italian Wiktionary's English section defines `summer` and a translation table also gives it Italian words
- **THEN** `summer` is glossed by the section's definition

#### Scenario: A word only a table glosses
- **WHEN** no sense of the section glosses an English lemma and the English Wiktionary's Italian table translates it
- **THEN** it is glossed by the table's Italian words, at most three per part of speech

#### Scenario: A section's row a table's words open
- **WHEN** the section glosses `can` « latta, lattina… » alone and the English Wiktionary's table lists « potere » under the verb it lists first
- **THEN** en-it's row opens on « potere », the section's senses after it

#### Scenario: Listed once
- **WHEN** the Italian Wiktionary's tables, read backwards, give `left-hand` « sinistro » under three parts of speech
- **THEN** « sinistro » is listed once

#### Scenario: A translator's note
- **WHEN** the English Wiktionary's Italian table gives `enrichment` « arricchimento (1,2) »
- **THEN** en-it's gloss reads « arricchimento »

#### Scenario: A rule of the Italian edition
- **WHEN** a rule of the Italian edition changes
- **THEN** en-it's rule digest moves, and no other committed pair's does

#### Scenario: A rule of en-it's own
- **WHEN** a rule of `reduce_english_it.py` changes
- **THEN** en-it's rule digest moves, and no other pair's does

#### Scenario: Nothing else moves
- **WHEN** en-it's tables are committed
- **THEN** every other pair's tables, pins, packs and goldens are unchanged
