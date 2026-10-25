# Spec Delta

## ADDED Requirements

### Requirement: Italian is glossed in English from the English Wiktionary's Italian section and the Italian Wiktionary's English translations
it-en's glosses, sense runs and expressions SHALL be reduced by a reducer of its own, reading Italian's studied tables as committed, through the native side every pair shares with the English Wiktionary's rules, from the English Wiktionary's Italian section's definitions, then the Italian Wiktionary's English translations, at most three words per part of speech, and never from the English Wiktionary's translation tables read backwards. Every gloss SHALL be English written by a person. After the English edition's pre-passes and before the shared rules, it-en's own pre-pass SHALL read the meaning a pointer sense carries after its pointer, by a reviewed list of wordings — Italian's own shapes (a form for euphony, the article's form, a pronoun's clitic case form, the `di` family's contraction) wherever they stand, and a contraction's, a plural's or a degree's only for a word whose lowercase entries hold nothing else, the meaning read from the gloss's text; SHALL keep an acronym's sense off a lemma the section gives a lowercase entry of its own unless that entry only points at the acronym, and a name's run off a word with a function word's entry, while a name beside a common word's senses is kept; SHALL gloss no word with a letter's name, Latin, or Greek where a common word's entry of the word holds another meaning; and SHALL take out notes that are not glosses. A sense repeating an earlier one of the same word word for word SHALL be shown once, a name's sense written with a capital kept beside a common word's. The acronym rows of Italian's studied tables SHALL be glossed by *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* from it-en's own glosses. it-en's expressions SHALL be the section's glossed headwords with a space, then those the Italian Wiktionary's English translations gloss, then the words without a space that Italian's pre-pass splits and the section or the Italian Wiktionary glosses with a meaning, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires. The English edition's shared rules, and every other pair's reducer, tables and pin, SHALL NOT move.

#### Scenario: A meaning a pointer carries
- **WHEN** the section's `della` carries the sense « contraction of di la; of the, from the »
- **THEN** `della`'s gloss is « of the, from the », not the pointer's wording

#### Scenario: An acronym off a common word
- **WHEN** the section's `ad` has the sense « alternative form of a for euphony…; to, at, in » and the acronym `AD` the sense « chief executive officer, CEO »
- **THEN** `ad`'s gloss is « to, at, in », with no « CEO »

#### Scenario: A name beside a common word's senses
- **WHEN** the section has `natale` as a lowercase entry and `Natale` « Christmas » as a proper noun
- **THEN** `natale`'s gloss keeps « Christmas »

#### Scenario: A name off a function word
- **WHEN** the section's `io` « I » shares its row with the moon `Io`
- **THEN** `io`'s gloss holds no run of the moon

#### Scenario: A Greek letter
- **WHEN** the section's `mi` holds the sense « mu (Greek letter) » beside its pronoun's « clitic accusative of io. me »
- **THEN** `mi`'s gloss holds no Greek letter and reads « me »

#### Scenario: The di family's contraction beside other pointers
- **WHEN** the section's `dei` has the contraction « contraction of di + i; of the, from the » and the sense « some », beside the plural of `dio` and two alternative forms
- **THEN** `dei`'s gloss is « of the, from the; some »

#### Scenario: A carrying pointer beside the word's own senses
- **WHEN** the section's `colla` has the contraction « contraction of con + la; with the » and the noun « glue or similar sticky material »
- **THEN** `colla`'s gloss is the noun's, with no « with the »

#### Scenario: A sense repeated
- **WHEN** the Italian Wiktionary lists « ex, former » under `ex`'s adjective and its noun
- **THEN** `ex`'s gloss is « ex, former », once

#### Scenario: A word that is only a Greek letter
- **WHEN** the section's `rho` holds « rho (Greek letter) » beside a town's proper noun, and no common word's entry of `rho` holds another meaning
- **THEN** `rho`'s gloss keeps « rho (Greek letter) »

#### Scenario: A gloss from the Italian Wiktionary
- **WHEN** the section's only sense of `perfino` is « synonym of persino », which lends nothing, and the Italian Wiktionary's `perfino` lists the English translation « even »
- **THEN** `perfino` is glossed « even »

#### Scenario: No table read backwards
- **WHEN** an English entry of the English Wiktionary lists an Italian word as its translation, and neither the section nor the Italian Wiktionary glosses that word
- **THEN** it-en does not gloss it

#### Scenario: Expressions from every source
- **WHEN** the section glosses `un po'` and `d'accordo`, and the Italian Wiktionary translates `meccanica quantica` into English
- **THEN** all three are expressions of `mwe.tsv`, keyed at build

#### Scenario: Nothing else moves
- **WHEN** it-en's tables are committed
- **THEN** `reduce_edition_en.py` and `reduce_common.py` are byte for byte what they were, and every other pair's tables, pins and goldens are unchanged

### Requirement: it-en is committed beside Italian's studied tables, held to a coverage floor, and not shipped
it-en's native side SHALL be reduced from its own pin — the English Wiktionary's Italian section, fetched and pinned by its own update as fr-en's is, and the Italian dump's derived English translations — and committed in `tables/it-en/`, reading `tables/it/` as committed and writing none of it; its pin SHALL record the studied tables it read, and its notice and manifest SHALL credit both sides. Its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, and SHALL stay under the size budget. Its coverage of the 5,000, 10,000 and 20,000 commonest Italian lemmas SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["it-en"]`, fixed before the first committed measurement at the study's figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail the checks. it-en SHALL gloss at least 89.3, 82.2 and 73.7 % of the 5,000, 10,000 and 20,000 commonest Italian lemmas, the floor so fixed: the study's 91.3, 84.2 and 75.7 % on change 9's committed ranks, with the Italian Wiktionary's translations and the section read as this change reads it, less two points. An it-en invariance baseline SHALL run over the Italian corpus with the committed it-en pack wherever the Italian one runs, the it-fr and it-en packs reading every Italian word alike. No package SHALL list it-en before Italian is enabled for English speakers.

#### Scenario: The floor holds
- **WHEN** it-en is reduced and its coverage is at or above `FLOORS["it-en"]`
- **THEN** the tables are committed and the coverage is shown in their README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces it-en and its committed tables measure under `FLOORS["it-en"]`
- **THEN** the checks fail — the gloss-coverage floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["it-en"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The credits
- **WHEN** it-en's tables are committed
- **THEN** its notice names the English Wiktionary's Italian section, the Italian Wiktionary, and every source of Italian's studied tables, and its manifest says Italian's levels are estimated

#### Scenario: One studied side, two natives
- **WHEN** the Italian corpus is read with the it-fr and the it-en packs
- **THEN** every token, lemma, reading and level is the same, and only the glosses differ

#### Scenario: Not shipped
- **WHEN** it-en's tables are committed and the list of shipped pairs does not name it-en
- **THEN** no package carries it-en, and no reader's package or engine moves

## MODIFIED Requirements

### Requirement: French is glossed in English from the English Wiktionary's French section
fr-en's glosses, sense runs and expressions SHALL be reduced from the English Wiktionary's French section alone — the derived file French's forms come from, at the same pinned snapshot — cleaned by the English Wiktionary's rules as es-en's glosses are, and no translation table SHALL gloss a French word or expression in fr-en.
Before the shared rules read the section, its letters and the entries written under a single capital
letter are left out, its senses are read as meanings and written in one English typography — the
rules es-en's glosses are read by — and the English edition's long-parenthesis bound and etymology
merging apply as they are set. A typographic apostrophe in a headword is read as `'`, as French's
forms are. Its expressions are the section's headwords with a space and the single words French's
tokenisation splits (an elision, an inversion) that are no form, less the senses that only point at
another spelling, an inverted form or their pieces, and less `à la`, whose one sense needs a word
after it. Nothing is pivoted or machine-translated: a word or an expression the section does not
gloss has no gloss. fr-en being French's reference pair, its glossed lemmas are French's dictionary
words. The coverage of the 5,000, 10,000 and 20,000 commonest French lemmas SHALL be at least 91.9,
85.1 and 74.4 %, checked by the reduce job, and no site SHALL publish it before a package lists
fr-en. The English edition's rules SHALL be read by es-en, fr-en and it-en and by no other committed pair,
so that a change to them re-pins those three and no other.

#### Scenario: Coverage
- **WHEN** `gloss_coverage.py --pair fr-en` runs on the committed tables, against its floor of 91.9, 85.1 and 74.4 %
- **THEN** it passes, and the measured figures are shown in the pull request

#### Scenario: A sense-group label
- **WHEN** the section nests « a room », « a hotel room », « a bedroom » and « a house of a parliament » under « chambre »'s « a chamber in its various senses, including: »
- **THEN** « chambre »'s gloss opens on « a room; a hotel room », and does not hold « in its various senses »

#### Scenario: A pronoun's senses under its description
- **WHEN** the section nests « we » and « us, to us » under « nous »'s « the plural personal pronoun in the first person: »
- **THEN** « nous »'s gloss opens on « we; us, to us »

#### Scenario: The edition's description in lower case
- **WHEN** the section glosses the partitive article « du » « Forms the partitive article. »
- **THEN** « du »'s gloss is « forms the partitive article »

#### Scenario: A word only a translation table glosses
- **WHEN** the section has no entry for `end`, ranked among the 1,000 commonest French words, while the French Wiktionary's entry `END` lists « NDE, NDI, NDT » as its English translations
- **THEN** `end` has no gloss in fr-en, and is no dictionary word of French

#### Scenario: A bigram an English entry translates
- **WHEN** the English Wiktionary's English entry « he's » lists the French « il est » among its translations
- **THEN** fr-en has no expression « il est »

#### Scenario: A word the pre-pass splits
- **WHEN** the section glosses `d'abord` « first, at first » and `allez-y`, neither a form of `tables/fr/forms.tsv`
- **THEN** fr-en's `mwe.tsv` holds both with their glosses, and holds no `aujourd'hui`, which the forms table lists

#### Scenario: An expression that only points
- **WHEN** the section's « crème fraiche » only says « post-1990 spelling of crème fraîche », « qu'elle » only « que + elle », and « y a-t-il » « subject-inverted form of il y a; is there? are there? »
- **THEN** fr-en has no expression « crème fraiche » or « qu'elle », keeps « crème fraîche », and glosses « y a-t-il » « is there? are there? »

#### Scenario: À la
- **WHEN** the section glosses « à la » « in the style or manner of » and « à la carte » on its own entry
- **THEN** fr-en has no expression « à la », and « à la carte » keeps its gloss

#### Scenario: French's dictionary words
- **WHEN** fr-en is reduced
- **THEN** `tables/fr/lexical.tsv` lists exactly the lemmas `tables/fr-en/gloss.tsv` glosses, and fr-en's pack carries no lexical table

#### Scenario: A gloss is its lemma's
- **WHEN** fr-en's glosses are committed
- **THEN** every lemma `tables/fr-en/gloss.tsv` glosses is a ranked lemma that `tables/fr/forms.tsv` reads as itself, so that the pack attaches no gloss to another word

#### Scenario: A rule of the English edition
- **WHEN** `reduce_edition_en.py` changes
- **THEN** es-en's, fr-en's and it-en's rule digests move, and en-fr's, es-fr's, en-es's, fr-es's and it-fr's do not
