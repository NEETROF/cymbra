# lingua-data-packs Specification

## Purpose
TBD - created by archiving change add-lingua-data-pack. Update Purpose after archive.
## Requirements
### Requirement: Versioned pack container, keyed by language pair
A pack SHALL be a single versioned container, keyed by pair (studied language → native language), holding: metadata (the pair, `pack_version`, the compatible `analyzer_version`, licences), a form→lemma FST, a frequency table (ranks), compressed glosses indexed by lemma, an optional per-lemma CEFR level table (present for pairs that have licence-clean CEFR data, absent otherwise), an optional multi-word expression table, optional grammar tables, and a NOTICE file. The core SHALL refuse a pack whose analyser version is incompatible. A table that only a new interface reads is additive: it SHALL be optional, a core that does not know it SHALL ignore it, and adding it SHALL bump `pack_version` and leave `analyzer_version` alone.

#### Scenario: Loading the EN→FR pack
- **WHEN** the extension starts with the (en → fr) pack embedded
- **THEN** the core exposes lemmatisation, frequency ranks, French glosses, CEFR levels, expressions and word grammar for English

#### Scenario: Pair without CEFR data
- **WHEN** a pack for a pair with no licence-clean CEFR data is loaded
- **THEN** it loads with no level table and the core reports levels as unavailable for that language

#### Scenario: Incompatible pack
- **WHEN** a pack declares an `analyzer_version` incompatible with the core
- **THEN** loading fails with an explicit error and no partial analysis is produced

#### Scenario: A pack whose only new table is additive
- **WHEN** a pack is rebuilt with an expression table and no other change
- **THEN** its `analyzer_version` is the one the core already accepted, and its `pack_version` is new

#### Scenario: A pack whose new tables are grammar tables
- **WHEN** a pack is rebuilt with grammar tables and no other change
- **THEN** its `analyzer_version` is the one the core already accepted, and its `pack_version` is new

### Requirement: Licence hygiene
The build pipeline SHALL accept only sources whose licence permits commercial use (ESDB, WordNet, wordfreq CC BY-SA, kaikki CC BY-SA) and SHALL reject any GPL, AGPL or non-commercial source (documented denylist). Every pack SHALL embed the complete stack of notices, and the user interface SHALL expose an attributions page.
Inflections SHALL come from ESDB, the maintained successor of AGID, completed by the Wiktionary form
links kaikki carries where the inflection is regular. A form a source marks archaic, rarer or
doubtful SHALL NOT be taken as an inflection.

#### Scenario: Notices embedded
- **WHEN** a pack is built
- **THEN** its NOTICE carries the ESDB copyright notice, wordfreq and kaikki, and the extension's "Attributions" page displays them

#### Scenario: A recent plural
- **WHEN** the analysis reads "smartphones" with a pack built from ESDB and kaikki
- **THEN** it is the lemma "smartphone", with that word's status and gloss

#### Scenario: An archaic variant is not an inflection
- **WHEN** the analysis reads "born" or "art"
- **THEN** neither is taken for a form of "bear" or of "be"

### Requirement: Reproducible offline build
Packs SHALL be built by a scripted, reproducible pipeline from reduced tables committed in the repository, and a release SHALL build its pack only from those tables, never from live upstream sources.
The raw data and the built pack SHALL NOT be committed; the reduced tables SHALL be, under their own
licences, with the sha256 of the pack they build. The raw sources the tables were reduced from SHALL
be pinned — at a commit and by sha256 where they are hosted, or kept as a snapshot where they are
not stable — so the tables can be reduced again from the same bytes. Two runs of the pipeline over
the same tables SHALL produce identical packs, and every package of a release — Chromium, Firefox
and Safari — SHALL carry that same pack. A build whose pack does not match the recorded sha256 SHALL
fail, whether in a release or in a pull request's checks.

#### Scenario: Rebuilding the pack
- **WHEN** the pipeline is re-run over the same tables
- **THEN** the resulting pack is byte-for-byte identical to the previous one

#### Scenario: An upstream source is gone
- **WHEN** a raw source of the dictionary no longer answers on release day
- **THEN** the release builds its pack from the committed tables and is not affected

#### Scenario: One dictionary per release
- **WHEN** a version is released to the Chrome Web Store, addons.mozilla.org and the App Store
- **THEN** the three packages carry byte-identical packs

#### Scenario: The reviewer's rebuild
- **WHEN** a reviewer rebuilds the pack from the submitted source archive with no network
- **THEN** they obtain the pack the package carries, byte for byte

#### Scenario: A pull request that breaks the pack
- **WHEN** a pull request changes the builder, the tables or a dependency so that the pack's sha256 or its size budget no longer holds
- **THEN** its checks fail, before any release

### Requirement: Size budget
The (en → fr) pack embedded in the extension SHALL stay under 5 MiB (5 × 1024 × 1024 bytes, the figure the builder enforces), covering every table it carries: the FST, the frequencies, the compressed glosses and the optional level, expression and grammar tables. If it goes over, the build SHALL fail naming what to reduce, and the remedy SHALL take from the optional tables first — the expressions, longest entries then rarest, then the readings of the rarest dictionary forms — then from gloss coverage, never from the FST or the frequencies, which every page analysis depends on. The part of speech of a gloss's senses follows its gloss, and SHALL leave the pack only with it.

#### Scenario: Arbitrating size
- **WHEN** `gloss.zst` pushes a pack carrying no expression or grammar table past the budget
- **THEN** the build fails, telling the operator to reduce the number of glossed lemmas

#### Scenario: The expression table is what pushes the pack over
- **WHEN** the expression table takes a pack past the budget
- **THEN** the build fails and names the expression table as what must be reduced

#### Scenario: The grammar readings are what push the pack over
- **WHEN** a pack carrying no expression table is taken past the budget by its grammar readings
- **THEN** the build fails and names the readings of the rarest dictionary forms as what must be reduced

### Requirement: CEFR level source and enumeration by band
The English pack's CEFR levels SHALL be built from licence-clean sources — CEFR-J Wordlist
v1.6 for A1–B2 and Octanove Vocabulary Profile C1/C2 v1.0 for C1–C2 — joined to lemmas the
same way frequency and glosses are, with one CEFR level per lemma resolved by a defined
collapse rule when a source lists several. The build SHALL carry the required attributions
in NOTICE (the CEFR-J citation string; Octanove CC BY-SA 4.0 attribution, with share-alike
applied to the derived level table), and the licence guard SHALL admit these sources. The
core SHALL let a caller enumerate the lemmas (with glosses) of a given CEFR level, or of a
frequency-rank band when no CEFR data exists.

#### Scenario: Enumerating a level
- **WHEN** a caller requests the lemmas of level B1
- **THEN** it receives the B1 lemmas with their glosses, drawn from the pack

#### Scenario: Attribution carried in NOTICE
- **WHEN** the English pack is built with CEFR levels
- **THEN** NOTICE contains the CEFR-J citation and the Octanove CC BY-SA 4.0 attribution

### Requirement: A dictionary update is a reviewed decision
Live upstream sources SHALL be read only to propose new tables, and new tables SHALL reach a release only through a pull request a person opens and merges.
The proposal SHALL come with a report, against the committed tables, of the lemmas, glosses, levels,
expressions, grammar readings and parts of speech of senses added, removed and changed, and of the
pack's size against its budget. No workflow SHALL open or approve that pull request. A change to the
reduction rules SHALL be applied to the pinned raw sources, so that its diff shows the rule change
and no upstream change; tables whose recorded reduction rules differ from the repository's SHALL
fail the checks.

#### Scenario: Updating the dictionary
- **WHEN** a maintainer runs the update
- **THEN** new tables and a report of what they change are pushed to a branch, and releases keep the committed tables until a person merges a pull request from it

#### Scenario: Reduction rules changed
- **WHEN** the reducer is edited
- **THEN** the checks fail until the tables are reduced again from the pinned raw sources, and that pull request's diff holds only what the edit changes

#### Scenario: The report covers the grammar
- **WHEN** an update changes the readings of a form or the part of speech of a word's senses
- **THEN** the report lists that form or that word among the changes

### Requirement: Upstream breakage is detected before it is needed
The upstream sources SHALL be checked on a monthly schedule without changing anything, and a source that cannot be fetched, a reduction that fails, or a table that collapses SHALL fail that check visibly.

#### Scenario: A source moved
- **WHEN** the monthly check runs and a source no longer answers at its address, or its format changed so that a table loses most of its rows
- **THEN** the check fails and says which source, while releases keep building from the committed tables

#### Scenario: Nothing wrong upstream
- **WHEN** the monthly check runs and every source reduces
- **THEN** it reports how far upstream has drifted from the committed tables, and commits and publishes nothing

### Requirement: A pack says which dictionary it is
A pack's `pack_version` SHALL identify the snapshot of tables it was built from.
Tables reduced again from the same pinned sources under changed reduction rules SHALL be a new
snapshot of tables: their `pack_version` SHALL still name the source snapshot, and SHALL also name
the reduction rules that produced them.

#### Scenario: Two releases, one dictionary
- **WHEN** two releases are built from the same committed tables
- **THEN** their packs report the same `pack_version`

#### Scenario: An updated dictionary
- **WHEN** a release is built after new tables were merged
- **THEN** its pack reports a different `pack_version`

#### Scenario: The same sources reduced under new rules
- **WHEN** the tables are reduced again from the pinned sources of snapshot `2026.09.26` after the reducer changed
- **THEN** the pack reports a `pack_version` that names `2026.09.26` and differs from the one built before the change

### Requirement: Grammar tables
A pack built for a pair whose sources describe their words' grammar SHALL carry grammar tables, drawn from the same licence-clean sources as its forms and glosses and written in one closed vocabulary: the Universal Dependencies part-of-speech tags and a named subset of the Universal Dependencies morphological features.
The tables SHALL hold:
- the **readings** of every inflected form whose dictionary form the pack holds: that dictionary
  form, a part of speech and features. A form spelled like its dictionary form SHALL carry a
  reading too when it is also another form of it, as `put` is its own prétérit;
- for every written form the core's own analysis reads as one dictionary form, the **other
  dictionary forms** the pack holds that the form is believably a reading of. A believable
  dictionary form has the part of speech the relation needs, and the form either inflects it
  regularly or is one the dictionary names as a form of it;
- for every glossed dictionary form, the **part of speech of each sense** of its gloss, with the
  features the word carries in that part of speech whatever its form, such as a noun's gender.

A reading SHALL come only from a relation the reducer already accepts as an inflection, so a form
it rejects as archaic, rarer or doubtful SHALL NOT be read as that inflection.

The vocabulary SHALL name at least tense, mood, person, number, gender, verb form, degree, and a
personal pronoun's case and reflexivity, so that a pack for a Romance language fits it with no
change to the container. The vocabulary is closed on both sides:
- A table SHALL NOT carry a code outside the vocabulary, and the build SHALL fail on one, naming
  it.
- A core SHALL ignore a feature it does not know rather than refuse the pack.

The tables are optional and additive: a pack without them loads, a pack with them loads on a core
that does not read them, and their presence SHALL NOT change `analyzer_version`. The vocabulary
takes only tag names from Universal Dependencies, and no data.

#### Scenario: An irregular verb
- **WHEN** the (en → fr) pack is built
- **THEN** `went` reads as `go`, a verb in the past tense, finite, and `gone` reads as `go`, a verb as past participle

#### Scenario: A regular verb
- **WHEN** the (en → fr) pack is built
- **THEN** `walked` carries two readings of `walk`: past tense, finite, and past participle

#### Scenario: A form spelled like its dictionary form
- **WHEN** the (en → fr) pack is built
- **THEN** `put` carries readings of `put` as past tense and as past participle

#### Scenario: A form of two dictionary forms
- **WHEN** the analysis reads `leaves` as `leave`, and the pack holds `leaf`
- **THEN** the tables name `leaf` as another dictionary form of `leaves`, whose reading is a plural noun

#### Scenario: A relation that is not believable is not named
- **WHEN** the analysis reads `uses` as `use`
- **THEN** the tables do not name `us` as another dictionary form of `uses`

#### Scenario: A rejected variant carries no reading
- **WHEN** the (en → fr) pack is built
- **THEN** `born` carries no reading of `bear`

#### Scenario: A Romance pack fits the vocabulary
- **WHEN** a test pack gives `dijéramos` as `decir`, a verb in the imperfect subjunctive, first person plural, finite; `leche` as a feminine noun; and `me` as a personal pronoun, first person singular, dative
- **THEN** it builds with the container this change defines, and the core reads every one of those features back

#### Scenario: A code outside the vocabulary
- **WHEN** the reduced tables carry a feature or a part of speech the vocabulary does not name
- **THEN** the build fails and names it

#### Scenario: An older core reads a pack that has the tables
- **WHEN** a core built before this change loads a pack carrying the grammar tables
- **THEN** the pack loads and behaves as it did, the tables being ignored

### Requirement: A gloss groups its senses by part of speech
A pack's gloss SHALL list its senses grouped by part of speech, each group in the order its part of speech first appears among the senses picked, and SHALL NOT hold the sense separator inside a sense.
The senses SHALL be picked as the pack picked them before this change, one per entry in turn, up to
eight for a word, but for the acronyms and the Wiktionary's notes the next requirements leave out.
They are then reordered so that the senses of one part of speech are adjacent, and a separator
inside a sense becomes a comma. A word's gloss SHALL hold each sense within 300 characters and all
of them within 800: the card pages it, so its length is no longer what one card can show. A cut SHALL
fall on a word boundary and end with an ellipsis, never mid-word, and a sense that would keep fewer
than 20 characters SHALL be left out rather than cut. An expression's gloss keeps its own limits. The part-of-speech table SHALL account
for exactly the senses the final gloss holds, and the build SHALL fail when the two disagree.

#### Scenario: Senses picked across parts of speech
- **WHEN** the senses picked for a word are a preposition sense, a particle sense and a second preposition sense, in that order
- **THEN** its gloss holds the two preposition senses, then the particle sense, and the table records two preposition senses followed by one particle sense

#### Scenario: A separator inside a sense
- **WHEN** a picked sense reads « Lettre; caractère »
- **THEN** the gloss holds it as « Lettre, caractère », one sense

#### Scenario: A word with many senses
- **WHEN** the dictionary gives a word six senses of a few words each
- **THEN** its gloss holds all six

#### Scenario: A long sense ends on a whole word
- **WHEN** a word's sense is longer than the room its gloss has left for it
- **THEN** the gloss holds it cut after a whole word and ended with « … », never mid-word, or leaves it out when fewer than 20 characters of room remain

#### Scenario: A table that disagrees with its gloss
- **WHEN** the part-of-speech table records a different number of senses than a word's gloss holds
- **THEN** the build fails and names the word

### Requirement: An acronym does not gloss the word it is spelled like
A word's gloss SHALL NOT take senses from an entry whose headword is written all in capitals — an acronym such as `AND`, `WHO` or `US` — when the word has an entry of its own in lower case, and an acronym with no such word SHALL keep its gloss.
The part of speech of a word the reader meets is its own, not that of an acronym the dictionary
lowercases into it: grouped by part of speech, the logic operator `AND` made the card of `and` read
« verbe Faire le ET de ».

#### Scenario: A common word and its acronym
- **WHEN** the dictionary holds `and`, a conjunction, and `AND`, the logic operator, as a noun and a verb
- **THEN** the gloss of `and` holds only the conjunction's sense, under one part of speech

#### Scenario: An acronym with no common word
- **WHEN** the dictionary holds `NATO` and no lower-case `nato`
- **THEN** the gloss of `nato` is the one `NATO` gives

#### Scenario: A capitalised word is no acronym
- **WHEN** the dictionary holds `He`, capitalised, beside `he`
- **THEN** the gloss of `he` holds the senses of both

### Requirement: A gloss holds none of the Wiktionary's notes to its readers
A gloss SHALL NOT hold what the Wiktionary writes for its own readers rather than as a translation: a pointer to another page, such as « → voir there be » or « (→ voir bone marrow) », and the placeholder of an unfinished page, such as « Définition manquante ou à compléter. (Ajouter) ».
The note SHALL be taken out wherever it sits in a sense, and the rest of the sense kept. A sense
that is nothing but notes SHALL be left out, and a word left with no sense SHALL have no gloss. This
holds for the glosses of words and of expressions alike.

#### Scenario: A pointer after a translation
- **WHEN** the dictionary glosses `there` « Y avoir. → voir there be »
- **THEN** the gloss of `there` holds « Y avoir » and no pointer

#### Scenario: A placeholder after a translation
- **WHEN** the dictionary glosses a sense of `pig` « Vivre dans la promiscuité et la saleté. Définition manquante ou à compléter. (Ajouter) »
- **THEN** that sense reads « Vivre dans la promiscuité et la saleté »

#### Scenario: A sense that is only notes
- **WHEN** the dictionary glosses `because` « Parce que » and « Définition manquante ou à compléter. (Ajouter) → voir because of »
- **THEN** the gloss of `because` is « Parce que »

### Requirement: A word that is only a form of another takes that word's gloss
A dictionary form the pack keeps whose every sense in the dictionary only says which word it is a form of SHALL be glossed with the senses of that word in the same part of speech, and SHALL have no gloss when that word has none in that part of speech.
The word list keeps some forms as words of their own — `catacombs`, `bacteria`, `footsteps` — and
the dictionary glosses them only as « Pluriel de catacomb »; without this, their card has no
translation. When the senses name several words, the first one that has senses in that part of
speech gives them. A word of fewer than three letters, such as a letter of the alphabet, gives none.
A form that has a meaning of its own keeps it.

#### Scenario: A plural kept as a word
- **WHEN** the dictionary glosses `catacombs` only as the plural of `catacomb`, and `catacomb` as « Catacombe »
- **THEN** the gloss of `catacombs` is « Catacombe », a noun sense

#### Scenario: A word only in another part of speech
- **WHEN** the dictionary glosses the adjective `hearted` only as a form of `heart`, which is only a noun
- **THEN** `hearted` has no gloss

### Requirement: A pair's reduction rules include the rules it shares
The reduction rules of a pair SHALL be its own reducer together with every module of rules shared between pairs, and the record committed with a pair's tables SHALL name those files and carry one digest over all of them. A change to any of them SHALL fail the checks of every pair whose tables were reduced by the previous rules, until those tables are reduced again from their pinned sources; the version of a pack reduced again SHALL name that digest.

#### Scenario: A shared rule changes
- **WHEN** a pull request edits a rule module shared by the en-fr and es-fr reducers without reducing their tables again
- **THEN** the checks of both pairs fail, naming the rule files that changed

#### Scenario: Another pair's reducer changes
- **WHEN** a pull request edits only the es-fr reducer
- **THEN** the en-fr tables still pass their check

#### Scenario: Moving rules into a shared module
- **WHEN** rules are moved out of a pair's reducer into a shared module without changing what they do
- **THEN** the pair's tables reduced again from the same pinned sources are byte-identical, except for the pack version that names the new rule digest

### Requirement: A source is credited as its licence requires
The attribution notice of a pack SHALL credit each source in the form its licence makes a condition of use, including the author's name where the licence names how the author is to be credited.

#### Scenario: wordfreq
- **WHEN** a pack's frequencies come from wordfreq
- **THEN** its notice credits wordfreq to Robyn Speer, with the CC BY-SA 4.0 licence of its data

### Requirement: A pack names the language it studies
The core SHALL read the language a pack studies from the pack's metadata (`studied`, an ISO 639-1 code), SHALL refuse to load a pack whose studied language it has no analyser for, and SHALL check the pack's `analyzer_version` against the analyser version of that language, never of another. Adding a studied language to the core SHALL leave every existing pack loadable and byte-identical.

#### Scenario: Loading the EN→FR pack names English
- **WHEN** the en-fr pack is loaded
- **THEN** the core reports its studied language as English and accepts its `analyzer_version` `1.1.0`

#### Scenario: A pack for a language the core cannot analyse
- **WHEN** a pack whose metadata names `pt` is loaded by a core with no Portuguese analyser
- **THEN** loading fails with an explicit error naming the language, and no partial analysis is produced

#### Scenario: Versions are compared within a language
- **WHEN** one Spanish pack declares Spanish's analyser version, and another Spanish pack declares English's `1.1.0`
- **THEN** the first loads and the second is refused as built for another analyser generation

#### Scenario: The en-fr pack does not change
- **WHEN** the en-fr pack is built from its committed tables after the core gains a second language
- **THEN** its sha256 is the one recorded in `pin.json` before the change

### Requirement: The shipped pairs are one list
The extension's build SHALL read the language pairs it ships from one list, whose first pair gives the default studied language, and SHALL build each listed pair's pack from that pair's committed tables, checked against that pair's own recorded sha256. It SHALL refuse a pack whose analyser version is not the version of the pack's own studied language. Every package of a release SHALL carry exactly the listed packs, each byte-identical across Chromium, Firefox and Safari. Until Spanish is enabled for readers, a package whose list is anything other than en-fr SHALL be refused.

#### Scenario: Building the listed packs
- **WHEN** a release builds its packs with the list holding en-fr
- **THEN** it builds the en-fr pack from the en-fr tables, its sha256 is the one en-fr's pin records, and the package carries that pack and no other

#### Scenario: A pack built for another analyser generation of its language
- **WHEN** a listed pack's analyser version differs from the core's version for that pack's studied language
- **THEN** the build fails with a message naming the pack, both versions, and the command that rebuilds it

#### Scenario: A pair the build cannot make
- **WHEN** the list names a pair with no committed tables, or no testdata for a test build
- **THEN** the build fails with a message naming the pair and what to add

#### Scenario: A list widened too early
- **WHEN** a package is built with a list holding a pair other than en-fr, before Spanish is enabled for readers
- **THEN** the variant check refuses it

#### Scenario: English unchanged
- **WHEN** the en-fr pack is built from its tables after this change
- **THEN** it is byte-for-byte the pack built before, and only its path inside the package differs

### Requirement: The Spanish pack's forms and frequencies
The es-fr pack's forms SHALL come from the English Wiktionary's Spanish section as kaikki extracts it, from its tagged inflections and its form-of links, without the forms that combine a verb with clitic pronouns, which the analyser's enclitic rule resolves; such a form SHALL never be ranked as a lemma of its own, and a string that is also a plain form of another word SHALL keep that word. Each form SHALL map to one lemma, chosen by a reviewed override list, then by its counts in UD Spanish-GSD, then by the form's own entry, then by the lemma's frequency. The pack SHALL keep the 60,000 commonest Spanish lemmas by wordfreq and their forms attested in wordfreq. Its tables SHALL be committed and pinned like every pair's, and SHALL build in the extension's checks.

#### Scenario: A homograph by evidence
- **WHEN** the tables are reduced and `fue` is a form of both *ser* and *ir*
- **THEN** `fue` maps to *ser*, the lemma GSD counts it under

#### Scenario: A combined form is left to the analyser
- **WHEN** the tables are reduced
- **THEN** `dámelo` is not in the forms table, and the analyser still lemmatises it as *dar* through its enclitic rule

#### Scenario: A plural that is also a combined form
- **WHEN** the tables are reduced and `principales` is both *principar* with the pronoun `les` and the plural of *principal*
- **THEN** `principales` maps to *principal*, and is not a lemma of its own

#### Scenario: The pack builds where en-fr's does
- **WHEN** a pull request runs the extension's checks
- **THEN** the es-fr pack is built from the committed tables and checked against its pinned sha256

### Requirement: Spanish forms are measured on a held-out treebank
The pipeline SHALL measure the es-fr pack with the real analyser on UD Spanish-PUD, a treebank the reduction never reads. The measurement SHALL count punctuation, numbers, symbols and proper nouns out, and SHALL fail when fewer than 98.5 % of the tokens resolve in the lexicon, fewer than 93.5 % of the content words (nouns, verbs, adjectives, adverbs) take PUD's lemma, or fewer than 97 % of the auxiliaries do.

#### Scenario: The committed tables pass the gates
- **WHEN** the harness runs over UD Spanish-PUD with the pack built from the committed tables
- **THEN** it reports at least 98.5 % of tokens resolved, 93.5 % of content lemmas and 97 % of auxiliaries, and succeeds

### Requirement: The Spanish pack's word grammar
The es-fr pack SHALL carry the grammar of the forms its forms table holds, read from kaikki's tags as Universal Dependencies tags: a verb form's mood, tense, person and number, or its infinitive, gerund or agreed participle; a noun's gender on its own form and on its plural; an adjective's, determiner's or pronoun's agreement. A reading of another dictionary form the pack keeps SHALL be marked so that the card names it. A form that combines a verb with clitic pronouns SHALL carry no reading.

#### Scenario: A verb form says what it is
- **WHEN** the card asks the grammar of `hablábamos` as *hablar*
- **THEN** it answers the indicative imperfect, first person plural (`VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin`)

#### Scenario: A noun says its gender
- **WHEN** the card asks the grammar of `casas` as *casa*
- **THEN** it answers a feminine plural noun (`NOUN|Gender=Fem|Number=Plur`)

#### Scenario: A homograph names its other dictionary form
- **WHEN** the card asks the grammar of `vino` as the noun *vino*
- **THEN** it also names *venir*, whose preterite third person singular `vino` is

