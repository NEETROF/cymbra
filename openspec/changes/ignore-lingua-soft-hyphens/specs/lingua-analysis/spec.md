## ADDED Requirements

### Requirement: A soft hyphen is not part of a word
The core SHALL read every word of a text without the soft hyphens (U+00AD) written inside it, in every studied language, before any rule of tokenisation reads the word: a token's text, its parts and its dictionary form SHALL be those of the word without them, and its span SHALL be the word's span in the text as written, soft hyphens included. The core SHALL decide whether a block is in a studied language, and what it weighs when a document's language is chosen, on the block without its soft hyphens: the detector, the minimum length and every guard — *Catalan and Galician are not read as Spanish*, *Occitan is not read as Spanish*, *Catalan, Occitan and Romanian are not read as French* — read that text. A phrase gloss and a word's grammar SHALL read their text the same way. A text holding soft hyphens SHALL therefore be analysed, detected, chosen and glossed exactly as the same text without them, spans aside, and a text holding none SHALL be analysed byte for byte as before but for the analyser version it reports. Reading a word without its soft hyphens SHALL bump the analyser version of every studied language. In the scenarios, ‧ stands for a soft hyphen.

#### Scenario: A Spanish word read whole
- **WHEN** « vi‧da » is tokenised as Spanish
- **THEN** it is one token, `vida`, whose dictionary form is `vida`, spanning the six bytes of « vi‧da » as written

#### Scenario: A Spanish line is not cut into syllables
- **WHEN** "To‧do el mun‧do sa‧be que ma‧ña‧na ha‧brá mu‧chas co‧sas que ha‧cer." is offered as Spanish
- **THEN** it is Spanish, as the same line without soft hyphens is, its syllables `do`, `ma` and `sas` counting for neither Galician nor Occitan

#### Scenario: A French line is not cut into syllables
- **WHEN** "On a chan‧té en‧semble jus‧qu’au ma‧tin, per‧sonne n’a vou‧lu dor‧mir." is offered as French
- **THEN** it is French, as the same line without soft hyphens is, and its tokens are those of that line — `chanté`, `jusque`, `à`, `le`, `personne` among them — each spanning its word as written

#### Scenario: English words and a contraction
- **WHEN** "The gov‧ern‧ment could‧n't an‧swer the ques‧tion yes‧ter‧day af‧ter‧noon." is analysed as English
- **THEN** its tokens are `The`, `government`, `could`, `not`, `answer`, `the`, `question`, `yesterday`, `afternoon`, with the dictionary forms the same line gives without soft hyphens, `could` and `not` sharing the span of « could‧n't »

#### Scenario: A French elision
- **WHEN** « lors‧qu’il » is tokenised as French
- **THEN** it is `lorsque` and `il`, each with its own span, as « lorsqu’il » is

#### Scenario: A page holding soft hyphens
- **WHEN** a page whose words hold soft hyphens at their syllable breaks is analysed, and offered with several candidate languages
- **THEN** its tokens' texts, dictionary forms, classes and glosses, its counts, its percentage and the language chosen are those of the same page without them, and each token's span indexes the page as written

#### Scenario: A selection and a card
- **WHEN** « ma‧ña‧na » is glossed as a Spanish selection, and the grammar of « can‧tá‧ba‧mos » is asked for the dictionary form `cantar`
- **THEN** the selection is `mañana`, known or not as the reader's status says and glossed, and the card names the readings of « cantábamos »

#### Scenario: Text without soft hyphens does not move
- **WHEN** the English, Spanish, French, es-en and en-es invariance baselines run after this change
- **THEN** every probe is byte for byte the output recorded before but for the analyser version, which English, Spanish and French each bump

## MODIFIED Requirements

### Requirement: An analyser version per studied language
The core SHALL keep one analyser version per studied language, SHALL bump only the version of the language whose output a change can alter, and SHALL report in every page analysis the analyser version of the language the page was analysed as. A rule every studied language shares SHALL bump every language's version when it can alter their output, and a language served by the baseline analysis SHALL carry a `0.x` version.

#### Scenario: Each analysis names its own language's version
- **WHEN** one page is analysed as English and another as Spanish
- **THEN** the first reports English's own analyser version and the second reports Spanish's own version

#### Scenario: A Spanish rule change leaves English alone
- **WHEN** Spanish's analyser version is bumped
- **THEN** English's analyser version is unchanged and the en-fr pack still loads

#### Scenario: A rule every language shares bumps every version
- **WHEN** a change alters how every studied language reads a text, as reading a word without its soft hyphens does
- **THEN** English's, Spanish's and French's analyser versions are each bumped, and every pack is built again at its own language's new version

### Requirement: Analysis by studied language
The core SHALL run, for each studied language, that language's own tokenisation pre-pass, lemmatisation cascade and function-word tables, and SHALL never run a rule written for one language on text analysed as another. A studied language whose rules are not written yet SHALL get the baseline analysis: segmentation, the pre-pass rules that belong to no language (edge apostrophes, hyphenated compounds, words with digits dropped, single letters counted only when the pack lists them, a word read without its soft hyphens), and the pack's form→lemma lookup. The baseline has no exception table, no morphological rule, no contraction split and no function words. Adding a studied language, or writing a language's rules, SHALL leave the output of every other language byte-for-byte unchanged.

#### Scenario: A Spanish word that looks English keeps its own lemma
- **WHEN** the token `has` is lemmatised as Spanish with a pack that lists it as a form of `haber`
- **THEN** the lemma is `haber`, never the English `have`

#### Scenario: English contractions are English's
- **WHEN** the text "don't" is tokenised as Spanish
- **THEN** it is a single token; tokenised as English, it is still `do` + `not`

#### Scenario: A language without function-word tables leaves no word out
- **WHEN** a selection is glossed word by word in a studied language whose function-word tables are not written yet
- **THEN** no token is flagged as a function word

#### Scenario: Each language's closed classes are its own
- **WHEN** a selection is glossed word by word as Spanish
- **THEN** `de` is flagged as a function word and `the` is not; glossed as English, `the` is flagged and `de` is not

#### Scenario: English output does not move
- **WHEN** the English invariance baseline (the engine's output over its fixed corpus, with the real en-fr pack) runs after Spanish's rules are written
- **THEN** every probe is byte-for-byte the output recorded before, at the same English analyser version

### Requirement: A word's grammar, from the pack
The core SHALL answer, for a word as written and the dictionary form its card is keyed by, that word's grammar from the pack: the readings of the form as that dictionary form, the other dictionary forms the form is also a reading of, the pieces the studied language's pre-pass split the written word into, and the dictionary form's gloss with its senses grouped by part of speech.
It SHALL read the written word through the tokeniser and pre-pass the page analysis uses, and SHALL
take its readings from the piece whose dictionary form is the one asked about. It SHALL report
pieces only when the pre-pass split the written word into more than one, and SHALL then name no
other dictionary form: the split has already settled what the piece is. With a pack that carries
no grammar tables it SHALL still answer:
- the gloss, as one group of senses with no part of speech;
- the pieces, which come from the pre-pass and not from the pack;
- no reading and no other dictionary form.

The answer SHALL be deterministic at an equal `analyzer_version` and an equal pack, and identical
byte for byte between the native and the WASM targets over the fixture corpus. Answering it SHALL
leave the page analysis's output unchanged.

#### Scenario: An irregular form
- **WHEN** the core is asked about `went` for the dictionary form `go`
- **THEN** it answers one reading, a verb in the past tense, finite, and no pieces

#### Scenario: A form with two readings
- **WHEN** the core is asked about `walked` for `walk`
- **THEN** it answers two readings: past tense, finite, and past participle

#### Scenario: A contraction
- **WHEN** the core is asked about `doesn't` for `do`
- **THEN** it answers the pieces `does` and `not`, the reading of `does` as a verb in the present tense, third person singular, and no other dictionary form, although `does` alone may be the plural of `doe`

#### Scenario: A form of another dictionary form too
- **WHEN** the core is asked about `leaves` for `leave`, and the pack holds `leaf`
- **THEN** it answers the reading of `leaves` as `leave`, and names `leaf` as another dictionary form, read as a plural noun

#### Scenario: Senses grouped by part of speech
- **WHEN** the core is asked about `can` for `can`
- **THEN** its gloss comes back as a noun group holding « Boîte de conserve » followed by a verb group holding the verb senses

#### Scenario: A pack without grammar tables
- **WHEN** the core is asked about `went` for `go` with a pack that carries no grammar tables
- **THEN** it answers the gloss of `go` as one group with no part of speech, and no reading

#### Scenario: The page analysis does not move
- **WHEN** the fixture corpus is analysed as a page with a pack carrying grammar tables
- **THEN** the output is byte-for-byte the one produced before this change, at the same `analyzer_version`

#### Scenario: Native and WASM agree
- **WHEN** the fixture words are answered by the native build and by the WASM build
- **THEN** the two answers are byte-for-byte identical

### Requirement: French is a studied language served by its own analysis
The core SHALL analyse French as a studied language, tagged `fr` and detected as French, and SHALL serve it with its own analysis — its tokenisation pre-pass (*French tokenisation pre-pass*, *French hyphenated inversions are read as words*), its normalisation (*French text is read in NFC*), its lemmatisation cascade (*French lemmatisation cascade*), its closed classes (*French closed classes*) and its names rule (*A French document's names are set aside*) — beside segmentation and the pre-pass rules that belong to no language, and SHALL no longer serve it with the baseline analysis. French SHALL carry its own analyser version, no longer a `0.x` one, reported by every page analysed as French and compared with a French pack's. Adding French, or writing its rules, SHALL leave English and Spanish output byte for byte unchanged, and every committed pack (en-fr, es-fr, es-en and en-es) loadable and byte-identical.

#### Scenario: An elided word is one token
- **WHEN** « aujourd'hui », « presqu'île » and « quelqu'un » are tokenised as French with a pack that lists none of them
- **THEN** each is one token, and its lemma is itself, lowercased

#### Scenario: Contracted articles are whole
- **WHEN** « du » and « des » are tokenised as French
- **THEN** each is one token, lemmatised as the pack lists it or as itself

#### Scenario: A French block for a learner of French only
- **WHEN** a French paragraph is analysed by a learner of French, and the same paragraph by a learner of English or of Spanish
- **THEN** the first counts it, and the others exclude it, as any block outside their language

#### Scenario: Each language reports its own version
- **WHEN** one page is analysed as English, one as Spanish and one as French
- **THEN** they report English's own version, Spanish's own version and French's own version, which is no `0.x` one

#### Scenario: No rule of another language runs on French
- **WHEN** « don't » is tokenised as French, and `as`, `are`, `ate` and `has` are lemmatised as French with a pack that does not list them
- **THEN** « don't » is one token, and each word comes back as itself, never as an English lemma, and a form with an acute accent is not retried without it

#### Scenario: English and Spanish output do not move
- **WHEN** the English and Spanish invariance baselines run after the core gains French
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and every committed pin is unchanged
