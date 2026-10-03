## ADDED Requirements

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
