# Spec Delta

## ADDED Requirements

### Requirement: Italian's forms and frequencies
Italian's studied tables SHALL be reduced by its reference pair, it-fr, from the English Wiktionary's Italian section as derived from the English edition's dump, and committed once, in `tables/it/`. A form SHALL come from the inflections a lemma's entry lists or from the form-of links of the form's own entry, never from an inflection the dictionary marks alternative, obsolete, archaic, rare, dated, misspelt, nonstandard or abbreviated, nor from a gender or number marker an entry lists among its forms, and a form whose candidate is itself a form of another word SHALL reach that word when both links are of one part of speech. A form Morph-it! lists under a lemma the tables rank, which the Wiktionary lists under no lemma, SHALL join the table, and a form the Wiktionary reads SHALL never be moved by it. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in UD Italian-PUD and MarkIT's training section, then by the form's own entry, then by the lemma's frequency, then alphabetically; a form that is a name and a commoner word SHALL keep the commoner reading. The tables SHALL serve Italian's tokenisation as change 6 writes it: every word its pre-pass writes SHALL be a form, each elided piece a form of the word it stands for by the reviewed table, the articulated prepositions of `di` lemmas of their own with `dell'` a form of `dello`, the other articulated prepositions neither forms nor ranks, each truncation a form of its word, and no plain word beginning with an elided piece nor any verb form carrying clitics a form. The ranks SHALL be wordfreq's Italian order over the 60,000 commonest lemmas whose own form reads as themselves, dense, wordfreq's elision stems no words, a hyphenated word ranked only on a treebank's evidence.

#### Scenario: A participle's agreement
- **WHEN** the tables are reduced and `parlata` is looked up
- **THEN** it reads as *parlare*, through the participle `parlato` along the verb entries

#### Scenario: A doubtful inflection
- **WHEN** an entry lists an inflection tagged archaic or misspelt
- **THEN** no form of the tables comes from it

#### Scenario: One lemma per form
- **WHEN** `porta` is a form of *porta* (the door) and of *portare* (he carries)
- **THEN** it maps to the one the treebanks' counts favour, else the form's own entry, else the commoner lemma, and the tables name the rule that decided

#### Scenario: A name and a word
- **WHEN** `marco` is both a name and the noun *marco* (the coin)
- **THEN** it keeps the commoner reading by wordfreq, as the tables record

#### Scenario: The elided pieces are forms
- **WHEN** `lo`, `una`, `questo`, `anche` and `dello` are looked up
- **THEN** each is a form, `lo` and `dello` of their own lemmas, and `l'`, `un'`, `quest'`, `anch'` and `dell'` are read as them by the pre-pass

#### Scenario: Articulated prepositions
- **WHEN** `del`, `della` and `nel` are looked up
- **THEN** `del` and `della` are lemmas of their own, and `nel` is no form and no rank

#### Scenario: A form carrying clitics
- **WHEN** `dirmelo` and `vattene` are looked up
- **THEN** neither is a form: the enclitic rule reads them

#### Scenario: Morph-it! completes a verb
- **WHEN** Morph-it! lists a form of a verb the tables rank which the Wiktionary's entry does not list
- **THEN** the form joins the table under that verb, and the pin records Morph-it! as a source

#### Scenario: Morph-it! never moves a reading
- **WHEN** Morph-it! files a form under a lemma other than the one the Wiktionary reads it as
- **THEN** the Wiktionary's reading stands

#### Scenario: wordfreq's elision stems
- **WHEN** wordfreq ranks `l`, `d`, `dell` and `quest` among its commonest words
- **THEN** none is a lemma of the tables, and the ranks close up

#### Scenario: Every rank on its own lemma
- **WHEN** the it-fr pack is built from the committed tables
- **THEN** every ranked lemma's own form reads as itself, and the pack gives each lemma its own rank

### Requirement: Italian forms are measured on held-out treebanks
Italian's committed tables SHALL be measured by the real analyser over UD Italian-PUD, held to the gates Spanish's and French's were (98.5 % of tokens resolved, 93.5 % of content words taking the treebank's lemma, 97 % of auxiliaries), and over MarkIT's test section, reported; both fetched at a pinned commit and checked by sha256, never committed, and never read by the reduction, which reads MarkIT's training section alone for its counts. A reduction whose tables fall under a gate SHALL NOT be committed, and the figures SHALL be recorded with each update of the tables.

#### Scenario: The committed tables pass
- **WHEN** the measurement runs over the committed tables
- **THEN** PUD's three figures meet the gates, and MarkIT's test section's are reported beside them

#### Scenario: A reduction under a gate
- **WHEN** a reduction's tables fall under a gate on PUD
- **THEN** the update is refused, naming the gate and the figure

#### Scenario: The test section is held out
- **WHEN** the reduction reads the treebanks for their counts
- **THEN** it reads MarkIT's training section and PUD, and MarkIT's test section is read by the measurement alone
