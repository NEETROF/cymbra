# Spec Delta

## MODIFIED Requirements

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair glossed in a native language gives that native language's default studied language, and SHALL build each listed pair's layer, and its studied language's base, from the pair's committed tables, each checked against the pair's own recorded sha256. It SHALL refuse a base whose analyser version is not the version of its studied language, and a layer whose `base_version` is not its base's. Every package of a release SHALL carry exactly the listed pairs' layers and their studied languages' bases, one of each, byte-identical across Chromium, Firefox and Safari. The list SHALL hold en-fr, es-fr, es-en, en-es then fr-en, followed by fr-es once fr-es's tables are committed at or above its coverage floor, then it-fr, and a package whose list is anything else SHALL be refused. A listed pair held to a coverage floor SHALL measure at or above it on its committed tables.

#### Scenario: Building the listed packs
- **WHEN** a release builds its parts with the list holding en-fr, es-fr, es-en, en-es, fr-en, fr-es then it-fr
- **THEN** it builds each pair's layer and each language's base — the Italian one among them — from their own tables, each sha256 is the one that pair's pin records, and the package carries those layers and bases and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pair's base declares an analyser version other than the core's version for its studied language
- **THEN** the build fails with a message naming the base, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list other than en-fr, es-fr, es-en, en-es then fr-en, with fr-es after them or not as its floor allows, then it-fr
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack, its layer and the English base are built from their tables after this change
- **THEN** the pack is byte for byte what was built before, its layer and the English base are what change 3's checks build from the same tables, and English stays a French-native reader's default studied language

#### Scenario: English speakers learning Spanish
- **WHEN** a reader whose browser is in English installs a package built with this list
- **THEN** the native-language choice is offered, preset to English, and the reader studies Spanish through es-en

#### Scenario: Spanish speakers learning English
- **WHEN** a reader whose browser is in Spanish installs a package built with this list
- **THEN** the native-language choice is offered, preset to Spanish, and the reader studies English through en-es

#### Scenario: English speakers learning French
- **WHEN** a reader whose native language is English uses a package built with this list
- **THEN** Spanish stays their default studied language, French is offered beside it, and a French page is read through fr-en

#### Scenario: Spanish speakers learning French
- **WHEN** fr-es is listed and a reader whose native language is Spanish uses a package built with this list
- **THEN** English stays their default studied language, French is offered beside it, and a French page is read through fr-es

#### Scenario: French for English speakers alone
- **WHEN** fr-es's first committed measurement fell under its floor, so that no fr-es table is committed
- **THEN** fr-en is followed by it-fr, no package carries fr-es, and a reader whose native language is Spanish is offered English alone

#### Scenario: No French for French speakers
- **WHEN** a reader whose native language is French uses a package built with this list
- **THEN** their pairs are en-fr, es-fr and it-fr, French is never offered, and neither French layer nor the French base is fetched

#### Scenario: A listed pair under its floor
- **WHEN** the list names a pair held to a coverage floor whose committed tables measure under it at one of the three tops
- **THEN** the coverage tests fail, naming the pair, the top and the figure

#### Scenario: French speakers learning Italian
- **WHEN** a reader whose native language is French uses a package built with this list
- **THEN** English stays their default studied language, Italian is offered after Spanish, and an Italian page is read through it-fr, the Italian base and it-fr's layer fetched on first need

#### Scenario: No Italian for English and Spanish speakers
- **WHEN** a reader whose native language is English or Spanish uses a package built with this list
- **THEN** Italian is never offered, and neither it-fr's layer nor the Italian base is fetched

### Requirement: Italian's forms and frequencies
Italian's studied tables SHALL be reduced by its reference pair, it-fr, from the English Wiktionary's Italian section as derived from the English edition's dump, and committed once, in `tables/it/`. A form SHALL come from the inflections a lemma's entry lists or from the form-of links of the form's own entry, never from an inflection the dictionary marks alternative, obsolete, archaic, rare, dated, misspelt, nonstandard or abbreviated, nor from a row naming a derived word — a diminutive, augmentative, pejorative, derogatory or endearing form, or an adjective's adverb —, nor from a row of a verb carrying a clitic other than the reflexive one, alone or in a cluster, nor from a gender or number marker an entry lists among its forms, and a form whose candidate is itself a form of another word SHALL reach that word when both links are of one part of speech. A conjugation-table form SHALL be read without the stress marks the table writes on it — a non-final mark always, a final one only where the entry carrying the verb's form-of link for that cell is the bare spelling —; a table row written with its reflexive pronoun SHALL give its bare form as the candidate, under the plain verb where the section has one and under the reflexive lemma otherwise, a reflexive lemma whose plain verb the section has being no lemma of the tables and a reflexive-only lemma's clitic-bearing rows giving their bare base under it; a multi-word lemma's cells, a canonical row's bracketed auxiliary and a row listing alternatives SHALL give no form. When Morph-it! is among the pinned sources — kept only where the measurement shows it adds without moving the gates, the owner settling it —, a form it lists under a lemma the tables rank, which the Wiktionary lists under no lemma, SHALL join the table, and a form the Wiktionary reads SHALL never be moved by it. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in MarkIT's training and development sections, then by the form's own entry, then by the lemma's frequency, then alphabetically; a form that is a name and a commoner word SHALL keep the commoner reading. The tables SHALL serve Italian's tokenisation as change 6 writes it: every word its pre-pass writes SHALL be a form, each elided piece a form of the word it stands for by the reviewed table, the articulated prepositions of `di` lemmas of their own with `dell'` a form of `dello`, the other articulated prepositions neither forms nor ranks — `dai` excepted, a form of *dare* and no rank, the truncation `da'`'s word —, each truncation a form of its word, and no plain word beginning with an elided piece nor any verb form carrying clitics a form. The ranks SHALL be wordfreq's Italian order over the 60,000 commonest lemmas whose own form reads as themselves, dense, wordfreq's stems of the elided pieces and truncations and its tokens beginning with an elided piece no words — `un` and the apocopated forms written without an apostrophe (`buon`, `nessun`) words —, a hyphenated word ranked only on MarkIT's evidence. `tables/it/studied.json` SHALL name it-fr as Italian's reference pair; `tags.tsv` SHALL hold the one canonical tag until change 13's pool, and `lexical.tsv` SHALL hold nothing until change 11's acronym rows and change 16's dictionary words; the it-fr pack SHALL be built from the committed tables, whole and in parts, and checked against its pin in the extension's checks as the six pairs' are; the update workflow SHALL offer it-fr and publish its sources release; a package SHALL carry it-fr only once the list of shipped pairs names it; and nothing of the six pairs' tables, pins and goldens, nor the Italian baseline's fixture and golden, SHALL move.

#### Scenario: A participle's agreement
- **WHEN** the tables are reduced over an entry listing `creduta` only as the feminine of the participle `creduto`, filed under *credere*, and `creduta` is looked up
- **THEN** it reads as *credere*, through the participle along the verb entries

#### Scenario: The table's stress marks
- **WHEN** *dare*'s table lists `dàre`, `dàto`, `dièdi`, `darà` and `dà`, and *avere*'s `hà`
- **THEN** the forms are `dare`, `dato`, `diedi`, `darà` and `dà` (their entries are written so; `da` the preposition is no form of *dare*) and `ha` (*avere*'s entry for the cell is bare)

#### Scenario: A row written with its pronoun
- **WHEN** *lavarsi*'s table lists `mi làvo` and `si làva` and *lavare* is an entry, and *pentirsi*'s lists `mi pènto` and `pentìrsi` with no *pentire*
- **THEN** `lavo` and `lava` are forms of *lavare* and `lavarsi` is no lemma; `pento`, `pentire` and `pentirsi` are forms of *pentirsi*; and no form holds a space

#### Scenario: A derived word
- **WHEN** *casa*'s entry lists `casétta` as its diminutive and *vero*'s lists `veramente` as its adverb
- **THEN** neither is a form of *casa* or *vero*; each is read from its own entry

#### Scenario: A doubtful inflection
- **WHEN** an entry lists an inflection tagged archaic or misspelt
- **THEN** no form of the tables comes from it

#### Scenario: One lemma per form
- **WHEN** `porta` is a form of *porta* (the door) and of *portare* (he carries)
- **THEN** it maps to the one the treebanks' counts favour, else the form's own entry, else the commoner lemma, and the tables name the rule that decided

#### Scenario: A name and a word
- **WHEN** `bianca` is both the name and the feminine of *bianco*
- **THEN** it keeps the commoner reading by wordfreq, *bianco*, as the tables record

#### Scenario: The elided pieces are forms
- **WHEN** `lo`, `una`, `questo`, `anche` and `dello` are looked up
- **THEN** each is a form, `lo` and `dello` of their own lemmas, and `l'`, `un'`, `quest'`, `anch'` and `dell'` are read as them by the pre-pass

#### Scenario: Articulated prepositions
- **WHEN** `del`, `della`, `nel` and `dai` are looked up
- **THEN** `del` and `della` are lemmas of their own, `nel` is no form and no rank, and `dai` is a form of *dare* and no rank

#### Scenario: A form carrying clitics
- **WHEN** `dirmelo` and `vattene` are looked up
- **THEN** neither is a form: the enclitic rule reads them

#### Scenario: Morph-it! completes a verb
- **WHEN** Morph-it! is among the pinned sources and lists a form of a verb the tables rank which the Wiktionary's entry does not list
- **THEN** the form joins the table under that verb, and the pin records Morph-it! as a source

#### Scenario: Morph-it! never moves a reading
- **WHEN** Morph-it! is among the pinned sources and files a form under a lemma other than the one the Wiktionary reads it as
- **THEN** the Wiktionary's reading stands

#### Scenario: wordfreq's elision stems
- **WHEN** wordfreq ranks `l`, `d`, `po`, `dell'acqua`, `un` and `buon` among its commonest words
- **THEN** `l`, `d`, `po` and `dell'acqua` are no lemmas of the tables, `un` and `buon` are words, and the ranks close up

#### Scenario: The reference and the pipeline
- **WHEN** the tables are committed
- **THEN** `tables/it/studied.json` names it-fr, `tags.tsv` holds the one canonical tag until change 13's pool replaces it, `lexical.tsv` holds nothing until change 11's acronym rows and change 16's dictionary words, the extension's checks build the it-fr pack whole and in parts to its pin, and a package carries it-fr only once the list of shipped pairs names it

#### Scenario: Nothing else moves
- **WHEN** the tables are committed
- **THEN** the six pairs' tables, pins and goldens and the Italian baseline's fixture and golden are byte for byte what they were

#### Scenario: Every rank on its own lemma
- **WHEN** the it-fr pack is built from the committed tables
- **THEN** every ranked lemma's own form reads as itself, and the pack gives each lemma its own rank
