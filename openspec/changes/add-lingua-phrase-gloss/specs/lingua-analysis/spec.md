## ADDED Requirements

### Requirement: Phrase gloss
The core SHALL gloss a short text — a reader's selection — with the same tokeniser, the same lemma cascade and the same status classification as the page analysis, applying neither the per-block language detection nor the minimum token count, which exist to score pages.
For every token, in order, it SHALL return the surface, the dictionary form, the status
class, the pack gloss whatever that class (absent when the pack carries none), and whether
the dictionary form is a function word of the studied language — a closed-class word:
article or other determiner, pronoun, preposition or particle, conjunction, auxiliary or
modal, negation. A hyphenated
compound the lexicon does not list SHALL also carry its parts, each with its dictionary
form, its status class, its pack gloss and its function-word flag. The result SHALL hold
only strings, booleans and integers, SHALL be deterministic at a given `analyzer_version`
and pack, and SHALL fall under the native/WASM parity contract. Adding it SHALL leave the
page analysis output unchanged, byte for byte.

#### Scenario: A selection too short for the page analysis
- **WHEN** the text `gave up` is glossed with a pack that holds `give` and `up`
- **THEN** two tokens are returned, `gave` with the dictionary form `give` and `up` with the dictionary form `up`, each with its pack gloss, although the page analysis reports the same text as not analysable

#### Scenario: A known word keeps its gloss
- **WHEN** the lemma `city` is known and the text `the city` is glossed
- **THEN** the token `city` carries the class "known" and its pack gloss

#### Scenario: Function words are told apart
- **WHEN** the text `in spite of` is glossed
- **THEN** `in` and `of` are flagged as function words and `spite` is not

#### Scenario: A compound the lexicon does not list
- **WHEN** the text `error-prone` is glossed, the lexicon lists `error` and `prone` but not the compound, and the reader knows `error`
- **THEN** one token is returned with the dictionary form `error-prone` and no gloss of its own, and its parts `error` and `prone` each carry their class — "known" for `error` — their function-word flag and their pack gloss

#### Scenario: A compound the lexicon lists
- **WHEN** the text `x-ray` is glossed and the lexicon lists `x-ray`
- **THEN** one token is returned with its own gloss and no parts

#### Scenario: A name outside the lexicon
- **WHEN** the text `Jenkins` is glossed and the lexicon holds no form of it
- **THEN** the token carries the class "proper noun outside the lexicon", as it would on a page

#### Scenario: A text the language detector would reject
- **WHEN** a text that is not in the studied language is glossed
- **THEN** its tokens are returned without error, glossed only where the pack happens to hold the form

#### Scenario: Native and WASM agree
- **WHEN** the same text is glossed by the native build and by the WASM module with the same pack and the same knowledge state
- **THEN** the two results are byte-for-byte identical
