# Spec Delta

## ADDED Requirements

### Requirement: French is glossed in Italian from the Italian Wiktionary's French section and the two Wiktionaries' translation tables
fr-it's glosses, sense runs and expressions SHALL be reduced from the Italian Wiktionary's French section with the Italian edition's rules, then, for what that section leaves out, from the Italian translations the French Wiktionary's French entries list, in the table's order, then from the Italian entries of the Italian Wiktionary whose translations list the French word, the commonest Italian word first — words people wrote in Italian, at most three per part of speech from each table, with no pivot and no machine translation —, and fr-it SHALL read French's committed studied tables as a pair that is not its studied language's reference does, loading no other pair's reducer. In both tables a letter SHALL gloss no word, and an Italian word that several parts of speech of one Italian entry list for a French word SHALL be listed once, under the first of them French's readings name, or else the first listed. A typographic apostrophe in a French headword, or in a French word of either table, SHALL be read as `'`, and every table word SHALL be written in the Italian edition's typography. A definition whose every sense is the French headword itself, as written up to case, SHALL give way to the French Wiktionary's Italian translations of the word when they hold no word spelled as the headword. An adjective section the edition tags possessive or demonstrative, or whose every sense is a form of such a word, SHALL be read as a determiner. A gloss a lemma only borrows through a form or pointer sense of another word SHALL give way to the direct table's gloss of the lemma, else the inverted table's, and SHALL be kept when no table glosses it. A pack built from fr-it's tables carries French's dictionary words, fr-en's glossed lemmas: a lemma fr-it alone glosses is none. The share of the glossed lemmas a translation table glosses, among the 10,000 commonest, SHALL be measured and shown with the tables, and stored in no pack.

#### Scenario: A definition first
- **WHEN** the Italian Wiktionary's French section defines `maison` « casa (edificio) » and the French Wiktionary's table also gives it « casa »
- **THEN** `maison` is glossed « casa (edificio) », the section's definition

#### Scenario: A word only a table glosses
- **WHEN** no sense of the section glosses a French lemma and the French Wiktionary's Italian table translates it
- **THEN** it is glossed by the table's Italian words, at most three per part of speech

#### Scenario: A borrowed gloss gives way
- **WHEN** the section's only entry spelled `fond` is a form of `fondre`, whose gloss « fondere » it would borrow, and the French Wiktionary's table translates `fond` « fondo, sfondo »
- **THEN** the gloss of `fond` is the table's, never « fondere »

#### Scenario: A borrowed gloss kept
- **WHEN** a French lemma is glossed only through a pointer sense of another word, and no table glosses it
- **THEN** it keeps the borrowed gloss

#### Scenario: An apostrophe
- **WHEN** the French Wiktionary's table lists `chef-d’œuvre` « capolavoro », with a typographic apostrophe
- **THEN** the lemma `chef-d'œuvre` is glossed « capolavoro »

#### Scenario: A letter
- **WHEN** either table lists Italian words for the French letters `h` or `x`
- **THEN** neither is glossed by them

#### Scenario: Listed once
- **WHEN** the Italian Wiktionary's `mio`, read backwards, lists `mon` under both its adjective and its pronoun
- **THEN** « mio » is listed once in the gloss of `mon`

#### Scenario: The studied word is no definition
- **WHEN** the section defines `igloo` only as « igloo », and the French Wiktionary's table translates it « iglù, iglò »
- **THEN** the gloss of `igloo` is « iglù, iglò »

#### Scenario: A cognate keeps its definition
- **WHEN** the section defines a French word only by the same spelling, and the French Wiktionary's table lists that spelling among its Italian words
- **THEN** the definition is kept

#### Scenario: A determiner
- **WHEN** the section's `ce` is an adjective tagged demonstrative
- **THEN** fr-it reads it as a determiner, as fr-es does

#### Scenario: The Italian edition's typography
- **WHEN** a word of the direct table is written with three dots « ... »
- **THEN** fr-it's gloss writes « … »

#### Scenario: French's dictionary words
- **WHEN** fr-it glosses a lemma fr-en does not
- **THEN** a pack built from fr-it's tables does not count it among French's dictionary words

#### Scenario: The share
- **WHEN** fr-it's tables are written
- **THEN** the share of the glossed lemmas among the 10,000 commonest that a translation table glosses is shown with them, and stored in no pack

#### Scenario: Nothing else moves
- **WHEN** fr-it's tables are first committed, by its own reducer, with no shared or edition module edited
- **THEN** every other pair's tables, pins, packs and goldens are unchanged

### Requirement: fr-it is committed beside French's studied tables, held to a coverage floor, and not shipped
fr-it's native side SHALL be reduced from its own pin — the Italian Wiktionary's French section and French translations, and the French Wiktionary's Italian translations — and committed in `tables/fr-it/`, reading `tables/fr/` as committed and writing none of it; its pin SHALL record the studied tables it read, and its notice and manifest SHALL credit both sides. Its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, and SHALL stay under the size budget. Its coverage of the 5,000, 10,000 and 20,000 commonest French lemmas SHALL be measured at each reduction and SHALL NOT fall under `FLOORS["fr-it"]`, fixed before the first committed measurement at the measured figures on the committed ranks less two points, settled by the owner, and never lowered after it: a first measurement under the floor SHALL leave the tables uncommitted, and a later reduction under it SHALL fail the checks. A fr-it invariance baseline SHALL run over the French corpus with the committed fr-it pack wherever the French ones run, and the extension's tests SHALL render fr-it's card in Italian into a committed snapshot re-blessed with its golden. No package SHALL list fr-it before French is enabled for Italian speakers.

#### Scenario: The floor holds
- **WHEN** fr-it is reduced and its coverage is at or above `FLOORS["fr-it"]`
- **THEN** the tables are committed and the coverage, by source, is shown in their README

#### Scenario: Under the floor
- **WHEN** a pull request re-reduces fr-it and its committed tables measure under `FLOORS["fr-it"]`
- **THEN** the checks fail — the gloss-coverage floor step of the extension check's `reduce` job —, naming the pair, the top and the figure

#### Scenario: Not lowered after measuring
- **WHEN** a pull request lowers `FLOORS["fr-it"]` after the first committed measurement
- **THEN** the test holding the floor to the figures the requirement states fails

#### Scenario: The credits
- **WHEN** fr-it's tables are committed
- **THEN** its notice names the Italian Wiktionary, the French Wiktionary and every source of French's studied tables

#### Scenario: One studied side, three natives
- **WHEN** the French corpus is read with the fr-en, fr-es and fr-it packs
- **THEN** every token, lemma, reading and level is the same, and only the glosses differ

#### Scenario: Not shipped
- **WHEN** fr-it's tables are committed and the list of shipped pairs does not name fr-it
- **THEN** no package carries fr-it, and no reader's package or engine moves

## MODIFIED Requirements

### Requirement: A word that is only a form of another takes that word's gloss
A dictionary form the pack keeps whose every sense in the dictionary only says which word it is a form of SHALL be glossed with the senses of that word in the same part of speech, and SHALL have no gloss when that word has none in that part of speech. A pair whose reduction lets a translation table's gloss of the form itself win (fr-it) SHALL gloss such a form, and a form the Italian edition reads as a pointer, with the direct table's words for it, else the inverted table's, keeping the borrowed senses only when no table glosses it.
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

#### Scenario: A form a table glosses, in fr-it
- **WHEN** the Italian Wiktionary's only entry spelled `fond` is a form of `fondre`, glossed « fondere », and the French Wiktionary's table translates `fond` « fondo, sfondo »
- **THEN** fr-it's gloss of `fond` is the table's, and every other pair glosses a form as before

### Requirement: English is glossed in Italian from the Italian Wiktionary's English section and the two Wiktionaries' translation tables
en-it's glosses SHALL be reduced from the Italian Wiktionary's English section with the Italian edition's rules, then, for what that section leaves out, from the English Wiktionary's Italian translation tables, then from the Italian Wiktionary's English translations read backwards — words people wrote in Italian, at most three per part of speech from each table, with no pivot and no machine translation —, reading the committed English studied tables as a reader pair does and loading no other pair's reducer. No translation table SHALL gloss a letter: a letter's entry, a one-letter word's noun entry and an entry whose every translation is its own letter are left out of both tables, and, read backwards, every one-letter English word. In the direct table a word labelled disused SHALL NOT be listed, and a translator's note inside a word — a sense number, a loanword's respelling, a label holding no word wordfreq's Italian list rates at 1.0 Zipf or more, a usage note opening on « with » — SHALL be taken out and the word kept. Read backwards, an Italian word SHALL be listed once per English word, under the first of its parts of speech English's readings name, or else the first listed, the words ordered by Italian frequency. Every table word SHALL be written in the Italian edition's typography, for words and expressions alike. Its expressions SHALL be keyed as en-es's are. The share of glossed lemmas among the 10,000 commonest whose gloss came from a translation table SHALL be measured and shown. The Italian edition's rules SHALL be read by en-it and fr-it and by no other committed pair.

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
- **THEN** en-it's and fr-it's rule digests move, and no other committed pair's does

#### Scenario: Nothing else moves
- **WHEN** en-it's tables are committed by its own reducer, with no shared or edition module edited
- **THEN** every other pair's tables, pins, packs and goldens are unchanged, but for the line of fr-it's golden naming the en-it pack beside it
