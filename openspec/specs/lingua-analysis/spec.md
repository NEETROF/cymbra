# lingua-analysis Specification

## Purpose
TBD - created by archiving change add-lingua-analysis. Update Purpose after archive.
## Requirements
### Requirement: Tokenisation of studied-language text
The core SHALL tokenise a text into candidate words per UAX #29
(`unicode-segmentation`), preceded by a pre-pass for the studied language (for
English: contractions `don't` → `do` + `not`, edge apostrophes stripped).
Single-character tokens SHALL only be counted when they belong to the lexicon of the
studied language ("I", "a" in English).

#### Scenario: Everyday English text
- **WHEN** the text "Teams don't ship code." is analysed as English
- **THEN** the tokens produced are `Teams`, `do`, `not`, `ship`, `code`

### Requirement: Cascading lemmatisation
For each token, the core SHALL produce a lemma through the cascade: exception table
(irregulars) → the pack's forms→lemmas FST lookup → morphological fallback (morphy-style
rules) → regular plural fallback for forms outside the lexicon → the form itself. The
returned lemma SHALL be normalised lowercase.

#### Scenario: Irregular form
- **WHEN** the token `went` is lemmatised
- **THEN** the lemma is `go`

#### Scenario: Out-of-lexicon regular plural
- **WHEN** the token `endeavors` is lemmatised and neither `endeavors` nor `endeavor` is in the frequency lexicon
- **THEN** the lemma is `endeavor` (plural fallback), so that `endeavor` and `endeavors` count as a single distinct word

### Requirement: Per-block language detection
The core SHALL determine, per block of text, whether the content is in the studied
language and SHALL exclude from the analysis every block that is not (including blocks
in the user's native language). A document without enough content in the studied
language SHALL be reported as not analysable.

#### Scenario: Mostly French page for an English learner
- **WHEN** a page whose text is French is analysed (studied language: English, native language: French)
- **THEN** the analysis returns "not analysable" and no token is counted

### Requirement: Known-token percentage
The core SHALL compute a text's percentage of known words as `known tokens / counted
tokens`, counting **occurrences** (tokens), not distinct words. Tokens whose lemma is
ignored SHALL count as known; tokens in the "learning" status SHALL count as not known;
out-of-lexicon proper nouns SHALL be excluded from the count.

#### Scenario: Repeated unknown word
- **WHEN** a text of 10 counted tokens contains 2 occurrences of the same unknown lemma and 8 known tokens
- **THEN** the known percentage is 80%

### Requirement: Determinism at a given analyzer version
The core SHALL expose an `analyzer_version` and SHALL produce, at equal version and
equal pack, identical results whatever the compilation target (native or WASM).

#### Scenario: Native / WASM parity
- **WHEN** the same fixture text is analysed by the native binary and by the WASM module at the same `analyzer_version`
- **THEN** the (token, lemma, classification) lists produced are byte-for-byte identical

### Requirement: Native/WASM parity
The core SHALL be compilable as a WASM module (a `wasm-pack --target web` build) exposing batch-of-blocks analysis (classified tokens, statuses, percentage, glosses), and SHALL produce, at equal `analyzer_version` and equal pack, byte-for-byte identical output between the native target and the WASM target over the fixture corpus. A CI lane SHALL build the WASM module and run the parity tests.

#### Scenario: Parity over the fixture corpus
- **WHEN** the same fixture text is analysed by the native binary and by the WASM module at the same `analyzer_version` and with the same pack
- **THEN** the outputs (token, lemma, classification, percentage) are byte-for-byte identical

#### Scenario: Divergence blocked in CI
- **WHEN** a change to the core makes the WASM output diverge from the native output on a fixture
- **THEN** the CI parity lane fails

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
- **THEN** the output is byte-for-byte the one produced before this change, at `analyzer_version` `1.1.0`

#### Scenario: Native and WASM agree
- **WHEN** the fixture words are answered by the native build and by the WASM build
- **THEN** the two answers are byte-for-byte identical

### Requirement: Analysis by studied language
The core SHALL run, for each studied language, that language's own tokenisation pre-pass, lemmatisation cascade and function-word tables, and SHALL never run a rule written for one language on text analysed as another. A studied language whose rules are not written yet SHALL get the baseline analysis: segmentation, the pre-pass rules that belong to no language (edge apostrophes, hyphenated compounds, words with digits dropped, single letters counted only when the pack lists them), and the pack's form→lemma lookup. The baseline has no exception table, no morphological rule, no contraction split and no function words. Adding a studied language, or writing a language's rules, SHALL leave the output of every other language byte-for-byte unchanged.

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
- **THEN** every probe is byte-for-byte the output recorded before, at English analyser version `1.1.0`

### Requirement: An analyser version per studied language
The core SHALL keep one analyser version per studied language, SHALL bump only the version of the language whose output a change can alter, and SHALL report in every page analysis the analyser version of the language the page was analysed as. English's analyser version SHALL remain `1.1.0` through this change, and a language served by the baseline analysis SHALL carry a `0.x` version.

#### Scenario: Each analysis names its own language's version
- **WHEN** one page is analysed as English and another as Spanish
- **THEN** the first reports `analyzer_version` `1.1.0` and the second reports Spanish's own version

#### Scenario: A Spanish rule change leaves English alone
- **WHEN** Spanish's analyser version is bumped
- **THEN** English's analyser version is unchanged and the en-fr pack still loads

### Requirement: One WASM engine serves every studied language
The WASM engine SHALL hold at most one pack per studied language in a single instance, and SHALL take the studied language on every call whose answer depends on it; a call that names no language SHALL use the language of the first pack loaded. The engine SHALL refuse, with an explicit error, a call naming a language it holds no pack for, and a second pack for a language it already holds. Calls that concern the whole reader (backup, restore, resets, counts, the review session, sync exports and applies) SHALL cover every language the state holds.

#### Scenario: An engine built from the en-fr pack answers as before
- **WHEN** the engine is built from the en-fr pack and called without a language
- **THEN** every answer is byte-for-byte the one recorded by the English invariance baseline

#### Scenario: A second language is served by its own pack
- **WHEN** a Spanish pack is added to an engine built from the en-fr pack, and a page is analysed as Spanish
- **THEN** the analysis uses the Spanish pack and reports Spanish's analyser version, and an analysis that names no language is still English

#### Scenario: A language without a pack is refused
- **WHEN** a call names Spanish on an engine holding only the en-fr pack
- **THEN** it fails with an explicit error and the state is unchanged

#### Scenario: One pack per language
- **WHEN** a second English pack is added to an engine
- **THEN** it is refused and the engine keeps the first

### Requirement: Sync records carry their own language
The WASM engine SHALL export each status, declared level and card with the studied language it belongs to, and SHALL apply incoming records only for the languages it holds a pack for, leaving the others out of its state. A record without a language SHALL be read as English.

#### Scenario: English records export as English
- **WHEN** an engine holding the en-fr pack exports its statuses, declared levels and cards
- **THEN** every record names `en`, as before

#### Scenario: A Spanish record exports as Spanish
- **WHEN** a Spanish status is set on an engine holding the en-fr pack and a Spanish pack
- **THEN** its exported record names `es`

#### Scenario: Records of a language the engine does not study are skipped
- **WHEN** a Spanish status change is applied to an engine holding only the en-fr pack
- **THEN** nothing changes, as before; applied to an engine holding the Spanish pack, the status is recorded under Spanish

### Requirement: A document's language, chosen among the reader's
The core SHALL choose the language of a document among candidate studied languages. Each block long enough to be detected SHALL vote, weighted by its length, for the language detected in it when that language is a candidate. The language with the most weight SHALL win. The document's declared language SHALL break a tie between candidates and SHALL decide when no block votes, and the first candidate SHALL win otherwise. A single candidate SHALL be chosen without detection. The choice SHALL NOT change the page analysis, which is then made in the chosen language.

#### Scenario: A Spanish page for a reader of English and Spanish
- **WHEN** a page whose blocks are Spanish is offered with English and Spanish as candidates
- **THEN** Spanish is chosen

#### Scenario: A page mostly in one language
- **WHEN** a page has long English paragraphs and a short Spanish quotation
- **THEN** English is chosen, and the quotation is excluded from the English analysis

#### Scenario: Nothing to detect
- **WHEN** no block of a page is long enough to be detected, and the page declares Spanish
- **THEN** Spanish is chosen; without a declared candidate language, the first candidate is

#### Scenario: One candidate
- **WHEN** English is the only candidate
- **THEN** English is chosen without detecting anything

### Requirement: Spanish tokenisation pre-pass
The core SHALL normalise Spanish text to NFC before reading it, and SHALL split the contractions `al` and `del` into `a` + `el` and `de` + `el`, the two tokens sharing the source span and the first keeping the token's leading capital. The baseline's rules SHALL otherwise apply unchanged, and nothing SHALL be split inside a hyphenated compound.

#### Scenario: Two contractions
- **WHEN** the text "Vengo del mercado al centro." is tokenised as Spanish
- **THEN** the tokens are `Vengo`, `de`, `el`, `mercado`, `a`, `el`, `centro`

#### Scenario: A capitalised contraction
- **WHEN** the text "Del mar." is tokenised as Spanish
- **THEN** the tokens are `De`, `el`, `mar`, the first two sharing the span of `Del`

#### Scenario: A decomposed accent
- **WHEN** a Spanish token writes `á` as `a` followed by a combining acute accent
- **THEN** it is read, and looked up, as the precomposed `á`

### Requirement: Spanish lemmatisation cascade
For each Spanish token, the core SHALL produce a lowercase lemma through the cascade: the pack's forms → the form without its acute accents → the enclitic rule → a plural fallback for forms outside the lexicon → the form itself. The enclitic rule SHALL run only when the whole form is not in the lexicon, SHALL strip at most two clitic pronouns, and SHALL require a base the lexicon lists that takes enclitics: an infinitive, a gerund, another verb form of two vowel groups or more, or a monosyllabic imperative from a closed list. A written accent on the base SHALL sit on the vowel the base stresses. Spanish's analyser version SHALL be `1.0.0`, and English's SHALL remain `1.1.0`.

#### Scenario: Two clitics on an imperative
- **WHEN** `dámelo` is lemmatised with a pack that lists `da` as a form of `dar` and does not list `dámelo`
- **THEN** the lemma is `dar`

#### Scenario: A listed word is never split
- **WHEN** `sale` is lemmatised with a pack that lists it as a form of `salir`
- **THEN** the lemma is `salir`, never `sa` + `le`

#### Scenario: An accent the stress does not explain
- **WHEN** `comélo` is lemmatised with a pack that lists `come` as a form of `comer`
- **THEN** no enclitic is stripped, since `come` stresses its first syllable

#### Scenario: A monosyllable outside the closed list
- **WHEN** `vale` is lemmatised with a pack that lists `va` but not `vale`
- **THEN** no enclitic is stripped: `va` is not an imperative of the closed list

#### Scenario: An old spelling
- **WHEN** `fué` is lemmatised with a pack that lists `fue` as a form of `ser` but not `fué`
- **THEN** the lemma is `ser`

#### Scenario: An unlisted plural
- **WHEN** `luces` is lemmatised and the lexicon lists neither `luces` nor `luz`
- **THEN** the lemma is `luz`, so the singular and the plural count as one word

### Requirement: Spanish closed classes
The core SHALL flag as function words, in a word-by-word gloss in Spanish, the Spanish determiners, pronouns, prepositions, conjunctions, auxiliaries and modals, and negation, by the lemma the cascade produced, as English's tables do for English.

#### Scenario: A Spanish phrase
- **WHEN** "la casa de mi padre" is glossed word by word as Spanish
- **THEN** `la`, `de` and `mi` are flagged as function words, and `casa` and `padre` are not

### Requirement: Catalan and Galician are not read as Spanish
The core SHALL refuse as Spanish a block the detector reads as Spanish whose Catalan function words, or whose Galician function words, outnumber its Spanish ones. A refused block SHALL be excluded from a Spanish analysis and SHALL NOT vote for Spanish when a document's language is chosen. A tie, or a block with no such function word, SHALL stay Spanish. The guard SHALL leave English's detection unchanged, and Spanish's analyser version SHALL be `1.1.0`. A short Catalan or Galician line with no function word of either kind is not refused, and still reads as Spanish.

#### Scenario: A Catalan paragraph
- **WHEN** "El govern ha aprovat el pressupost amb el suport dels grups, però també amb crítiques." is offered as Spanish
- **THEN** it is not Spanish, and a Spanish analysis excludes it

#### Scenario: A Galician paragraph
- **WHEN** "Non hai unha solución sinxela, pero o concello xa traballa nela." is offered as Spanish
- **THEN** it is not Spanish

#### Scenario: Spanish prose is kept
- **WHEN** "El gobierno aprobó el presupuesto con el apoyo de los grupos, pero también hubo críticas." is offered as Spanish
- **THEN** it is Spanish

#### Scenario: A Catalan page for a reader of English and Spanish
- **WHEN** a page whose blocks are Catalan is offered with English and Spanish as candidates
- **THEN** its blocks give Spanish no vote

#### Scenario: The leak the guard leaves
- **WHEN** a short Catalan caption with no function word of either kind is offered as Spanish
- **THEN** the guard does not refuse it, as this requirement states

