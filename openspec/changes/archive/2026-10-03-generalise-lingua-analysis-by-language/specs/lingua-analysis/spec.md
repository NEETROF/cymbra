## ADDED Requirements

### Requirement: Analysis by studied language
The core SHALL run, for each studied language, that language's own tokenisation pre-pass, lemmatisation cascade and function-word tables, and SHALL never run a rule written for one language on text analysed as another. A studied language whose rules are not written yet SHALL get the baseline analysis: segmentation, the pre-pass rules that belong to no language (edge apostrophes, hyphenated compounds, words with digits dropped, single letters counted only when the pack lists them), and the pack's form→lemma lookup. The baseline has no exception table, no morphological rule, no contraction split and no function words. Adding a studied language SHALL leave the output of every existing language byte-for-byte unchanged.

#### Scenario: A Spanish word that looks English keeps its own lemma
- **WHEN** the token `has` is lemmatised as Spanish with a pack that does not list it
- **THEN** the lemma is `has`, not the English `have`

#### Scenario: English contractions are English's
- **WHEN** the text "don't" is tokenised as Spanish
- **THEN** it is a single token; tokenised as English, it is still `do` + `not`

#### Scenario: A language without function-word tables leaves no word out
- **WHEN** a selection is glossed word by word as Spanish
- **THEN** no token is flagged as a function word

#### Scenario: English output does not move
- **WHEN** the English invariance baseline (the engine's output over its fixed corpus, with the real en-fr pack) runs after the core gains a second language
- **THEN** every probe is byte-for-byte the output recorded before, at English analyser version `1.1.0`

### Requirement: An analyser version per studied language
The core SHALL keep one analyser version per studied language, SHALL bump only the version of the language whose output a change can alter, and SHALL report in every page analysis the analyser version of the language the page was analysed as. English's analyser version SHALL remain `1.1.0` through this change, and a language served by the baseline analysis SHALL carry a `0.x` version.

#### Scenario: Each analysis names its own language's version
- **WHEN** one page is analysed as English and another as Spanish
- **THEN** the first reports `analyzer_version` `1.1.0` and the second reports Spanish's own version

#### Scenario: A Spanish rule change leaves English alone
- **WHEN** Spanish's analyser version is bumped
- **THEN** English's analyser version is unchanged and the en-fr pack still loads
