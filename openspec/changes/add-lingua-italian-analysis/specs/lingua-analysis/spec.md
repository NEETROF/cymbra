# Spec Delta

## ADDED Requirements

### Requirement: Italian is a studied language served by its own analysis
The core SHALL analyse Italian as a studied language, tagged `it` and detected as Italian, and SHALL serve it with its own analysis — its tokenisation pre-pass (*Italian tokenisation pre-pass*), its normalisation (*Italian text is read in NFC*), its lemmatisation cascade with its enclitic rule (*Italian lemmatisation cascade*), its closed classes (*Italian closed classes*) and its names rule (*An Italian document's names are set aside*) — and with nothing of English's, Spanish's or French's. Italian SHALL carry an analyser version of its own, which a pack studying Italian is checked against and no other language's pack is, and SHALL leave the baseline's `0.x` versions once its own analysis serves it. A rule written for English, Spanish or French SHALL NOT run on Italian, and a rule written for Italian SHALL NOT run on them.

#### Scenario: An elided word is pieces
- **WHEN** « l'uomo » and « c'è » are tokenised as Italian with a pack that lists `lo`, `uomo`, `ci` and `è`
- **THEN** the tokens are `lo` and `uomo`, `ci` and `è`, each lemmatised by the pack

#### Scenario: The di family is whole
- **WHEN** « del » and « nel » are tokenised as Italian
- **THEN** « del » is one token, lemmatised as the pack lists it, and « nel » is `in` and `il` sharing the word's span

#### Scenario: An Italian block for a learner of Italian only
- **WHEN** an Italian paragraph is analysed by a learner of Italian, and the same paragraph by a learner of English, of Spanish or of French
- **THEN** the first counts it, and the others exclude it, as any block outside their language

#### Scenario: Each language reports its own version
- **WHEN** one page is analysed as English, one as Spanish, one as French and one as Italian
- **THEN** each reports its own language's version, and Italian's is at or above `1.0.0`

#### Scenario: No rule of another language runs on Italian
- **WHEN** « don't » and « l'homme » are tokenised as Italian, and `as`, `has`, `dió` and `été` are lemmatised as Italian with a pack that does not list them
- **THEN** « don't » is one token, « l'homme » is `lo` and `homme` by Italian's own rule, and each word comes back as itself, never as another language's lemma, a form with an acute accent not retried without it

#### Scenario: English, Spanish and French output do not move
- **WHEN** the six invariance baselines run after Italian gains its own analysis
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and every committed pin is unchanged

### Requirement: Italian text is read in NFC
The core SHALL read Italian text in Unicode normalisation form C: its tokenisation pre-pass SHALL compose every word before comparing it with anything — the elided forms, the articulated prepositions, the truncations, a hyphenated run the pack lists whole — and every Italian token's text SHALL be composed, while each token's span SHALL still point into the source text, combining marks included. English text SHALL still be read as it came.

#### Scenario: A decomposed accent
- **WHEN** « città » is written with its accent as a combining mark and analysed as Italian with a pack that lists `città`
- **THEN** the token's text is the composed word, it resolves to the pack's lemma, and its span covers the decomposed source

#### Scenario: A decomposed elision
- **WHEN** « l'età » is written with its accent as a combining mark and tokenised as Italian
- **THEN** the tokens are `lo` and the composed `età`

#### Scenario: A decomposed articulated preposition
- **WHEN** « dall'età » is tokenised as Italian with the accent of « età » as a combining mark
- **THEN** the pieces are `da`, `lo` and the composed `età`

#### Scenario: English keeps its text as it came
- **WHEN** an English word is written with a combining mark and analysed as English
- **THEN** its token's text is the decomposed source, as before

### Requirement: Italian lemmatisation cascade
For each Italian token, the core SHALL produce a lowercase lemma through the cascade: the pack's forms → the enclitic rule → an unlisted plural whose singular the pack does not list either, read as that singular → the form itself. The enclitic rule SHALL run only when the whole form is not in the lexicon, SHALL strip one clitic pronoun (`mi`, `ti`, `si`, `ci`, `vi`, `lo`, `la`, `li`, `le`, `ne`, `gli`) or one combined clitic (`me`, `te`, `se`, `ce`, `ve` or `glie` followed by `lo`, `la`, `li`, `le` or `ne`), and SHALL accept the rest only as a base the lexicon lists shaped to take clitics: an infinitive with its ending restored (a rest in `-r` tried with `-e` restored, then with `-re`), a gerund in `-ando` or `-endo`, an imperative the lexicon files under a verb, one of the five monosyllabic imperatives `da`, `fa`, `va`, `di`, `sta` with the clitic's doubled consonant undone and read as `dare`, `fare`, `andare`, `dire`, `stare`, or `ecco`; the lemma SHALL be the base's. The unlisted-plural step SHALL read `-i` as `-o` (`-chi` as `-co`, `-ghi` as `-go`) and `-e` as `-a` (`-che` as `-ca`, `-ghe` as `-ga`), for a form written in lowercase of four letters or more the pack does not hold, whose singular it does not hold either, and SHALL leave alone an elided or hyphenated word and a form ending in `-ai`, `-ei` or `-ii`. The order SHALL be the determinism contract, and a change to it SHALL bump Italian's analyser version. No step SHALL read a form as a word the pack lists under another lemma: the tables decide (M8).

#### Scenario: A clitic on an infinitive
- **WHEN** « parlarmi », « dirlo » and « porlo » are lemmatised as Italian with a pack listing `parlare`, `dire` and `porre` but not the forms
- **THEN** the lemmas are `parlare`, `dire` and `porre`

#### Scenario: A combined clitic
- **WHEN** « dirmelo », « dargliela » and « andarsene » are lemmatised as Italian with a pack listing `dire`, `dare` and `andare`
- **THEN** the lemmas are `dire`, `dare` and `andare`

#### Scenario: A clitic on a gerund and on an imperative
- **WHEN** « parlandogli », « scrivimi » and « prendetelo » are lemmatised as Italian with a pack filing `parlando`, `scrivi` and `prendete` under `parlare`, `scrivere` and `prendere`
- **THEN** the lemmas are `parlare`, `scrivere` and `prendere`

#### Scenario: A monosyllabic imperative
- **WHEN** « dammi », « fallo », « vattene », « dimmi » and « stacci » are lemmatised as Italian
- **THEN** the lemmas are `dare`, `fare`, `andare`, `dire` and `stare`, the doubled consonant undone; « digli » and « fagli » too, without doubling

#### Scenario: Ecco
- **WHEN** « eccolo » and « eccomi » are lemmatised as Italian with a pack listing `ecco`
- **THEN** the lemma is `ecco`

#### Scenario: A listed word is never split
- **WHEN** « cinema », « sistema », « tema » and « vela » are lemmatised as Italian with a pack that lists them
- **THEN** each is the pack's lemma, and the enclitic rule does not run

#### Scenario: A base the pack does not list
- **WHEN** « zorbarmi » is lemmatised as Italian with a pack that lists no `zorbare`
- **THEN** the lemma is `zorbarmi`

#### Scenario: An unlisted plural
- **WHEN** « tavoli », « amiche » and « botteghe » are lemmatised as Italian with a pack that lists neither them nor `tavolo`, `amica`, `bottega`
- **THEN** the lemmas are `tavolo`, `amica` and `bottega`

#### Scenario: The tables decide
- **WHEN** « porta » and « porte » are lemmatised as Italian with a pack filing both under `porta`
- **THEN** each is `porta`, and no rule runs

#### Scenario: A plural whose singular is another word's form
- **WHEN** « vite » is lemmatised as Italian with a pack that lists `vita` but not `vite`
- **THEN** the lemma is `vite`: the singular the rule would reach is a word the pack lists under another lemma

#### Scenario: Italian leaves the baseline's version
- **WHEN** a page is analysed as Italian after this change
- **THEN** it reports Italian's own version, at or above `1.0.0`, and a pack built at `0.2.0` is refused as built for another analyser generation

### Requirement: Italian closed classes
The core SHALL flag as function words, in a word-by-word gloss in Italian, the Italian articles and other determiners, pronouns, prepositions, conjunctions, the quantifiers `molto`, `tanto`, `poco`, `tutto` and `troppo` among the determiners, the auxiliaries and modals `essere`, `avere`, `potere` and `dovere`, and the negation `non`, `no` and `né`, by the lemma the cascade produced, as English's, Spanish's and French's tables do for their languages. The pieces of a split articulated preposition and of an elision SHALL be flagged by their own lemmas. `mai`, `più`, `neanche`, `nemmeno`, `neppure`, `volere`, `secondo`, `lungo` and `stare` SHALL NOT be flagged.

#### Scenario: An Italian phrase
- **WHEN** « Non ho visto il cane di Marco » is glossed word by word as Italian with a pack filing `ho` under `avere`
- **THEN** `non`, `avere` (of « ho ») and `di` and `il` are function words, and `visto`, `cane` and `Marco` are not

#### Scenario: The negation
- **WHEN** « non lo so » and « né oggi né mai » are glossed word by word as Italian
- **THEN** `non`, `lo` and `né` are function words, and `mai` is not

#### Scenario: Words that look closed but carry meaning
- **WHEN** « più », « mai », « sto parlando », « stare », « voglio », « il secondo giorno » and « molto tempo » are glossed word by word as Italian with a pack filing `sto` under `stare` and `voglio` under `volere`
- **THEN** none of `più`, `mai`, `stare`, `volere` and `secondo` is a function word, and `molto` in « molto tempo » is one

#### Scenario: The pieces of a contraction and of an elision
- **WHEN** « al mare » and « l'uomo » are glossed word by word as Italian
- **THEN** `a`, `il` and `lo` are function words, and `mare` and `uomo` are not

#### Scenario: Each language keeps its own tables
- **WHEN** `the` is glossed as Italian, and `il` as English, Spanish or French
- **THEN** neither is a function word

### Requirement: An Italian document's names are set aside
The core SHALL set aside, in an Italian document, every occurrence of a form that the document never writes in lowercase, that it capitalises at least once in mid-sentence — after a letter, a digit, a comma or a semicolon, or right after an elided piece —, and whose dictionary form is not one of the pack's dictionary words, as Spanish's and French's rules do; a hyphenated run the pack does not list SHALL be read as one form for this rule. A form set aside SHALL be excluded from the counts, the statuses and the cards, as any proper noun is.

#### Scenario: A city
- **WHEN** « Sono andato a Roma. Roma è grande. » is analysed as Italian with a pack that does not list `roma`
- **THEN** « Roma » is set aside in both sentences

#### Scenario: After an elided piece
- **WHEN** « L'Italia è lunga » is analysed as Italian with a pack that does not list `italia`
- **THEN** « Italia » is set aside, the elided piece counting as mid-sentence

#### Scenario: A hyphenated name
- **WHEN** « con Anna-Maria Rossi » is analysed as Italian with a pack that lists none of its pieces
- **THEN** « Anna-Maria » is set aside as one form

#### Scenario: A dictionary word
- **WHEN** « a Firenze, Marco legge » is analysed as Italian with a pack that lists `marco` as a dictionary word
- **THEN** « Firenze » is set aside and « Marco » is not

#### Scenario: Capitals at the head of sentences only
- **WHEN** « Ecco. Ecco la casa. » is analysed as Italian with a pack that does not list `ecco`
- **THEN** « Ecco » is not set aside, having appeared at the head of sentences only

#### Scenario: The same form as a word
- **WHEN** « il Sale e il sale » is analysed as Italian
- **THEN** « Sale » is not set aside, its lowercase form appearing in the document

#### Scenario: Spanish's and French's rules do not change
- **WHEN** the Spanish and French baselines run after this change
- **THEN** every names probe is byte for byte the output recorded before

## MODIFIED Requirements

### Requirement: An Italian invariance baseline runs beside the English, Spanish and French ones
The core's tests SHALL hold an Italian invariance baseline: the engine's output over a fixed thirteen-page raw-text Italian corpus — pages authored around one phenomenon each (news, fiction, homographs, elisions, the articulated prepositions in both readings, enclitic pronouns, names and titles, informal Italian with the apostrophe for an accent, a recipe, technical prose with one block in NFD, a mixed page with English, Spanish, French, Catalan, Galician and Latin blocks, verb forms) and two public-domain excerpts, set in real Italian punctuation — through the same probes the other baselines use, over a fixture pack studying Italian glossed in French until the Italian tables are committed, with the engine started on the real en-fr pack. The baseline SHALL run wherever the others run; a change that moves its golden SHALL re-bless it and say why, and a change of an Italian rule SHALL bump Italian's analyser version.

#### Scenario: The corpus
- **WHEN** the baseline reads its corpus
- **THEN** it holds the thirteen pages in order, each with at least one block, the reader's pages are among them, and the technical page's NFD block is still decomposed

#### Scenario: Nothing of Italian changed
- **WHEN** the baseline runs on a pull request that changes no Italian rule, no fixture table and no beside pack
- **THEN** every probe is byte for byte the golden

#### Scenario: What the baseline shows today
- **WHEN** the baseline analyses its `elisioni`, `preposizioni`, `clitici`, `tecnico` and `misto` pages
- **THEN** « l'uomo » is `lo` and `uomo`, « del » is whole, « al » is `a` and `il`, « po' » is `poco`, « dirmelo » resolves to `dire`, the NFD block's words resolve to the pack's, `non` and `di` are function words, the Catalan and Galician blocks are analysed as Italian, and the analysis reports Italian's own version

#### Scenario: An Italian rule lands
- **WHEN** a later change gives Italian a tokenisation, lemmatisation or detection rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps Italian's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the Italian tables and the it-fr pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are Italian
- **THEN** the backup has schema version 4

## REMOVED Requirements

### Requirement: Italian is a studied language served by the baseline analysis
**Reason**: Italian is served by its own analysis from this change: its name, its scenario *No Italian word is a function word yet*, its scenario *An enclitic form is one token* and its `0.2.0` would read false.
**Migration**: *Italian is a studied language served by its own analysis* carries its scenarios that still hold (an elided word is pieces, the di family is whole, an Italian block for a learner of Italian only, each language reports its own version, no rule of another language runs on Italian, the three other languages' output does not move) with the wording of Italian's own rules.
