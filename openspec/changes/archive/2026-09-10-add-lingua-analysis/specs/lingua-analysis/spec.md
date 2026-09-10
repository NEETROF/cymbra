# lingua-analysis — text analysis pipeline

## ADDED Requirements

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
