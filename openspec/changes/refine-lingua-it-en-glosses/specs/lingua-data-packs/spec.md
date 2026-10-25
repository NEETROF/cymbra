# Spec Delta

## ADDED Requirements

### Requirement: it-en's glosses read an Italian word's meanings in the order Italian uses them
The it-en reduction SHALL open a function word's row on the part of speech UD Italian-MarkIT's training and development sections read the word as at least ten times and at least twice as often as the part of speech the section opens the row on, never on a proper noun's; SHALL open a row whose first meaning the section marks archaic, obsolete, dated or historical on its first meaning, in the section's order of entries and senses, that carries none of those four marks; and SHALL open a row a reviewed table names on the sense the table gives it, whatever the treebank or the marks read.
The function words' parts of speech the treebank may move first are `ADP`, `DET`, `PRON`, `CCONJ`,
`SCONJ`, `PART` and `ADV`; a noun, a verb or an adjective the treebank reads more often than the noun,
verb or adjective the section opens on does not move. The treebank's counts are it-fr's: a word inside a
fixed expression counts for none, a noun, a verb, an adjective or a proper noun counts under its lemma
and any other part of speech under its own form; it-en's pin records MarkIT's two sections at the commit
and sha256s it-fr's pin records, and reads them for these counts alone. A row the marks move opens on
that meaning, its entry first, every other sense in its place; a row whose every meaning carries one of
the four marks keeps the section's order; a mark other than the four (rare, uncommon, literary, poetic,
regional) moves nothing. The marks' rule orders an expression's entries as a word's; the treebank's
rule and the table order words alone. The reviewed table holds a row only for a row the rules do not
move or move wrongly, each with its reason, decided by the owner; its key is the headword's case, a part
of speech and the sense the row opens on — a sense of the section, or one the section nests under
another, then read by its own text —, and the reduction fails on a row whose sense it does not find. The
treebank's order applies first, then the marks', then the table's, which reads the senses a short
word's pointer lends; no order adds or removes a sense. These are rules of it-en's own modules: they re-pin it-en and no other pair,
Italian's studied tables do not move, its coverage stays at least its floor, and the it-en baseline is
re-blessed once, in the pull request that commits them.

#### Scenario: A function word's commonest part of speech
- **WHEN** the section gives `su` the adverb « up, upstairs » before the preposition « on, upon, onto, on top », and MarkIT reads `su` 92 times as a preposition and once as an adverb
- **THEN** the gloss of `su` opens on « on, upon, onto, on top »

#### Scenario: A content word keeps the section's order
- **WHEN** the section opens `piacere` on the verb « to be pleasing to » and MarkIT reads it 19 times as a noun and 6 times as a verb
- **THEN** the gloss of `piacere` still opens on the verb

#### Scenario: An outdated first sense
- **WHEN** the section gives `polizia` the sense « polity », marked archaic, before « police »
- **THEN** the gloss of `polizia` is « police; polity »

#### Scenario: A mark the rule does not read
- **WHEN** the section marks `grande`'s first adjective senses rare, and none of its senses archaic, obsolete, dated or historical
- **THEN** the gloss of `grande` still opens on the adjective's first sense, not on the adverb « really (intensifier) »

#### Scenario: A row the reviewed table orders
- **WHEN** the section gives `testo` a noun entry opening on « an earthenware lid » before the noun entry « text (writing) », and the reviewed table names `testo`'s « text (writing) »
- **THEN** the gloss of `testo` opens on « text (writing) »

#### Scenario: A row the table holds against the treebank
- **WHEN** MarkIT reads `come` 39 times as a subordinating conjunction and 16 times as an adverb, the part of speech the section opens it on, and the reviewed table names `come`'s adverb « how »
- **THEN** the gloss of `come` still opens on « how »

#### Scenario: A row whose every sense is marked
- **WHEN** the section marks every meaning of a row archaic, obsolete, dated or historical, and the reviewed table names no row for it
- **THEN** the row keeps the section's order

#### Scenario: A row the table opens where every sense is marked
- **WHEN** the section marks every sense of `gente` archaic, its adjective « noble » first, and the reviewed table names its noun « people, guys, folks (body of human beings) »
- **THEN** the gloss of `gente` opens on « people, guys, folks (body of human beings) »

#### Scenario: A sense nested under another
- **WHEN** the section nests « parliament » under `parlamento`'s « public assembly or meeting », and the reviewed table names `parlamento`'s « parliament »
- **THEN** the gloss of `parlamento` opens on « parliament », then « public assembly or meeting »

#### Scenario: An expression's outdated sense
- **WHEN** the section gives the expression `a vanvera` « at random, without any effort to achieve particular outcome », marked dated, before « without thinking »
- **THEN** the gloss of `a vanvera` opens on « without thinking »

#### Scenario: A sense the table no longer finds
- **WHEN** an update finds the section no longer writing a sense a row of the reviewed table names
- **THEN** the reduction fails, naming the row

#### Scenario: Nothing else moves
- **WHEN** it-en is reduced again with these rules
- **THEN** `tables/it/` and the seven other pairs' tables, pins, packs and goldens are byte for byte as before, and it-en's coverage is at least `FLOORS["it-en"]`

### Requirement: it-en reads a pointer as its word's meaning where the section names one
The it-en reduction SHALL read as a pointer to its word a sense the section writes as a labelled form or spelling of a word, a clitic compound or a truncation, and an alternative form whose word no field names; SHALL let a pointer to an Italian word of one or two letters lend that word's meanings of its part of speech, after the row's own senses, when its entry holds no meaning of its own, the shared rules lend the word nothing, and the section does not tag the pointer archaic, obsolete, dated or a misspelling; and SHALL read a meaning nested under a comparative's or a superlative's wording by its own gloss.
A labelled form or spelling names its word last, or before a colon, after labels — archaic, regional,
dialectal, obsolete, Old Italian, proscribed, censored, dominant, a place's name — and « form of » or
« spelling of »; a clitic compound's meaning written after its closing full stop is read as a sense. A
pointer lends as the rules every pair shares lend — its word's senses in the same part of speech — from a
word of three letters or more; a word of one or two letters lends here instead. Whether a pointer may
lend is read from the section's tags, not from the labels its wording carries. Lent senses come after
the row's own, never more than the room the eight-sense bound leaves them, none the section tags
regional, dialectal, archaic, obsolete or dated, and none whose words a sense of the row already says,
so that no sense of the row's own is pushed past the bound. A « female equivalent of » pointer lends
nothing beside the word's own senses: a female equivalent is no word's meaning, and the masculine's
English is often a man's. A word with other glossed senses keeps them.

#### Scenario: A proscribed spelling
- **WHEN** the section gives `che` the conjunction's sense « proscribed spelling of ché » beside its own senses
- **THEN** the gloss of `che` does not hold « proscribed spelling of ché »

#### Scenario: A regional form of a short word
- **WHEN** the section's `de` holds « alternative form of di » and « Marche, Milan, Rome, and Venice form of di » as prepositions, and no other sense
- **THEN** the gloss of `de` is `di`'s prepositional senses

#### Scenario: A pointer whose wording names a label it is not tagged with
- **WHEN** the section gives the article `el` « archaic, regional, and Rome form of il », with no tag marking it archaic, beside the pronoun « he »
- **THEN** the gloss of `el` is « he; the »

#### Scenario: Lent senses after the row's own
- **WHEN** `ne` holds seven senses of its own and the preposition « alternative form of in »
- **THEN** the gloss of `ne` holds its seven senses, then `in`'s « in » alone, the one sense the bound leaves

#### Scenario: A word the shared rules lend to
- **WHEN** the section's `je` holds only « alternative form of gli (…) » and « alternative form of le (…) », and `gli` lends its pronoun's senses
- **THEN** the gloss of `je` is `gli`'s « him, to him, it, to it; … », and `le`'s senses lend nothing

#### Scenario: A short word's obsolete form
- **WHEN** the section gives `il` the pronoun « Tuscany form of lo », marked obsolete, beside its article « the »
- **THEN** the gloss of `il` holds none of `lo`'s pronoun senses

#### Scenario: A clitic compound
- **WHEN** the section gives `aprile` « April » and the verb « compound of apri, the second-person singular imperative form of aprire, with le »
- **THEN** the gloss of `aprile` holds « April » and not the compound's wording

#### Scenario: A meaning nested under a degree
- **WHEN** the section nests « minimum; (the) least, (the) smallest… » and « very small, very little, tiny… » under `minimo`'s « superlative degree of piccolo »
- **THEN** the gloss of `minimo` opens on « minimum, (the) least, (the) smallest… », and holds no « superlative degree of »

#### Scenario: A female equivalent beside the word's own senses
- **WHEN** the section gives `signora` « female equivalent of signore » beside its own senses « lady » and « a title of address for women »
- **THEN** the gloss of `signora` holds none of `signore`'s senses (« lord (ruler) »)

### Requirement: it-en's glosses hold no name's note or page note a common word does not need
The it-en reduction SHALL leave a proper noun's sense that only says its word is a surname out of a row whose word holds a common meaning; SHALL leave a proper noun's sense that only says its word is a given name out of a row whose first meaning is a common word's; SHALL leave a proper noun's sense naming its place a town, a comune, a village, a hamlet, a frazione or a municipality out of a word with a common meaning, unless the row opens on a proper noun's sense, the sense calls the place itself a city or a capital, or a town and province, or a reviewed list keeps it; SHALL leave no such note out of a row whose every lowercase meaning the section tags archaic, obsolete, dated, historical, rare, poetic, literary, regional or dialectal; and SHALL read an article's sense describing its own form by its last clause.
A word holds a common meaning when a lowercase entry of it holds a meaning, or when a lowercase pointer
of it lends once none of its proper nouns' senses stays; a row holding only proper nouns' senses keeps
its surname's note, and a given name's note that opens its row stays. A place called a city or a
capital is the place itself (« a city and comune », « the capital of »), never a province it lies in
(« the Metropolitan City of … »); a commune is a place only before « in » or « of » (« a commune of
Liguria »), never a revolutionary government (« Commune (of Paris) »). Whether a lowercase meaning is
so marked is read from the section's tags, never from its wording; the notes such a row keeps stay
where the section writes them. A proper noun whose senses only point at another proper noun reads
that name's senses as the section writes them, before these rules, its own row then read by them. The
reviewed list holds a commune, each with its reason, only where the section names a provincial capital
without saying so, or where readers meet the word as the place or its wine, game or abbey, decided by
the owner. A proper noun's entry left with no sense goes. The reduction
also writes « etc » with its period in glosses and expressions, leaves out a list marker opening a
sense, and reads a sense opening on a quoted meaning in parentheses (« (“sea”) … ») as that meaning
followed by the rest.

#### Scenario: A surname after a common word
- **WHEN** the section gives `musica` « music » and the proper noun `Musica` « a surname »
- **THEN** the gloss of `musica` is « music »

#### Scenario: A surname alone
- **WHEN** the section's only sense of `mattarella` is the proper noun's « a surname »
- **THEN** the gloss of `mattarella` is « a surname »

#### Scenario: A surname beside another name
- **WHEN** the section gives `garibaldi` the proper nouns' senses « a surname » and « Anita Garibaldi, Italian patriot… », and no lowercase entry
- **THEN** the gloss of `garibaldi` keeps « a surname »

#### Scenario: A given name after a common word
- **WHEN** the section gives `delfino` « dolphin », « dauphin » and « hunchbacked », and the proper noun `Delfino` « a male given name », after them
- **THEN** the gloss of `delfino` holds no given name

#### Scenario: A given name opening its row
- **WHEN** the section gives `Alice` « a female given name, equivalent to English Alice » before the noun `alice` « anchovy »
- **THEN** the gloss of `alice` keeps the given name

#### Scenario: A town beside a common word
- **WHEN** the section gives `noto` « of common knowledge; well-known, known » and the proper noun `Noto` « a town in Syracuse, Sicily, Italy »
- **THEN** the gloss of `noto` holds no town

#### Scenario: A town opening its row
- **WHEN** the section gives the proper noun `Cervia` « a town in Ravenna, Emilia-Romagna, Italy » before the noun `cervia` « hind (female deer) »
- **THEN** the gloss of `cervia` keeps the town

#### Scenario: A province's metropolitan city
- **WHEN** the section gives `russo` « Russian » and the proper noun « a frazione in Rizziconi, Metropolitan City of Reggio Calabria, Calabria, Italy »
- **THEN** the gloss of `russo` holds no town

#### Scenario: A name pointing at a name
- **WHEN** the section's `Pinuccio` only points at `Pino`, whose given name's note `pino`'s row leaves out beside « pine »
- **THEN** the gloss of `pinuccio` keeps « a diminutive of the male given name Giuseppe… »

#### Scenario: A row whose common meanings are all marked
- **WHEN** the section gives the proper noun `Esposito` « a common surname » before the noun `esposito` « foundling », tagged obsolete
- **THEN** the gloss of `esposito` is « a common surname; foundling »

#### Scenario: A commune named as a government
- **WHEN** the section gives `comune` « comune, municipality » and the proper noun `Comune` « Commune (of Paris, properly “Comune di Parigi”) »
- **THEN** the gloss of `comune` keeps « Commune (of Paris, properly “Comune di Parigi”) »

#### Scenario: A provincial capital
- **WHEN** the section gives `fermo` « still; firm » and the proper noun `Fermo` « a town and province of Marche, Italy »
- **THEN** the town stays in the gloss of `fermo`

#### Scenario: A town the reviewed list keeps
- **WHEN** the section gives `massa` « mass; crowd » and the proper noun `Massa` « Massa (a town in Tuscany, Italy) », and the reviewed list keeps `massa`'s town
- **THEN** the gloss of `massa` keeps « Massa (a town in Tuscany, Italy) »

#### Scenario: An article's usage note
- **WHEN** the section defines `lo` « the form of il that is used before the so-called impure consonants…; before a vowel it becomes l'; the »
- **THEN** the gloss of `lo` opens on « the »

#### Scenario: The page's typography
- **WHEN** a sense ends on « etc », another opens on « # », and `mar` reads « (“sea”) (used in poetry and in names of some seas) »
- **THEN** the first ends on « etc. », the second holds no « # », and the gloss of `mar` is « sea (used in poetry and in names of some seas) »

### Requirement: it-en corrects and completes, by name, what its sources write
The it-en reduction SHALL read the entries a reviewed list names as the list corrects or completes them — each keyed by the source, the headword, its part of speech and its text as the pinned files write it, with the corrected text, a sense to add, or none, and its reason, and reported or suggested to the Wiktionary that wrote it — and SHALL apply none whose source text it does not find.
A correction takes its words from a source where one has them and is written by a person otherwise;
the owner reads each. A row may add a meaning the section leaves out (a feminine noun's person, which
its « female equivalent of » pointer does not lend), not only remove or rewrite a slip. An entry a correction adds is read before the section's own entries of the word,
the row's order then set as for any row. An update that finds a correction firing on nothing names it,
and it is removed. No correction glosses an acronym row, which its own requirement glosses.

#### Scenario: A jocular spelling among the translations
- **WHEN** the Italian Wiktionary lists « ancient », « anshent » and « antique » as `all'antica`'s English translations, and the list removes « anshent »
- **THEN** the gloss of `all'antica` is « ancient, antique »

#### Scenario: A meaning the section leaves out
- **WHEN** the section gives `amica` « female equivalent of amico » and « girlfriend », and the list adds « friend » to `amica`'s noun
- **THEN** the gloss of `amica` opens on « friend », then « girlfriend »

#### Scenario: The page corrected upstream
- **WHEN** an update finds `all'antica`'s translations no longer holding « anshent »
- **THEN** the correction fires on nothing, and the update says so

## MODIFIED Requirements

### Requirement: Italian is glossed in English from the English Wiktionary's Italian section and the Italian Wiktionary's English translations
it-en's glosses, sense runs and expressions SHALL be reduced by a reducer of its own, reading Italian's studied tables as committed, through the native side every pair shares with the English Wiktionary's rules, from the English Wiktionary's Italian section's definitions, then the Italian Wiktionary's English translations, at most three words per part of speech, and never from the English Wiktionary's translation tables read backwards. Every gloss SHALL be English written by a person. After the English edition's pre-passes and before the shared rules, it-en's own pre-pass SHALL read the meaning a pointer sense carries after its pointer, by a reviewed list of wordings — Italian's own shapes (a form for euphony, the article's form, a pronoun's clitic case form, the `di` family's contraction), a comparative's or a superlative's and a clipping's wherever they stand, a plural's where the word's own meanings are of another part of speech only, and a contraction's or a plural's otherwise only for a word whose lowercase entries hold nothing else, the meaning read from the gloss's text; SHALL keep an acronym's sense off a lemma the section gives a lowercase entry of its own unless that entry only points at the acronym, and a name's run off a word with a function word's entry, while a name beside a common word's senses is kept but for the notes *it-en's glosses hold no name's note or page note a common word does not need* leaves out; SHALL gloss no word with a letter's name, Latin, or Greek where a common word's entry of the word holds another meaning; and SHALL take out notes that are not glosses. A sense repeating an earlier one of the same word word for word SHALL be shown once, a name's sense written with a capital kept beside a common word's. The acronym rows of Italian's studied tables SHALL be glossed by *Italian's acronyms are a reviewed table, glossed through their entry's expansions or synonyms* from it-en's own glosses. it-en's expressions SHALL be the section's glossed headwords with a space, then those the Italian Wiktionary's English translations gloss, then the words without a space that Italian's pre-pass splits and the section or the Italian Wiktionary glosses with a meaning, each keyed at build as *An Italian pack keys its expressions as Italian is read* requires. The English edition's shared rules, and every other pair's reducer, tables and pin, SHALL NOT move.

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

#### Scenario: A degree beside the word's own senses
- **WHEN** the section's `più` holds the adverb's « comparative degree of molto; more » before « anymore, any longer »
- **THEN** the gloss of `più` opens on « more »

#### Scenario: A clipping
- **WHEN** the section's `auto` holds « clipping of automobile: car » and « clipping of autobus: bus » beside the historical « auto (genre of dramatic literature) »
- **THEN** the gloss of `auto` opens on « car; bus »

#### Scenario: A plural beside another part of speech
- **WHEN** the section's `guai` holds the interjection « woe! » and the noun « plural of guaio (“trouble”) », and no noun sense of its own
- **THEN** the gloss of `guai` is « woe!; trouble »

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
