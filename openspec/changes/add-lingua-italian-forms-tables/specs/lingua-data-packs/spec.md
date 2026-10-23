# Spec Delta

## ADDED Requirements

### Requirement: Italian's forms and frequencies
Italian's studied tables SHALL be reduced by its reference pair, it-fr, from the English Wiktionary's Italian section as derived from the English edition's dump, and committed once, in `tables/it/`. A form SHALL come from the inflections a lemma's entry lists or from the form-of links of the form's own entry, never from an inflection the dictionary marks alternative, obsolete, archaic, rare, dated, misspelt, nonstandard or abbreviated, nor from a row naming a derived word — a diminutive, augmentative, pejorative, derogatory or endearing form, an adjective's adverb, a noun's relational adjective, a name's demonym or a plural noun's `plural-of` row —, nor from a row of a name's entry, nor from a sense naming its entry such a derived word, nor from a row of a verb carrying a clitic other than the reflexive one, alone or in a cluster, nor from a gender or number marker an entry lists among its forms, and a form whose candidate is itself a form of another word SHALL reach that word when both links are of one part of speech. A conjugation-table form SHALL be read without the stress marks the table writes on it — a non-final mark always, a final one only where the entry carrying the verb's form-of link for that cell is the bare spelling —; a table row written with its reflexive pronoun SHALL give its bare form as the candidate, under the plain verb where the section has one and under the reflexive lemma otherwise, a reflexive lemma whose plain verb the section has being no lemma of the tables and a reflexive-only lemma's clitic-bearing rows giving their bare base under it; a multi-word lemma's cells, a canonical row's bracketed auxiliary and a row listing alternatives SHALL give no form. Morph-it! is among the pinned sources — kept by the owner on 2026-10-23, the measurement showing it adds without moving a gate down —: a form it lists under a lemma the tables rank, which the Wiktionary lists under no lemma, SHALL join the table, and a form the Wiktionary reads SHALL never be moved by it. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in MarkIT's training and development sections, then by the form's own entry, then by the lemma's frequency, then alphabetically; before it, a form whose own lemma entries are names alone and which another word's entry reads SHALL read as that word, and every other form SHALL be left to those rules. The tables SHALL serve Italian's tokenisation as change 6 writes it: every word its pre-pass writes SHALL be a form, each elided piece a form of the word it stands for by the reviewed table, the articulated prepositions of `di` lemmas of their own with `dell'` a form of `dello`, the other articulated prepositions neither forms nor ranks — `dai` excepted, a form of *dare* and no rank, the truncation `da'`'s word —, each truncation a form of its word, and no plain word beginning with an elided piece nor any verb form carrying clitics a form. The ranks SHALL be wordfreq's Italian order over the 60,000 commonest lemmas whose own form reads as themselves, dense, wordfreq's stems of the elided pieces and truncations and its tokens beginning with an elided piece no words — `cent` and the apocopes written without an apostrophe words —, a hyphenated word ranked only on MarkIT's evidence; each apocope written without an apostrophe SHALL be a form of its word alone by a reviewed table (`un` → *uno*, `bel` → *bello*, `san` → *santo*), whatever the section's own entries for it say. `tables/it/studied.json` SHALL name it-fr as Italian's reference pair; `tags.tsv` SHALL hold the one canonical tag until change 13's pool, and `lexical.tsv` SHALL hold nothing until change 11's acronym rows and change 16's dictionary words; the it-fr pack SHALL be built from the committed tables, whole and in parts, and checked against its pin in the extension's checks as the six pairs' are; the update workflow SHALL offer it-fr and publish its sources release; no package SHALL list it-fr; and nothing of the six pairs' tables, pins and goldens, nor the Italian baseline's fixture and golden, SHALL move.

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
- **WHEN** `ti` is a surname's entry and the pronoun's, which MarkIT counts as itself
- **THEN** it reads as the pronoun, the names rule leaving it to the counts; and `bianca`, the name, a cue ball and the feminine of *bianco*, reads as *bianco* by its reviewed override row

#### Scenario: An apocope
- **WHEN** `bel`'s entry is a meaning (« preconsonantal masculine singular form of bello ») and `san`'s own entry is the Greek letter
- **THEN** `bel` reads as *bello* and `san` as *santo* by the reviewed apocope table, and neither is ranked

#### Scenario: A name alone, spelled like a word
- **WHEN** `bambini` is a surname's entry and the plural *bambino*'s entry lists
- **THEN** it reads as *bambino*, and no rank goes to the surname

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
- **THEN** `l`, `d`, `po` and `dell'acqua` are no lemmas of the tables, `un` and `buon` are forms of *uno* and *buono*, and the ranks close up

#### Scenario: The reference and the pipeline
- **WHEN** the tables are committed
- **THEN** `tables/it/studied.json` names it-fr, `tags.tsv` holds the one canonical tag until change 13's pool replaces it, `lexical.tsv` holds nothing until change 11's acronym rows and change 16's dictionary words, the extension's checks build the it-fr pack whole and in parts to its pin, and no package lists it-fr

#### Scenario: Nothing else moves
- **WHEN** the tables are committed
- **THEN** the six pairs' tables, pins and goldens and the Italian baseline's fixture and golden are byte for byte what they were

#### Scenario: Every rank on its own lemma
- **WHEN** the it-fr pack is built from the committed tables
- **THEN** every ranked lemma's own form reads as itself, and the pack gives each lemma its own rank

### Requirement: Italian forms are measured on held-out treebanks
Italian's committed tables SHALL be measured by the real analyser over UD Italian-PUD, held to Italian's gates — 98.5 % of tokens resolved and 97 % of auxiliaries taking the treebank's lemma, as Spanish's and French's were, and 90.5 % of content words, the owner's gate of 2026-10-23 for Italian, whose treebank counts are about sixteen times thinner than French's —, and over MarkIT's test section, reported; both fetched at a pinned commit and checked by sha256, never committed, and never read by the reduction, which reads MarkIT's training and development sections alone for its counts. The measurement SHALL fail, naming the gate and the figure, when a figure falls under its gate, and the figures SHALL be recorded with each update of the tables.

#### Scenario: The committed tables pass
- **WHEN** the measurement runs over the committed tables
- **THEN** PUD's three figures meet the gates, and MarkIT's test section's are reported beside them

#### Scenario: A reduction under a gate
- **WHEN** a reduction's tables fall under a gate on PUD
- **THEN** the measurement fails, naming the gate and the figure

#### Scenario: The test section is held out
- **WHEN** the reduction reads the treebanks for their counts
- **THEN** it reads MarkIT's training and development sections, and PUD and MarkIT's test section are read by the measurement alone

## MODIFIED Requirements

### Requirement: One base per studied language
The base built from any pair's committed tables SHALL be byte-identical to the base built from the reference pair's of the same studied language, metadata included; the checks over the committed tables SHALL fail, naming both pairs and what differs, when it is not. The builder SHALL refuse to build a base from inputs that would make it depend on the pair — a glossed lemma that is neither the lemma of a form nor a ranked lemma, or dictionary words that are exactly the pair's glossed lemmas —, naming the lemma or the dictionary words; a pair that glosses nothing, whose studied language has no dictionary words yet, SHALL not be refused for it, neither its base nor its layer naming a dictionary word. The parts of every committed pair SHALL each stay under the size budget.

#### Scenario: One English base
- **WHEN** en-fr's and en-es's bases are built from the committed tables
- **THEN** they are byte-identical, and so are es-fr's and es-en's, and fr-en's and fr-es's

#### Scenario: A pair whose manifest disagrees with its language's reference
- **WHEN** a second pair studying Spanish declares its levels estimated otherwise than es-fr's manifest does
- **THEN** the checks fail, naming both pairs and the field

#### Scenario: A pair whose glosses would reach its base
- **WHEN** a pair's glosses hold a lemma its studied tables do not, or its dictionary words are exactly the lemmas it glosses
- **THEN** the split build fails naming the lemma or the dictionary words, and the whole pack builds as before

#### Scenario: A pair that glosses nothing
- **WHEN** it-fr's tables hold forms and ranks, an empty `gloss.tsv` and an empty `lexical.tsv` (add-lingua-italian-forms-tables D9)
- **THEN** the split build writes its base without a lexical table and its layer without glosses, and the assembled pack names no dictionary word

#### Scenario: Every part under budget
- **WHEN** the six committed pairs are built as bases and layers
- **THEN** every base and every layer is under 5 MiB
