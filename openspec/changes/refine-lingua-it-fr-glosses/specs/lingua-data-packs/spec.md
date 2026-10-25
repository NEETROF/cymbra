## ADDED Requirements

### Requirement: it-fr's glosses read an Italian word's meanings in the order Italian uses them
The it-fr reduction SHALL open a function word's row on the part of speech UD Italian-MarkIT's training and development sections read the word as at least ten times and at least twice as often as the part of speech the section opens the row on, never on a proper noun's; and SHALL open a row a reviewed table names on the entries of the headword's case and the part of speech the table gives it, whatever the treebank reads.
The function words' parts of speech the treebank may move first are `ADP`, `DET`, `PRON`, `CCONJ`,
`SCONJ`, `PART` and `ADV`, and a row the section opens on a proper noun's may move too; a noun, a verb
or an adjective the treebank reads more often than the noun, verb or adjective the section opens on
does not move. The treebank's counts are the ones it-fr's pin already records: a word inside a fixed
expression counts for none, a noun, a verb, an adjective or a proper noun counts under its lemma and
any other part of speech under its own form. The entries moved first come first, the others keep
their order; no order adds or removes a sense. The reviewed table holds a row only for a row the rule
does not move, or moves wrongly, each with its reason, decided by the owner; its key is the headword's
case and a part of speech, so it can order two entries of one part of speech. These are rules of
it-fr's own reducer: they re-pin it-fr and no other pair, its coverage stays at least its floor, and
the Italian baseline is re-blessed once, in the pull request that commits them.

#### Scenario: A function word's commonest part of speech
- **WHEN** the section gives `fra` the noun « Frère » before the preposition « Entre, durant », and MarkIT reads `fra` 13 times as a preposition and never as a noun
- **THEN** the gloss of `fra` is « Entre, durant; Frère »

#### Scenario: A row the table holds against the treebank
- **WHEN** MarkIT reads `come` 39 times as a subordinating conjunction and 16 times as an adverb, the part of speech the section opens it on, and the reviewed table names `come`'s adverb
- **THEN** the gloss of `come` still opens on « Comme »

#### Scenario: A content word keeps the section's order
- **WHEN** the section opens `piacere` on the verb « Plaire » and MarkIT reads it more often as a noun
- **THEN** the gloss of `piacere` still opens on « Plaire »

#### Scenario: Too little evidence
- **WHEN** the section opens `subito` on the adjective « Immédiat, instantané » and MarkIT reads `subito` 3 times as an adverb
- **THEN** the gloss of `subito` still opens on « Immédiat, instantané »

#### Scenario: A row the reviewed table orders
- **WHEN** the reviewed table names `fino`'s preposition, and the section opens `fino` on the adjective « Fin »
- **THEN** the gloss of `fino` opens on « Jusque »

#### Scenario: Two entries of one part of speech
- **WHEN** the section gives the capitalised pronoun `Loro` « Pronom de la troisième personne du pluriel, utilisé pour la politesse… » before the pronoun `loro` « Ils, elles, eux », and the reviewed table names `loro`'s lowercase pronoun
- **THEN** the gloss of `loro` opens on « Ils, elles, eux »

#### Scenario: Nothing else moves
- **WHEN** it-fr is reduced again with these rules
- **THEN** the six other pairs' tables, pins, packs and goldens are byte for byte as before, and it-fr's coverage is at least `FLOORS["it-fr"]`

### Requirement: it-fr reads a pointer as its word's meaning where the section names one
The it-fr reduction SHALL read a pointer whose text names its word (« Pluriel de lettera ») or that the source files as an alternative form, unless the sense is marked obsolete, archaic or dated, as a pointer to that word; SHALL read a superlative's or a plural's pointer that writes its meaning in quotation marks as that meaning; SHALL let a noun's or an adjective's pointer lend its word's senses when the headword's other senses are only a proper noun's or marked archaic, obsolete or dated; and SHALL read the links of the `di` family, and a sense « Article contracté souvent traduit par X » where the word has no other glossed sense, as glosses.
A pointer named by its text names the word after its last « de », « du » or « d' » when that word is
the whole rest of the text. A pointer lends as the rules every pair shares lend — its word's senses
in the same part of speech — and a lemma glossed otherwise is not read again. A quoted meaning is read
only after a superlative's or a plural's wording; a clitic compound's quoted pronoun stays a pointer.
The entries a pointer lends to come before the proper noun's and the outdated sense's, so the row opens
on a lent sense; a verb form's pointer never lends beside a name. A sense « Article contracté souvent
traduit par X » reads « X », capitalised; a word with other glossed senses keeps them alone. A verb
form's borrowed senses are written as the verb's, the card's readings naming the verb.

#### Scenario: A plural the source does not tag
- **WHEN** the section's only sense of `lettere` is « Pluriel de lettera. », with no field naming the word, and `lettera` is glossed as a noun
- **THEN** `lettere` is glossed by `lettera`'s noun senses

#### Scenario: An alternative form
- **WHEN** the section's only sense of `paro` is « Variante de paio. », filed as an alternative form of `paio`
- **THEN** the gloss of `paro` is `paio`'s « Paire »

#### Scenario: An obsolete alternative form
- **WHEN** the section's only sense of `fora` is « Variante de fuori. », filed as an alternative form and marked obsolete
- **THEN** `fora` is not glossed by `fuori`'s senses

#### Scenario: A superlative's meaning
- **WHEN** the section's only sense of `pochissimo` is « Superlatif de poco : « très peu ». », its target named as « peu »
- **THEN** the gloss of `pochissimo` is « Très peu »

#### Scenario: A clitic compound's pronoun
- **WHEN** the section defines `vallo` « Agglutination de la deuxième personne du singulier de l'impératif de andare avec le pronom lo (« le ») »
- **THEN** `vallo` is not glossed « Le »

#### Scenario: A plural kept from lending by a constellation
- **WHEN** the section gives `gemelli` only « Pluriel de gemello » as a noun and « Masculin pluriel de gemello » as an adjective, and the proper noun `Gemelli` « Gémeaux (constellation et signe zodiacal) »
- **THEN** the gloss of `gemelli` opens on `gemello`'s « Jumeau » and keeps « Gémeaux (constellation et signe zodiacal) », and `gemelli` is a dictionary word of Italian

#### Scenario: An archaic sense beside a plural
- **WHEN** the section gives `spezie` the archaic « Espèce » and « Pluriel de spezia »
- **THEN** the gloss of `spezie` opens on `spezia`'s « Épice… », « Espèce » after it

#### Scenario: An article read through its translation
- **WHEN** the section defines `della` « Article contracté souvent traduit par De la. », linked to the Italian `la`
- **THEN** the gloss of `della` is « De la »

#### Scenario: A word with a gloss of its own
- **WHEN** `pelle` has its own glossed senses and a sense « Article contracté souvent traduit par pour les »
- **THEN** the gloss of `pelle` is its own senses alone

### Requirement: it-fr's glosses hold no name, usage note or example a common word does not need, and no split word that is no Italian
The it-fr reduction SHALL read a proper noun's sense that a lowercase entry of a common part of speech gives word for word as that entry's; SHALL leave a proper noun's senses out of a word whose every lowercase entry is a preposition's, a conjunction's, a pronoun's, a determiner's, an article's, a particle's or an adverb's; SHALL read a proper noun's sense that designates its place as a commune right after its name as a name's note, kept off a word with a lowercase entry of its own as a surname's note is; SHALL leave a usage note out of a determiner's or a preposition's sense; SHALL leave out a sense that is an Italian example and its French translation joined by a dash; and SHALL NOT offer as an expression a word Italian's pre-pass splits whose letters after the elided piece hold `j`, `k`, `w`, `x` or `y`, or a second apostrophe.
A sense is repeated when it is equal up to case and its closing period. A proper noun's entry written
in capitals (an acronym) is not left out by the function words' rule. A commune's note is a sense
whose place the text names « commune » first after its name (« X, commune… », « X (commune… »,
« X, commune et ville… »); it is kept when the sense also names the place a chef-lieu or a capital,
when one of the headword's proper nouns' senses is, or ends on, « province autour de cette ville »,
whatever its case — a provincial capital —, or when a reviewed list keeps it, each row with its reason,
decided by the owner. A commune note that was a word's only sense leaves it with no gloss, as a
surname's note does. A usage note runs from « Note d'usage » to the end of its sense; another part of
speech's usage note stays, a reader's. An example is a sense whose text before « — » ends a sentence,
holds a form of the headword, is written in Italian and holds none of a closed list of French-only
words; the dash and the French after it go with it.

#### Scenario: A common word its name repeats
- **WHEN** the section gives the proper noun `Bibbia` « Bible » before the noun `bibbia` « Bible »
- **THEN** the gloss of `bibbia` is « Bible » as one noun run, and `bibbia` is a dictionary word of Italian

#### Scenario: A pronoun's and an adverb's homograph names
- **WHEN** the section gives `io` its pronoun's senses and the proper noun `Io` « Io, fille d'Inachus… », and `qui` the adverb « Ici » and the proper noun `Qui` « Riri »
- **THEN** the gloss of `io` holds no sense of `Io`, and the gloss of `qui` is « Ici »

#### Scenario: A commune written otherwise
- **WHEN** the section gives `vittoria` « Victoire » and the proper noun `Vittoria` « Vittoria, commune et ville du libre consortium municipal de Raguse… », and `ro` « Rhô » and `Ro` « Ro (commune italienne de la province de Ferrare… »
- **THEN** the gloss of `vittoria` is « Victoire », and the gloss of `ro` « Rhô »

#### Scenario: A provincial capital
- **WHEN** the section gives `matera` the archaic « Matière » and the proper noun `Matera` « Matera, commune et ville de la région de la Basilicate en Italie. » and « Province autour de cette ville. »
- **THEN** the gloss of `matera` keeps both of `Matera`'s senses

#### Scenario: A commune the reviewed list keeps
- **WHEN** the section gives `piacenza` « plaisance, agrément » and the proper noun `Piacenza` « Plaisance, commune italienne », gives no « province autour de cette ville », and the reviewed list keeps `piacenza`'s commune
- **THEN** the gloss of `piacenza` keeps « Plaisance, commune italienne »

#### Scenario: A city named more than a commune
- **WHEN** the section gives `bordeaux` its wine and colour and the proper noun `Bordeaux` « Bordeaux, commune, ville et chef-lieu de département français… », and `colonia` its noun's senses and `Colonia` « Cologne, ville, commune, arrondissement et district d'Allemagne… »
- **THEN** both glosses keep the city

#### Scenario: An article's usage note
- **WHEN** the section defines `del` « Du. Note d'usage : il s'utilise devant un mot commençant par une consonne… »
- **THEN** the gloss of `del` is « Du »

#### Scenario: A conjunction's usage note
- **WHEN** the section defines `sebbene` « Quoique. Note d'usage : suivi d'un verbe au subjonctif. »
- **THEN** the gloss of `sebbene` keeps its note

#### Scenario: An example and its translation
- **WHEN** the section gives `io`'s pronoun the senses « io sono in Francia. — moi, je suis en France. », « Pago io o paghi tu ? — je paie ou tu paies ? » and « io pago. — c'est moi qui paie. »
- **THEN** the gloss of `io` holds none of them

#### Scenario: A note after a dash
- **WHEN** the section defines `mai` « Jamais, en nul temps. — Note d'usage : Se rencontre combiné avec la particule négative non. »
- **THEN** the sense is not read as an example

#### Scenario: A split word that is no Italian
- **WHEN** the translation tables gloss `c'hwerty`, and the section `d'n'b`
- **THEN** neither is an expression of it-fr

### Requirement: it-fr corrects, by name, the slips its sources write
The it-fr reduction SHALL read the entries a reviewed list names as the list corrects them — each keyed by the source, the headword, its part of speech and its text as the pinned files write it, with the corrected text or none and its reason, and reported to the Wiktionary that wrote it — and SHALL apply none whose source text it does not find.
A correction takes its words from a source where one has them (the French tables' « que » for `che`)
and is written by a person otherwise; the owner reads each. A correction may remove a sense or leave
a word with no gloss. An entry a correction adds is read before the section's own entries of the
word, the row's order then set as for any row. An update that finds a correction firing on nothing
names it, and it is removed.

#### Scenario: A meaning the section misses
- **WHEN** the section gives `che` the pronoun « Quoi » and the adjective « Quel », no conjunction and no relative, and the list gives `che` the relative pronoun « Qui, que » and the conjunction « Que »
- **THEN** the gloss of `che` is « Qui, que; Quoi; Que; Quel »

#### Scenario: An example filed as senses
- **WHEN** the section gives `volere`'s verb the senses « Il faut dix heures pour aller à Brindisi. », « Ci vuole un uovo per la torta. » and « Il faut un œuf pour le gâteau. », and the list removes them
- **THEN** the gloss of `volere` holds none of the three

#### Scenario: A participle's lent meaning corrected
- **WHEN** the section's only sense of `dirotto` is « Participe passé de dirompere. », and the list glosses `dirotto` « À verse (dans a dirotto) »
- **THEN** the gloss of `dirotto` is « À verse (dans a dirotto) », not `dirompere`'s « Écanguer »

#### Scenario: A plural noun of its own
- **WHEN** `dati` is a lemma of its own, the section's only noun sense of `dati` is « Pluriel de dato. », which would lend `dato`'s « Donnée », and the list glosses `dati` « Données »
- **THEN** the gloss of `dati` is « Données », as a noun's run

#### Scenario: A sense that misleads a reader
- **WHEN** the section's only sense of `crypto` is « Variante de kripton (l’élément chimique Kr). », which readers meet as the cryptocurrency, and the list removes it; and the section defines `stupidissimo` « Superlatif de stupido : « très léger ». », and the list glosses it « Très stupide »
- **THEN** `crypto` is not glossed « Krypton (l’élément chimique Kr) », and the gloss of `stupidissimo` is « Très stupide »

#### Scenario: The page corrected upstream
- **WHEN** an update finds `volere`'s example no longer filed as senses
- **THEN** the correction fires on nothing, and the update says so

### Requirement: Italian's readings set aside, by name, a reading the section's tables give a form wrongly
Italian's reference reduction SHALL leave out of Italian's readings each reading a reviewed list names — the form, its lemma, the reading's tag and the reason —, or read it as the reading the row gives, and SHALL fail when a row names a reading the reduction does not give.
The list holds a row only where the section's table writes a cell wrongly, or an entry's head names a
form wrongly, and no rule of the readings reads it right; a row's reading keeps its mark, a replacement
is a tag of Italian's pinned pool, and the form's other readings stay. Editing the list re-pins it-fr
alone.

#### Scenario: An imperative read as a subjunctive
- **WHEN** *fare*'s second and third tables in the section list `fàte` beside `facciàte` as the present subjunctive's second person plural, and the list names `fate`'s subjunctive reading
- **THEN** `fate` reads as *fare*'s present indicative and imperative, second person plural, and as no subjunctive

#### Scenario: A plural noun read as a singular
- **WHEN** the English section heads `dati` « data (plural of dato) » with no number, so that its own entry gives `dati` a masculine singular noun's reading, and the list reads that reading as the masculine plural
- **THEN** `dati` reads as a masculine plural noun, and its other readings — *dare*'s participle among them — stay

#### Scenario: A row that finds nothing
- **WHEN** a row of the list names a reading the reduction does not give
- **THEN** the reduction fails, naming the row

## MODIFIED Requirements

### Requirement: Italian is glossed in French from the French Wiktionary's Italian section and the translation tables
it-fr's glosses, sense runs and expressions SHALL be reduced, by the reducer that writes Italian's studied side, through the native side every pair shares with the French Wiktionary's rules, as es-fr's are, from three sources in this order: the French Wiktionary's Italian section's definitions; the Italian Wiktionary's French translations, in the table's order; the French Wiktionary's translation tables read backwards, a French entry that lists an Italian word glossing that word with the French headword, commonest French word first — at most three words per part of speech from a table, a typographic apostrophe in a headword or in an Italian word of either table read as `'`. Every gloss SHALL be French written by a person, never a machine translation nor a pivot through another language. A sense the source tags as a form of a word that no Italian entry of the section holding a meaning heads, by its exact spelling, and whose text the French edition's form-of pattern does not read as an inflection, SHALL gloss its headword as a definition does — where the target is written in lowercase, only if the headword has no other glossed sense once names' and acronyms' senses are set aside; and the pointers and links *it-fr reads a pointer as its word's meaning where the section names one* reads SHALL be read so too. An acronym's sense, and a proper noun's sense that only says the word is a surname, a first name, a hamlet or a commune, SHALL NOT gloss a lemma the section gives a lowercase entry of its own, even one of form-of senses only; any other proper noun's sense SHALL stay beside the lemma's own, but where *it-fr's glosses hold no name, usage note or example a common word does not need, and no split word that is no Italian* leaves it out — under a word whose every lowercase entry is a function word's or an adverb's, where a lowercase entry of a common part of speech repeats it, or where it designates its place as a commune. What the source leaks into a sense SHALL NOT be shown as a gloss: an example and its translation filed as a sense, or a link's address, SHALL NOT gloss; a colour's codes SHALL be left out of the meaning they follow; an entry's head line merged into a sense SHALL give only the meaning written after it, or nothing; a letter's name the section writes bare SHALL NOT gloss a word holding another meaning; and a sense repeating an earlier one of the word, up to case, SHALL be shown once. This pre-pass is it-fr's own: no other pair's tables move with it. Italian's dictionary words SHALL be those of *Italian's dictionary words leave its names out*, the acronym rows glossed as *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* requires, a synonym's gloss read from its entry in the section whatever its part of speech, and the levels SHALL go to the dictionary words as *Italian's estimated levels* requires. it-fr's expressions SHALL be the section's glossed headwords with a space, then those the translation tables gloss, then the words without a space that Italian's pre-pass splits into two tokens or more, beginning with an elided piece, an articulated preposition written whole excepted, and a word whose letters after the piece hold `j`, `k`, `w`, `x` or `y`, or a second apostrophe, excepted too, from the section and from the tables, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires.

#### Scenario: A word glossed by its definition
- **WHEN** it-fr is reduced and `casa` is looked up
- **THEN** its gloss is the section's « Maison; Domicile », as one noun run of two senses

#### Scenario: The fallbacks' order
- **WHEN** an Italian lemma the section does not gloss is listed both in an Italian entry's French translations and as the Italian translation of a French entry
- **THEN** it is glossed by the Italian table's French words first, at most three per part of speech, and by the French headwords read backwards only where that table has none

#### Scenario: A gloss read as a link
- **WHEN** the section's entry `del` carries the sense « Du. » tagged as a form of « Du », which no Italian entry heads, and `storie` the sense « Pluriel de storia » tagged as a form of « Pluriel »
- **THEN** `del`'s gloss opens on « Du », and `storie` stays a form, its text read as an inflection

#### Scenario: A link to a lowercase word beside a gloss
- **WHEN** `pelle` has its own glossed senses and a sense « Article contracté … » tagged as a form of a lowercase word no Italian entry heads
- **THEN** `pelle`'s gloss is its own senses' alone

#### Scenario: What the source leaks into a sense
- **WHEN** the section files `stesso`'s example « io sono stanco… » with its translation as a sense, writes `bianco`'s colour codes after its meaning « Blanc, la couleur comme celle des os. #FFFFFF », merges `sedere`'s head line into its verb sense « \se.ˈde.re\intransitif… * Asseoir. », names `ci`'s Cyrillic letter « Tsy », and glosses `italiano` « Italien » and « italien »
- **THEN** `stesso` is glossed « Même » with no example, `bianco`'s sense reads « Blanc, la couleur comme celle des os », `sedere`'s verb sense « Asseoir », `ci` is glossed as the pronoun alone, and `italiano` « Italien », once

#### Scenario: A link to a word no entry gives a meaning
- **WHEN** `te`'s pronoun sense « Toi : pronom disjoint… » is tagged a form of `toi`, whose own entry is only a form of *togliere*, and `te` has no other glossed sense once its letter and the province `TE` are set aside
- **THEN** `te` is glossed « Toi : pronom disjoint… », not by the tables' « Te »

#### Scenario: A commune keeps off a common word
- **WHEN** the section has `ora` as a lowercase entry and `Ora` as a proper noun « Ora, commune d’Italie… », and `Terra` « La Terre, planète tellurique… » beside `terra`
- **THEN** `ora` is glossed by its own senses alone, and `terra` keeps the planet

#### Scenario: A proper noun beside a common word
- **WHEN** the section has `luna` as a lowercase entry glossed « Lune » and `Luna` as a proper noun « Lune, satellite de la Terre »
- **THEN** `luna`'s gloss keeps « Lune, satellite de la Terre »

#### Scenario: A surname keeps off a common word
- **WHEN** the section has `soldi` as a lowercase entry and `Soldi` as a surname, and `te` as a lowercase entry and `TE` as a province's acronym
- **THEN** neither `soldi` nor `te` is glossed by the surname or the province

#### Scenario: An acronym through a proper-noun synonym
- **WHEN** it-fr glosses the acronym row `USA`, whose synonym `Stati Uniti` the section files as a proper noun
- **THEN** the row's gloss joins `america`'s first sense and `Stati Uniti`'s « États-Unis », in the row's order, as change 11 requires

#### Scenario: The levels go to dictionary words
- **WHEN** it-fr is reduced with its glosses
- **THEN** every levelled lemma of `tables/it/level.tsv` is a dictionary word of `tables/it/lexical.tsv`

#### Scenario: Expressions from every source
- **WHEN** the section glosses `essere d’accordo` (written with the typographic apostrophe) and `all’improvviso`, and the translation tables gloss `meccanica quantica` and `d'accordo`
- **THEN** all four are expressions of `mwe.tsv`, keyed at build

### Requirement: it-fr is committed at its studied tables' snapshot, held to a coverage floor, and the Italian baseline runs on it
it-fr's native side SHALL be reduced from the pin Italian's studied tables were reduced from and the French and Italian editions' files the pin records, the English edition kept, fetching no other source, and committed beside them, every studied table byte for byte but the dictionary words, the levels, and the forms, ranks and readings a change to Italian's forms and readings rules moves, each moved line listed by its rule in that change's pull request; its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, SHALL stay under the size budget, and no package SHALL list it before Italian is enabled. Its coverage — the share of the 5,000, 10,000 and 20,000 commonest Italian lemmas with a gloss — SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["it-fr"]`, fixed before the first committed measurement at the study's figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail. it-fr SHALL gloss at least 87.2, 79.8 and 71.3 % of the 5,000, 10,000 and 20,000 commonest Italian lemmas, the floor so fixed: the study's 89.2, 81.8 and 73.3 % on change 9's committed ranks, with both fallbacks and the section read as this change reads it, less two points. The coverage, the glossed lemmas by source and a sample of glosses marked by source SHALL be shown in it-fr's README and in the sources' record, and stored in no pack. From the pull request that commits it-fr's glosses on, the Italian invariance baseline SHALL run over the pack built from the committed Italian tables, its golden re-blessed once in that pull request.

#### Scenario: The floor holds
- **WHEN** it-fr is reduced and its coverage is at or above `FLOORS["it-fr"]`
- **THEN** the tables are committed and the coverage is shown in the README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces it-fr and its committed tables measure under `FLOORS["it-fr"]`
- **THEN** the checks fail — the floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["it-fr"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The baseline on the committed tables
- **WHEN** the Italian baseline runs after this change
- **THEN** its pack is built from the committed `tables/it/` and `tables/it-fr/`, and its golden is the one this change re-blessed

#### Scenario: Nothing else moves
- **WHEN** it-fr's tables are committed
- **THEN** the six other pairs' tables, pins and packs are byte for byte as before and their baselines pass without re-blessing

### Requirement: Italian's forms and frequencies
Italian's studied tables SHALL be reduced by its reference pair, it-fr, from the English Wiktionary's Italian section as derived from the English edition's dump, and committed once, in `tables/it/`. A form SHALL come from the inflections a lemma's entry lists or from the form-of links of the form's own entry, never from an inflection the dictionary marks alternative, obsolete, archaic, rare, dated, misspelt, nonstandard or abbreviated, nor from a row naming a derived word — a diminutive, augmentative, pejorative, derogatory or endearing form, an adjective's adverb, a noun's relational adjective, a name's demonym or a plural noun's `plural-of` row —, nor from a row of a name's entry, nor from a sense naming its entry such a derived word, nor from a feminine noun's masculine row the form's own entry does not name (`signorini`, *signorino*'s plural, from *signorina*'s table), nor from a row whose form is one of its table's own tags (« archaic », « subjunctive »), nor from a row of a table Italian's readings set aside — the Classical Roman Dialect's, *essere*'s usage notes — where the form's own entry names another ranked word (`avi` and `ava`, *avo*'s plural and feminine), nor from a row of a verb carrying a clitic other than the reflexive one, alone or in a cluster, nor from a gender or number marker an entry lists among its forms, and a form whose candidate is itself a form of another word SHALL reach that word when both links are of one part of speech, unless the form is the candidate's plural and the candidate itself a noun's plural (`viti`, the plural of `vite`, reaches no *vita*; `faccie`, a misspelling of the plural `facce`, still reaches *faccia*). A conjugation-table form SHALL be read without the stress marks the table writes on it — a non-final mark always, a final one only where the entry carrying the verb's form-of link for that cell is the bare spelling —; a table row written with its reflexive pronoun SHALL give its bare form as the candidate, under the plain verb where the section has one and under the reflexive lemma otherwise, a reflexive lemma whose plain verb the section has being no lemma of the tables and a reflexive-only lemma's clitic-bearing rows giving their bare base under it; a multi-word lemma's cells, a canonical row's bracketed auxiliary and a row listing alternatives SHALL give no form. Morph-it! is among the pinned sources — kept by the owner on 2026-10-23, the measurement showing it adds without moving a gate down —: a form it lists under a lemma the tables rank, which the Wiktionary lists under no lemma, SHALL join the table, and a form the Wiktionary reads SHALL never be moved by it. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in MarkIT's training and development sections, then by the form's own entry — which counts only by a meaning when a verb's table gives the form as a person of a mood and the form's own entries hold only that verb's inflection written without its tag, an ellipsis, or another part of speech's pointer the section marks obsolete, archaic, literary, poetic, dated or colloquial (`sto`, `vado`, `sapete`), never for a participle, an alternative infinitive, an abbreviation, a contraction, a function word's homograph, a form another word's entry names as its inflection or a form the section heads as an acronym —, then by the lemma's frequency, then alphabetically; before it, a form whose own lemma entries are names alone and which another word's entry reads SHALL read as that word, and every other form SHALL be left to those rules. The tables SHALL serve Italian's tokenisation as change 6 writes it: every word its pre-pass writes SHALL be a form, each elided piece a form of the word it stands for by the reviewed table, the articulated prepositions of `di` lemmas of their own with `dell'` a form of `dello`, the other articulated prepositions neither forms nor ranks — `dai` excepted, a form of *dare* with no rank, the verb the pre-pass reads `dai` and `da'` as by their context —, each truncation a form of its word, and no plain word beginning with an elided piece nor any verb form carrying clitics a form. The ranks SHALL be wordfreq's Italian order over the 60,000 commonest lemmas whose own form reads as themselves, dense, wordfreq's stems of the elided pieces and truncations that are no word of their own, and its tokens beginning with an elided piece, no words — `cent` and the apocopes written without an apostrophe words —, a hyphenated word ranked only on MarkIT's evidence; each apocope written without an apostrophe SHALL be a form of its word alone by a reviewed table (`un` → *uno*, `bel` → *bello*, `san` → *santo*, `signor` → *signore*; the euphonic `od` → *o* with them), whatever the section's own entries for it say. `tables/it/studied.json` SHALL name it-fr as Italian's reference pair; `tags.tsv` SHALL hold the one canonical tag until change 13's pool, and `lexical.tsv` SHALL hold nothing until change 11's acronym rows and change 16's dictionary words; the it-fr pack SHALL be built from the committed tables, whole and in parts, and checked against its pin in the extension's checks as the six pairs' are; the update workflow SHALL offer it-fr and publish its sources release; no package SHALL list it-fr; and nothing of the six pairs' tables, pins and goldens, nor the Italian baseline's fixture, SHALL move, the Italian baseline's golden only by a re-bless its pull request lists.

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
- **THEN** the six pairs' tables, pins and goldens and the Italian baseline's fixture are byte for byte what they were, and its golden moves only by a re-bless the pull request lists

#### Scenario: Every rank on its own lemma
- **WHEN** the it-fr pack is built from the committed tables
- **THEN** every ranked lemma's own form reads as itself, and the pack gives each lemma its own rank

#### Scenario: A verb's first person read as its verb
- **WHEN** the English section gives `vado` the noun « alternative form of guado », marked obsolete and poetic, and the verb « first-person singular present indicative of andare », and `sapete` only « (you) know », an inflection of *sapere* written without its tag
- **THEN** `vado` is a form of *andare* and `sapete` of *sapere*, and neither is a lemma of its own

#### Scenario: What the own-entry rule leaves
- **WHEN** `dee` is the plural of `dea` and an old form of `deve`, `cogli` the contraction of `con gli` and a form of *cogliere*, and `elisa` the feminine of the participle `eliso`
- **THEN** each keeps the lemma the other rules give it, none read as *dovere*, *cogliere* or *elidere* by this rule

#### Scenario: No chain through a plural
- **WHEN** the English section gives `viti` as the plural of `vite`, and `vite` as a noun of its own and as the plural of `vita`, which reads `vite` as its form
- **THEN** `viti` is no form of *vita*, and leaves the tables

#### Scenario: A chain through a feminine
- **WHEN** `maestre` is the plural of `maestra`, the feminine of `maestro`
- **THEN** `maestre` is still a form of *maestro*

#### Scenario: A feminine noun's masculine row
- **WHEN** `signorina`'s table lists `signorini` as its masculine plural, `signorini` being the plural of `signorino`, and *eco*'s lists its masculine plural `echi`, whose own entry names *eco*
- **THEN** `signorini` is a form of *signorino*, and `echi` still of *eco*

#### Scenario: The dialect's table
- **WHEN** *avere*'s Classical Roman Dialect table lists `avi` and `ava`, whose own entries name *avo*, and *essere*'s lists `son`, whose own entry names `sono`
- **THEN** `avi` and `ava` are forms of *avo*, and `son` still of *essere*

#### Scenario: A table's tag is no form
- **WHEN** *essere*'s usage notes list the rows « archaic » and « subjunctive » as cells
- **THEN** neither is a form of the tables

#### Scenario: An override row
- **WHEN** the override list reads `usa` as *usare*, for its reason, and the adjective `uso` lists `usa` as its feminine
- **THEN** `usa` is a form of *usare*

#### Scenario: A plural noun of its own
- **WHEN** the override list reads `dati` as *dati*, the plural noun the English section heads « data », and `dato` reads as *dare*, MarkIT counting it 9 times so
- **THEN** `dati` is a lemma of its own with its own rank, and `dato` stays a form of *dare*

#### Scenario: An apocope and a euphonic form
- **WHEN** `signor` is « Variante de signore » and `od` the euphonic `o` before a vowel
- **THEN** `signor` reads as *signore* and `od` as *o* by the reviewed table, and neither is ranked

#### Scenario: The gates after the forms
- **WHEN** it-fr's tables are reduced with these rules
- **THEN** UD Italian-PUD reads at least 98.5 % of its words, 90.5 % of its content words and 97 % of its auxiliaries

### Requirement: Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms
Italian's reference reduction (it-fr's) and every pair glossing Italian SHALL read an Italian word as an acronym only from one reviewed table of an Italian rule module every pair glossing Italian imports, each row naming the acronym as written — wholly in capitals, two letters or more —, the expansions and the synonyms the English Wiktionary's Italian section's own entry for it gives, its part of speech and its reason; a row SHALL exist only where the section has an entry headed by the acronym and the acronym's lowercase is a form of another word of the tables, and a row SHALL be added only as a person's decision. The reference reduction SHALL write each row into Italian's studied tables as a form that is its own lemma, its capitals kept, with no rank and no level, the lowercase spelling keeping its own reading (`nato` stays a form of *nascere*), and as one of Italian's dictionary words whatever its sense runs and whether glossed, from the first reduction holding the row on. A pair SHALL gloss a row from its own glosses of the row's expansions, each cut to its first sense, in the row's order, an expansion it does not gloss passed over; when it glosses no expansion, from its own glosses of the row's synonyms the same way; and SHALL write no gloss when it glosses neither — never a gloss of its own words, a translation, or the entry's English. Editing the module SHALL re-pin it-fr, and the six other pairs' tables, pins, packs and invariance baselines SHALL NOT move.

#### Scenario: The rows
- **WHEN** it-fr is reduced with the reviewed table holding `NATO` and `USA`
- **THEN** `tables/it/forms.tsv` maps `NATO` to `NATO` and `USA` to `USA`, the forms holding a capital, `freq.tsv` and `level.tsv` hold neither, and `nato` and `usa` keep the readings the tables give their lowercase spelling, *nascere* and *usare*

#### Scenario: A pair glosses the synonyms
- **WHEN** it-fr glosses `USA`, whose entry points at the English name and names the synonyms `America` and `Stati Uniti`
- **THEN** the row's gloss is it-fr's own glosses of `america` and `stati uniti`, each cut to its first sense, joined

#### Scenario: No expansion or synonym glossed
- **WHEN** it-fr glosses `NATO`, whose entry names no expansion and no synonym
- **THEN** it writes no gloss for the acronym, which stays a form of the pack and one of Italian's dictionary words

#### Scenario: A row is a decision
- **WHEN** the table is read
- **THEN** every row is written wholly in capitals, carries its reason, is headed by an entry of the section, and names only expansions and synonyms that entry gives; a row with no entry (`ANSA`) fails the reduction naming it
