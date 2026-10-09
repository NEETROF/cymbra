## ADDED Requirements

### Requirement: French's word grammar
French's studied tables SHALL carry, in `tables/fr/grammar.tsv`, the readings of the forms `tables/fr/forms.tsv` holds under the lemmas `tables/fr/freq.tsv` ranks, reduced by fr-en from the English Wiktionary's French section and written as Universal Dependencies tags: a verb form's mood, tense, person and number — the passé simple as the indicative past, the conditional and the imperative with no tense —, its infinitive, its present participle, or its past participle with its gender and number; a noun's gender on its own form and on its plural, and both numbers on a noun the dictionary gives one form for; an adjective's, a determiner's, a pronoun's or a numeral's gender and number. A form that is a form of another form along one part of speech SHALL read as the word the forms table reaches through it, with its own agreement. A reading's part of speech SHALL be one the dictionary holds its lemma as, when it holds that lemma as a noun, a verb, an adjective, a determiner, a pronoun or a numeral, so that a past participle the forms table files under a noun's spelling reads as its verb's, which the card names. A pronominal verb's forms SHALL read without their pronoun. Each reading SHALL be stored as its own tag, never merged with another, so that a present such as `parle` carries its five readings — the present indicative and the present subjunctive of the first and third persons singular, and the imperative of the second person singular — and the word card can merge them. A reading SHALL NOT come from an inflection or a sense the dictionary marks doubtful, from a form's own sense it marks regional or of a register, from a capitalised headword, from an entry that is only an alternative form of another word or a neologism, from a compound tense, from a letter's name toward its plural, or from a link toward the word a reviewed override row of the forms table sets aside as a copy error, and a feminine noun's masculine SHALL NOT read as its inflection. A reading of another dictionary form the pack keeps SHALL be marked so that the card names it, only when that word is an entry of the dictionary that is not only regional; the form SHALL keep the one lemma its forms table gives it.

#### Scenario: A verb form says what it is
- **WHEN** the card asks the grammar of `fut` as *être*
- **THEN** it answers the passé simple, third person singular (`VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin`)

#### Scenario: A present of five readings
- **WHEN** the card asks the grammar of `parle` as *parler*
- **THEN** it answers five readings, each its own tag: `VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin`, the same with `Person=3`, `VERB|Mood=Sub|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin`, the same with `Person=3`, and `VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin`

#### Scenario: The conditional and the participles
- **WHEN** the card asks the grammar of `parlerait`, `parlant` and `dirigée`
- **THEN** `parlerait` is `VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin`, with no tense; `parlant` is `VERB|Tense=Pres|VerbForm=Part`; and `dirigée`, which the dictionary gives as the feminine of the participle `dirigé`, is `VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part` of *diriger*

#### Scenario: A pronominal verb
- **WHEN** the dictionary writes *s'évanouir*'s forms with their pronoun — `s'évanouit`, `s'évanouissaient`, and the imperative `évanouis-toi`
- **THEN** `évanouit` carries its present indicative, third person singular, `évanouissaient` its imperfect indicative, third person plural, and `évanouis` the imperative, second person singular, among their readings

#### Scenario: A noun says its gender
- **WHEN** the card asks the grammar of `maisons` as *maison*, and of `temps` as *temps*
- **THEN** `maisons` is a feminine plural noun (`NOUN|Gender=Fem|Number=Plur`), and `temps` is a masculine noun both singular and plural

#### Scenario: A homograph names its other dictionary form
- **WHEN** the card asks the grammar of `couvent` as the noun *couvent*
- **THEN** it also names *couver*, whose present indicative and present subjunctive, third person plural, `couvent` is

#### Scenario: A participle filed under a noun's spelling
- **WHEN** the forms table maps `citée` to the noun *cité*, which the dictionary also gives as the past participle of *citer*
- **THEN** the card of `citée` as *cité* answers no verb reading of *cité*, and names *citer*, whose feminine past participle `citée` is

#### Scenario: A noun the forms table reads as a verb
- **WHEN** the forms table maps `porte` to *porter*, so that the noun *porte* is no lemma of the tables
- **THEN** the card of `porte` as *porter* answers its five verb readings, and no reading names the noun

#### Scenario: What gives no reading
- **WHEN** the tables are reduced, the dictionary lists `estre` as an archaic spelling of *être* with a conjugation table holding `est`, the Louisiana word *vader* with `va`, `elles` as the plural of `elle`, the name of the letter L, and `dieu` as the masculine of *déesse*
- **THEN** `est` names no *estre*, `va` no *vader*, `elles` is no plural of the letter, and `dieu` is no form of *déesse*

#### Scenario: Nothing else moves
- **WHEN** French's readings are committed
- **THEN** `tables/fr/forms.tsv`, `freq.tsv`, `lexical.tsv` and `studied.json` are byte for byte as before; en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before and their invariance baselines pass without re-blessing; and the French invariance baseline still runs over its fixture pack, its golden unmoved

### Requirement: French's pinned tag pool
`tables/fr/tags.tsv`, French's pinned tag pool, SHALL hold every tag French's readings carry, each once, one per line, and no other tag — the first reduction's in byte order, a tag a later reduction adds appended after them, so that a pinned tag never changes index; it SHALL be written by a person and kept by every reduction, as English's and Spanish's are, and the checks SHALL fail, naming the tag, when it holds a tag no reading carries or misses one a reading carries. Every pack studying French SHALL lay its tag pool out as that pin, then any reading tag the pin lacks, then the tags only its senses carry, so that fr-en and fr-es store a form's readings byte for byte alike whatever their glosses' senses carry.

#### Scenario: The pin is the readings' tags
- **WHEN** French's readings are committed
- **THEN** every tag of `tables/fr/grammar.tsv` is a line of `tables/fr/tags.tsv`, every line of it is a tag of `grammar.tsv` written in canonical form, each once, and its lines are in byte order

#### Scenario: A later reduction carries a new tag
- **WHEN** fr-en is reduced from a later dump whose readings carry a tag the pin does not hold
- **THEN** the checks fail and name the tag until it is appended to `tables/fr/tags.tsv`, after the tags already pinned

#### Scenario: A reduction keeps the pin
- **WHEN** fr-en is reduced again from its pinned sources
- **THEN** `tables/fr/tags.tsv` is byte for byte as committed, and no reducer writes it

#### Scenario: Senses another pack's glosses carry
- **WHEN** a pack studying French is built from `tables/fr/` with a sense run tagged `INTJ`, a tag no reading carries
- **THEN** its readings are stored byte for byte as they are in the pack built without that run, and the core reads the run as `INTJ`

### Requirement: French readings are measured on held-out treebanks
The pipeline SHALL report, for French's committed tables, on UD French-PUD and on UD French-GSD's test section — fetched at the commits French's forms are measured at and never read by the reduction —, for each part of speech, the share of the words whose form the tables map to the treebank's lemma that carry a reading, and the share of those whose treebank part of speech and features are among their readings, the conditional's and the imperative's tense left aside. The report SHALL NOT decide the pipeline's exit status.

#### Scenario: The readings' figures are reported
- **WHEN** the French measurement runs over the committed tables
- **THEN** it prints, beside the forms' gates, the readings' figures for verbs (finite, participles, infinitives), nouns, adjectives, determiners and pronouns on both treebanks, and they do not decide its exit status
