## ADDED Requirements

### Requirement: fr-en's glosses read a French word's meanings in the order French uses them
The fr-en reduction SHALL read a pointer sense that carries its meaning as that meaning — in a word's gloss when the pointer names a degree of comparison, a synonym, a plural or a contraction, in an expression's whatever it names —, SHALL open a word's gloss on the part of speech UD French-GSD's training and development sections read the word as at least ten times and at least twice as often as the part of speech the page opens on, never on a proper noun's, when that part of speech is a function word's or the page opens the gloss on a proper noun's, SHALL leave a name's senses out of a word whose every other entry is a function word's, SHALL leave out an expression whose one sense needs a context its key does not hold, and SHALL gloss a post-1990 spelling that is keyed apart from its traditional spelling with that spelling's gloss.
A pointer's meaning is the one the section records for its target, or writes after it in quotation
marks or past a colon or a semicolon; a pointer that carries none, a name's, an acronym's and a
meaning written only in capitals stay pointers, as before. A word's other pointers — a female
equivalent, an alternative form, a spelling, a feminine singular, an ellipsis, a clipping — stay
pointers. The treebank's
counts are the ones fr-en's pin already records; a word inside a fixed expression of the treebank
counts for none, a noun, a verb, an adjective or a proper noun counts under its lemma and any other
part of speech under its own form; the entries of the commonest part of speech come first, the
others keep their order, and a word the treebank meets less often keeps the page's order. The
function words' parts of speech the treebank may move first are `ADP`, `DET`, `PRON`, `CCONJ`,
`SCONJ`, `PART` and `ADV`; a noun, a verb or an adjective that the treebank reads more often than
the noun, verb or adjective the page opens on does not move. A word's name is left out under a
preposition, a conjunction, a pronoun, a determiner, an article or a particle.
The expressions left out are listed by name, each with its reason, as `à la` is. These are rules
of fr-en's own reducer, which no other pair loads: they re-pin fr-en alone, and its coverage stays
at least its floor.

#### Scenario: An article's pointers
- **WHEN** the section gives `des` only pointers — « plural of un (“some”, the plural indefinite article) » and its partitive siblings, and « contraction of de + les, literally “of the, from the, some” » — and `du` « forms the partitive article » beside « contraction of de + le, literally “of the” »
- **THEN** `des`'s gloss is « some; of the, from the, some », borrowed from no other word, and `du`'s holds « of the » after « forms the partitive article »

#### Scenario: A degree of comparison
- **WHEN** the section glosses `mieux` « comparative degree of bien; better », « superlative degree of bien; best » and « more, -er. », and `moins` « comparative degree of peu; less, fewer » beside « minus; negative »
- **THEN** `mieux`'s gloss opens on « better; best », and `moins`'s on « less, fewer »

#### Scenario: A synonym
- **WHEN** the section glosses `ouais` « synonym of oui; yeah, yep, yup, yes, affirmative expression » beside its interjection « wow »
- **THEN** `ouais`'s gloss opens on « yeah, yep, yup, yes, affirmative expression »

#### Scenario: An expression's pointer
- **WHEN** the section glosses « il y a » « impersonal singular present indicative of y avoir: there is, there are » beside its preposition « ago »
- **THEN** fr-en glosses « il y a » « there is, there are; ago »

#### Scenario: Another pointer of a word stays a pointer
- **WHEN** the section glosses `y` « alternative form of il; he » beside « there (at a place) », and `directrice` only pointers, the first « female equivalent of directeur: directress »
- **THEN** `y`'s gloss holds « there (at a place) » and no « he », and `directrice`'s is `directeur`'s « director; school principal », as before

#### Scenario: A function word's commonest part of speech
- **WHEN** the page opens `pas` on its noun « step, pace, footstep », `son` on its noun « sound » and `leur` on its pronoun « (to) them », and the treebank reads `pas` 981 times as an adverb against 8 as a noun, `son` 1,506 times as a determiner against 19, `leur` 440 times against 50 as a pronoun
- **THEN** `pas`'s gloss opens on its negation, `son`'s on « his, her, their, its », and `leur`'s on « their »

#### Scenario: A common word before a place's name
- **WHEN** the page opens `marche` on « Marche (a department of France) » before its noun's « march », and the treebank reads `marche` 19 times as a noun and never as a proper noun
- **THEN** `marche`'s gloss opens on « march (formal, rhythmic way of walking) », and the department comes after the noun's senses

#### Scenario: A noun, a verb and an adjective keep the page's order
- **WHEN** the page opens `ferme` on its adjective « firm » and `devoir` on its noun « duty », and the treebank reads `ferme` more often as a noun and `devoir` as a verb
- **THEN** `ferme`'s gloss opens on « firm » and `devoir`'s on « duty », as before

#### Scenario: Too little evidence
- **WHEN** the treebank reads `même` 262 times as an adjective and 173 as an adverb, not twice as often, and the noun `phare` 8 times, fewer than ten
- **THEN** `même`'s gloss opens on « even » and `phare`'s on « leading, signature, key, flagship », as before

#### Scenario: A name is never promoted
- **WHEN** the treebank reads `Jean` 91 times as a proper noun and the page opens `jean` on « a pair of jeans »
- **THEN** `jean`'s gloss opens on « a pair of jeans », as before

#### Scenario: A function word's homograph name
- **WHEN** the section has a surname `Le` and a village `On` beside the article and pronoun `le` and the pronoun `on`
- **THEN** `le`'s gloss holds no « a surname from Vietnamese » and `on`'s no « a village in Luxembourg, Belgium »

#### Scenario: An expression whose sense needs a context
- **WHEN** the section glosses `et des` « or thereabouts, and change » and `un coup` « used to soften an order; once, one time », which UD French-GSD's 204 « et des » and 11 « un coup » never mean
- **THEN** fr-en has no expression `et des` or `un coup`, « du pain et des œufs » meets no expression, and « un coup d'œil » meets `coup d'œil` alone

#### Scenario: A post-1990 spelling keyed apart
- **WHEN** the section's `à priori` only says « post-1990 spelling of a priori », and the forms table reads `à` and `a` as two words
- **THEN** fr-en glosses `à priori` as it glosses `a priori`, « intuitively known, a priori; at first glance; preconceived idea »

#### Scenario: Nothing else moves
- **WHEN** fr-en is reduced again from its pin with these rules
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte unchanged, fr-en's pin keeps its snapshot and sources, and its coverage of the 5,000, 10,000 and 20,000 commonest lemmas is at least 91.9, 85.1 and 74.4 %

### Requirement: fr-en's glosses carry none of the page's notes
The fr-en reduction SHALL leave out of a sense the English Wiktionary's notes to its reader — « see usage notes », « (all senses) », « in its various senses », a folk etymology in parentheses, the citation of a quotation, a source's sense number in parentheses —, SHALL write in lower case a sense that opens on one of fr-en's description openers, and SHALL write « etc » with its period.
The openers are a closed list: « Substitutes », « Impersonal », « Followed », « Adverbial »,
« Designating », « Stresses », « Representing », « Indicating », « Names », « Describing »,
« Exclamation » and « Found », followed by a space and a lower-case letter or a parenthesis; any
other capital stays — a proper adjective, a language, a demonym, and the definitions the page writes
with a capital (« Military rank equivalent to corporal »). A citation is cut where a period is
followed by a year and a comma; a sense number is one digit in parentheses. A note left with nothing
around it leaves no empty parenthesis. The rules hold for the glosses of words and of expressions
alike, and are fr-en's own: es-en, which reads the same edition, does not move.

#### Scenario: Usage notes
- **WHEN** the section glosses `en` « in (used to indicate space, also see usage notes) » and `ne` « not (used alone to negate a verb, now chiefly with only a few particular verbs; see usage notes) »
- **THEN** their glosses read « in (used to indicate space) » and « not (used alone to negate a verb, now chiefly with only a few particular verbs) »

#### Scenario: All senses
- **WHEN** the section glosses `contrôle` « control (all senses) » and `consul` « consul, in its various senses »
- **THEN** their glosses read « control » and « consul »

#### Scenario: A folk etymology and a citation
- **WHEN** a sense of `mon` ends on « (Folk etymology: military-specific short for “monsieur”.) », and `liberté` is glossed « liberty, freedom. 1688, Guy Miège, The Great French Dictionary. “Qu'y a-t-il de plus doux…” »
- **THEN** `mon`'s sense ends on « within the military », and `liberté`'s gloss is « liberty, freedom »

#### Scenario: A description in a capital
- **WHEN** the section glosses `que` « Substitutes for another, previously stated conjunction » and `il` « Impersonal subject, it »
- **THEN** their glosses hold « substitutes for another, previously stated conjunction » and « impersonal subject, it »

#### Scenario: A capital that is no description
- **WHEN** a sense reads « German person », « Military rank equivalent to corporal » or « Swiss (of, from or relating to Switzerland) »
- **THEN** it keeps its capital

#### Scenario: A source's sense number
- **WHEN** the section glosses `téléphonie` « telephony (2) »
- **THEN** its gloss opens on « telephony »

#### Scenario: Etc with its period
- **WHEN** the shared rules write `le`'s « the, my, your, etc » and `pas`'s « …doesn't, etc », having taken off a sense's final period
- **THEN** they read « the, my, your, etc. » and « …doesn't, etc. »

### Requirement: A French level is given only to a French dictionary word
The fr-en reduction SHALL give no estimated level to a ranked French lemma that is not one of French's dictionary words, the six levels keeping English's sizes, and the checks SHALL fail, naming the lemma, when a French level is given to a lemma that is no dictionary word of French.
Which lemmas a CEFR list would hold is read as *French's estimated levels* reads it, from the
English Wiktionary's French section; a lemma French's dictionary words do not list — one fr-en does
not gloss, met only in an expression (`parce`, `quant`), an initialism the section writes in
capitals (`pme`), a pointer (`expliquez`), or one it glosses by a name's senses alone (`coran`) — is
left out as well, so that a card seeded from a level always carries a gloss. Leaving a lemma out
gives its place to the next one in rank order, so that the six levels keep 1,020, 1,158, 2,015,
2,347, 886 and 876 lemmas. From this change on, a change to fr-en's glosses that adds or removes a
dictionary word can move French's level table.

#### Scenario: A word met only in an expression
- **WHEN** fr-en is reduced, `parce` is ranked 103rd and fr-en glosses `parce que` but not `parce`
- **THEN** `parce` has no level, and `parce que` keeps its gloss

#### Scenario: A word glossed only as a name
- **WHEN** the section's `coran` only says « alternative form of Coran », so that fr-en glosses it by the proper noun's « Koran » alone
- **THEN** `coran` is no dictionary word of French and has no level

#### Scenario: English's sizes
- **WHEN** fr-en is reduced with these rules
- **THEN** `tables/fr/level.tsv` holds 8,302 lemmas, as many at each level as before, and every one of them is listed by `tables/fr/lexical.tsv`

#### Scenario: A level for a word with no gloss
- **WHEN** a French level table gives `parce` A1 while French's dictionary words do not list it
- **THEN** the checks fail, naming `parce`

## MODIFIED Requirements

### Requirement: A pack's dictionary words do not depend on its glosses
A pack SHALL hold the same dictionary words — the lemmas its vocabulary sizes count and the Spanish names rule keeps as words — whatever native language it is glossed in: the lemmas its studied language's reference pack glosses — en-fr for English, es-fr for Spanish, and for a language studied later the first pack built for it —, less, for French, the lemmas fr-en glosses by a proper noun's senses alone, which are names, not words to learn.
A pack whose glossed lemmas are not its dictionary words SHALL name them in a lexical table, an
optional section that a core which does not read it ignores. A pack without one SHALL read its
glossed lemmas as its dictionary words, so the en-fr and es-fr packs carry no lexical table and keep
their bytes; fr-en, whose dictionary words leave out the lemmas it glosses as names alone, carries
one. A lemma glossed by a proper noun's senses alone is one every sense run of whose gloss is a
proper noun's; English's and Spanish's dictionary words keep theirs. The builder SHALL read a pack's
dictionary words from an optional `lexical.tsv` beside its tables and SHALL write the lexical table
only when they differ from the lemmas the pack glosses. When it writes a lexical table, it SHALL
refuse, naming the lemma, a dictionary word or a glossed lemma that is neither the lemma of a form
nor a ranked lemma, so that no native language's glosses can add a lemma to the lexicon.

The checks SHALL fail when two packs of one studied language, built from committed tables, hold
different dictionary words.

#### Scenario: The shipped packs carry no lexical table
- **WHEN** the en-fr and es-fr packs are built from their committed tables after this change
- **THEN** neither carries a lexical table, and each sha256 is the one its pin records

#### Scenario: A pack glossed in English
- **WHEN** a pack studying Spanish is built from the es-fr studied tables, with English glosses that gloss `augusto`, and a `lexical.tsv` listing the lemmas es-fr glosses
- **THEN** it carries a lexical table, `casa` is one of its dictionary words, and `augusto`, which es-fr does not gloss, is not

#### Scenario: A lexical table that says what the glosses say
- **WHEN** a pack's `lexical.tsv` lists exactly the lemmas it glosses
- **THEN** the pack is byte for byte the one built without that file

#### Scenario: A gloss outside the lexicon
- **WHEN** a pack with a `lexical.tsv` glosses a lemma that no form maps to and no rank lists
- **THEN** the build fails and names the lemma

#### Scenario: Two packs of one language disagree
- **WHEN** the committed tables of a second pair studying Spanish name other dictionary words than es-fr's
- **THEN** the checks fail, naming both pairs

#### Scenario: An older core
- **WHEN** a core built before this change loads a pack carrying a lexical table
- **THEN** the pack loads, and that core reads its glossed lemmas as its dictionary words

#### Scenario: French's names
- **WHEN** fr-en glosses `paris` and `durand` only by proper nouns' senses, and `marche` by a common noun's beside a department's
- **THEN** `paris` and `durand` are no dictionary words of French, `marche` is, fr-en's pack carries a lexical table, and a French document that capitalises `Paris` in mid-sentence and never writes it in lowercase sets it aside as a name

### Requirement: A studied language's tables are kept once
The tables of a studied language — its forms, frequencies, readings and levels, its pinned tag pool and its dictionary words — SHALL be committed once, in that language's folder (`tables/<studied>/`), and every pair studying that language SHALL be built from that folder together with its own.
A pair's own folder (`tables/<pair>/`) SHALL hold what belongs to the pair alone: its glosses, the
parts of speech of their senses and its expressions, with its notice, manifest, pin and README. A
pair's folder SHALL NOT hold a table of its studied language, and the checks SHALL fail, naming the
pair, the file and the studied language's reference pair, when one does. The dictionary words SHALL
be those *A pack's dictionary words do not depend on its glosses* names — every lemma the reference
pair glosses, or every one but those it glosses by a proper noun's senses alone, as the reference's
reduction writes them —, kept as `lexical.tsv` in the studied language's folder, and the checks
SHALL fail, naming the reference pair and a lemma, when they are neither, or when the folder lacks
its tag pool or its dictionary words. Moving the tables SHALL leave every shipped pack
byte-identical.

#### Scenario: The shipped packs keep their bytes
- **WHEN** the en-fr pack is built from `tables/en/` and `tables/en-fr/`, and the es-fr pack from `tables/es/` and `tables/es-fr/`
- **THEN** each sha256 is the one its `pin.json` recorded before the move, neither pack carries a lexical table, and both invariance baselines pass without being re-blessed

#### Scenario: One copy per studied language
- **WHEN** the committed tables are read
- **THEN** English's `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `tags.tsv` and `lexical.tsv` are in `tables/en/` alone, Spanish's in `tables/es/` alone, and `tables/en-fr/` and `tables/es-fr/` hold only `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md`

#### Scenario: A studied table left in a pair's folder
- **WHEN** a second pair studying Spanish holds its own `grammar.tsv` or `lexical.tsv` in its folder
- **THEN** the checks fail, naming that pair, the file and es-fr

#### Scenario: Dictionary words that are not the reference's
- **WHEN** Spanish's `lexical.tsv` lists a lemma es-fr does not gloss, or leaves out one it glosses
- **THEN** the checks fail, naming es-fr and the lemma

#### Scenario: A pair glossed in another native language copies nothing
- **WHEN** a test pair studying Spanish, glossed in English, is built from `tables/es/` and a folder holding only its glosses, senses, expressions, notice and manifest
- **THEN** it builds, its readings are stored byte for byte as es-fr's, and its dictionary words are es-fr's

#### Scenario: Names left out by halves
- **WHEN** French's `lexical.tsv` leaves out `paris` but lists `lyon`, both glossed by fr-en with a proper noun's senses alone
- **THEN** the checks fail, naming fr-en and `lyon`
