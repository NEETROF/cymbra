# Spec Delta

## ADDED Requirements

### Requirement: Italian's word grammar
Italian's studied tables SHALL carry, in `tables/it/grammar.tsv`, the readings of the forms `tables/it/forms.tsv` holds under the lemmas `tables/it/freq.tsv` ranks, reduced by it-fr from the English Wiktionary's Italian section and written as Universal Dependencies tags, each table form read as change 9 reads it — its stress marks dropped, a reflexive row without its pronoun: a verb form's mood, tense, person and number — the passato remoto as the indicative past, the imperfetto as the imperfect, the conditional and the imperative with no tense —, its infinitive, its gerund, its present participle, or its past participle with its gender and number; a noun's gender on its own form and on its plural, and both numbers on a noun the dictionary gives no plural for; an adjective's, a determiner's, a pronoun's or a numeral's gender and number. A form that is a form of another form along one part of speech SHALL read as the word the forms table reaches through it, with its own agreement, and a reading's part of speech SHALL be one the dictionary holds its lemma as, when it holds that lemma as a noun, a verb, an adjective, a determiner, a pronoun or a numeral. Each reading SHALL be stored as its own tag, never merged with another, so that a present such as `parli` carries its four readings — the present indicative of the second person singular and the present subjunctive of the first, second and third — and the word card can merge them. A reading SHALL NOT come from a negative or a formal imperative row, from a compound tense, from an inflection or a sense the dictionary marks doubtful, from a form's own sense it marks regional or of a register, from a capitalised headword, from an entry that is only an alternative form of another word, from a defective verb's empty cell, or from a link toward the word a reviewed override row of the forms table sets aside, and a feminine noun's masculine SHALL NOT read as its inflection. A reading of another dictionary form the pack keeps SHALL be marked so that the card names it, only when that word is an entry of the dictionary that is not only regional; the form SHALL keep the one lemma its forms table gives it.

#### Scenario: A verb form says what it is
- **WHEN** the card asks the grammar of `parlò` as *parlare*, and of `parlavo`
- **THEN** `parlò` is the passato remoto, third person singular (`VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin`), and `parlavo` the imperfetto, first person singular (`…|Tense=Imp|…`)

#### Scenario: A present of four readings
- **WHEN** the card asks the grammar of `parli` as *parlare*
- **THEN** it answers four readings, each its own tag: `VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin`, and `VERB|Mood=Sub|Number=Sing|Tense=Pres|VerbForm=Fin` with `Person=1`, `Person=2` and `Person=3`

#### Scenario: The conditional, the imperative, the gerund and the participles
- **WHEN** the card asks the grammar of `parlerebbe`, `parla`, `parlando`, `parlante` and `parlata`
- **THEN** `parlerebbe` is `VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin`, with no tense; `parla` carries the present indicative third singular and the imperative second singular, with no tense; `parlando` is `VERB|VerbForm=Ger`; `parlante` is `VERB|Tense=Pres|VerbForm=Part`; and `parlata`, where the forms table files it under *parlare*, is `VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part`

#### Scenario: The table's stress marks and a reflexive row
- **WHEN** *dare*'s table writes `dièdi` and *lavarsi*'s writes `mi làvo`, *lavare* being an entry
- **THEN** `diedi` carries *dare*'s passato remoto, first person singular, and `lavo` *lavare*'s present indicative, first person singular

#### Scenario: Rows that give no reading
- **WHEN** a table lists the negative imperative `non parlàre` (written `parlàre` in the row), the formal imperative `parli` (Lei) and a defective verb's cell `-`
- **THEN** `parlare` reads only as the infinitive, `parli` carries no imperative reading, and `-` is no form

#### Scenario: A noun says its gender
- **WHEN** the card asks the grammar of `case` as *casa*, and of `città` as *città*
- **THEN** `case` is a feminine plural noun (`NOUN|Gender=Fem|Number=Plur`), and `città` a feminine noun both singular and plural

#### Scenario: Adjectives of four and of two forms
- **WHEN** the card asks the grammar of `rosse` as *rosso* and of `grandi` as *grande*
- **THEN** `rosse` is `ADJ|Gender=Fem|Number=Plur`, and `grandi` is `ADJ|Number=Plur`, with no gender

#### Scenario: A noun the forms table reads as a verb
- **WHEN** the forms table maps `porta` to *portare*, so that the noun *porta* is no lemma of the tables
- **THEN** the card of `porta` as *portare* answers its verb readings, and no reading names the noun

#### Scenario: What gives no reading
- **WHEN** the tables are reduced, the dictionary lists an archaic alternative form with a conjugation table, a capitalised headword, and `attore` as the masculine of *attrice*
- **THEN** no reading comes from the alternative form or the capitalised headword, and `attore` is no form of *attrice*

### Requirement: Italian's pinned tag pool
`tables/it/tags.tsv`, Italian's pinned tag pool, SHALL hold every tag Italian's readings carry, each once, one per line, and no other tag — the first reduction's in byte order, replacing the placeholder the forms tables pinned before any Italian pack shipped, a tag a later reduction adds appended after them, so that a pinned tag never changes index; it SHALL be written by a person and kept by every reduction, and the checks SHALL fail, naming the tag, when it holds a tag no reading carries or misses one a reading carries. Every base studying Italian SHALL lay its tag pool out as that pin, then any reading tag the pin lacks, and nothing of a pair's senses, so that it-fr, it-en and it-es share one base.

#### Scenario: The pin is the readings' tags
- **WHEN** Italian's readings are committed
- **THEN** every tag of `tables/it/grammar.tsv` is a line of `tables/it/tags.tsv`, every line of it is a tag of `grammar.tsv` written in canonical form, each once, and its lines are in byte order

#### Scenario: A later reduction carries a new tag
- **WHEN** it-fr is reduced from a later dump whose readings carry a tag the pin does not hold
- **THEN** the checks fail and name the tag until it is appended to `tables/it/tags.tsv`, after the tags already pinned

#### Scenario: A reduction keeps the pin
- **WHEN** it-fr is reduced again from its pinned sources
- **THEN** `tables/it/tags.tsv` is byte for byte as committed, and no reducer writes it

### Requirement: Italian readings are measured on held-out treebanks
The pipeline SHALL report, for Italian's committed tables, on UD Italian-PUD and on MarkIT's test section — fetched at the commits Italian's forms are measured at and never read by the reduction —, for each part of speech, the share of the words whose form the tables map to the treebank's lemma that carry a reading, and the share of those whose treebank part of speech and features are among their readings, the conditional's and the imperative's tense left aside. The report SHALL NOT decide the pipeline's exit status.

#### Scenario: The readings' figures are reported
- **WHEN** the Italian measurement runs over the committed tables
- **THEN** it prints, beside the forms' gates, the readings' figures for verbs (finite, participles, infinitives, gerunds), nouns, adjectives, determiners and pronouns on both treebanks, and they do not decide its exit status
