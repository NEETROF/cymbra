## ADDED Requirements

### Requirement: English is glossed in Italian from the Italian Wiktionary's English section and the two Wiktionaries' translation tables
en-it's glosses SHALL be reduced from the Italian Wiktionary's English section with the Italian edition's rules, then, for what that section leaves out, from the English Wiktionary's Italian translation tables, then from the Italian Wiktionary's English translations read backwards — words people wrote in Italian, at most three per part of speech from each table, with no pivot and no machine translation —, reading the committed English studied tables as a reader pair does and loading no other pair's reducer. No translation table SHALL gloss a letter: a letter's entry, a one-letter word's noun entry and an entry whose every translation is its own letter are left out of both tables, and, read backwards, every one-letter English word. In the direct table a word labelled disused SHALL NOT be listed, and a translator's note inside a word — a sense number, a loanword's respelling, a label holding no word wordfreq's Italian list rates at 1.0 Zipf or more, a usage note opening on « with » — SHALL be taken out and the word kept. Read backwards, an Italian word SHALL be listed once per English word, under the first of its parts of speech English's readings name, or else the first listed, the words ordered by Italian frequency. Every table word SHALL be written in the Italian edition's typography, for words and expressions alike. Its expressions SHALL be keyed as en-es's are. The share of glossed lemmas among the 10,000 commonest whose gloss came from a translation table SHALL be measured and shown. The Italian edition's rules SHALL be read by en-it and by no other committed pair.

#### Scenario: A definition first
- **WHEN** the Italian Wiktionary's English section defines `summer` and a translation table also gives it Italian words
- **THEN** `summer` is glossed by the section's definition

#### Scenario: A word only a table glosses
- **WHEN** no sense of the section glosses an English lemma and the English Wiktionary's Italian table translates it
- **THEN** it is glossed by the table's Italian words, at most three per part of speech

#### Scenario: Listed once
- **WHEN** the Italian Wiktionary's tables, read backwards, give `left-hand` « sinistro » under three parts of speech
- **THEN** « sinistro » is listed once

#### Scenario: A translator's note
- **WHEN** the English Wiktionary's Italian table gives `enrichment` « arricchimento (1,2) »
- **THEN** en-it's gloss reads « arricchimento »

#### Scenario: A rule of the Italian edition
- **WHEN** a rule of the Italian edition changes
- **THEN** en-it's rule digest moves, and no other committed pair's does

#### Scenario: Nothing else moves
- **WHEN** en-it's tables are committed
- **THEN** every other pair's tables, pins, packs and goldens are unchanged

### Requirement: en-it is committed beside English's studied tables, held to a coverage floor, and not shipped
en-it's native side SHALL be reduced from its own pin — the Italian Wiktionary's English section and English translations, and the English Wiktionary's Italian translations — and committed in `tables/en-it/`, reading `tables/en/` as committed and writing none of it; its pin SHALL record the studied tables it read, and its notice and manifest SHALL credit both sides. Its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, and SHALL stay under the size budget. Its coverage of the 5,000, 10,000 and 20,000 commonest English lemmas SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["en-it"]`, fixed before the first committed measurement at the measured figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail the checks. An en-it invariance baseline SHALL run over the English corpus with the committed en-it pack wherever the English ones run, and the extension's tests SHALL render en-it's card in Italian into a committed snapshot re-blessed with its golden. No package SHALL list en-it before Italian is an interface language.

#### Scenario: The floor holds
- **WHEN** en-it is reduced and its coverage is at or above `FLOORS["en-it"]`
- **THEN** the tables are committed and the coverage, by source, is shown in their README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces en-it and its committed tables measure under `FLOORS["en-it"]`
- **THEN** the checks fail — the gloss-coverage floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["en-it"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The credits
- **WHEN** en-it's tables are committed
- **THEN** its notice names the Italian Wiktionary, the English Wiktionary and every source of English's studied tables

#### Scenario: One studied side, three natives
- **WHEN** the English corpus is read with the en-fr, en-es and en-it packs
- **THEN** every token, lemma, reading and level is the same, and only the glosses differ

#### Scenario: Not shipped
- **WHEN** en-it's tables are committed and the list of shipped pairs does not name en-it
- **THEN** no package carries en-it, and no reader's package or engine moves
