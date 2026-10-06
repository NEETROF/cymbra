## ADDED Requirements

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

## MODIFIED Requirements

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
