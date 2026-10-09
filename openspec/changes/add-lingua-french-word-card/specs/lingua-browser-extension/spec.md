## ADDED Requirements

### Requirement: A French card names its forms in the interface language's grammar
A card of a French word SHALL name its forms in the interface language's grammar — in English, in the English Wiktionary's French form-of wording, the passé simple named the past historic; in Spanish, in RAE/ASALE terms — with their person and number, its finite tenses in the order indicative (present, imperfect, past historic, future), conditional, subjunctive (present, imperfect), imperative. A form whose readings name the indicative and the subjunctive of one tense, with the same persons and number, SHALL name the two moods once, as one tense. A present participle SHALL be named as such, and a past participle as the past participle beside it. On a card opened on its own dictionary form, a plural spelled like that form SHALL give no line; a numeral's form, and a determiner's or pronoun's plural that names no gender, SHALL give no line. A line naming two genders of one number SHALL name them once. What a French card names SHALL be decided once, the same in every renderer, the French one included. The cards of English and Spanish words SHALL read as before, in every interface language.

#### Scenario: A form of five readings
- **WHEN** the interface language is English and the card opens on « parle » read as a form of « parler »
- **THEN** one line reads "first- and third-person singular present indicative or subjunctive and second-person singular imperative of parler", with « parler » marked as French

#### Scenario: The same form in Spanish
- **WHEN** the interface language is Spanish and the card opens on « parle »
- **THEN** the line reads « primera y tercera persona del singular del presente de indicativo o de subjuntivo y segunda persona del singular del imperativo de parler »

#### Scenario: The passé simple
- **WHEN** the card opens on « fut » read as a form of « être »
- **THEN** the English line names the third-person singular past historic of « être », and the Spanish line the « tercera persona del singular del pretérito perfecto simple de indicativo de être »

#### Scenario: The indicative before the subjunctive
- **WHEN** the card opens on « finissions » read as a form of « finir »
- **THEN** the line names the first-person plural imperfect indicative or subjunctive, then the first-person plural present subjunctive, in that order in each language

#### Scenario: A present participle
- **WHEN** the card opens on « parlant »
- **THEN** the line reads "present participle of parler" in English and « participio presente de parler » in Spanish

#### Scenario: A noun spelled alike in both numbers
- **WHEN** the card opens on « temps », or on « un »
- **THEN** no line says that it may be the plural of itself, while the card of « fils » still says that it may be the plural of « fil »

#### Scenario: Two genders of one number
- **WHEN** the interface language is Spanish and the card opens on « sommes » read as a form of « être »
- **THEN** the line naming « somme » names its masculine and feminine plural once

#### Scenario: English and Spanish cards as before
- **WHEN** every form of the en-fr, es-fr, es-en and en-es packs is rendered in its pair's interface language
- **THEN** every line is byte for byte what it was before this requirement

### Requirement: The French card is pinned on the real packs
The word card of French SHALL be bounded by goldens of its own, its lines read from the goldens and never from the tables: the French invariance baseline over the committed fr-en tables, holding a grammar probe for each name the card says or leaves unsaid on purpose and lemmas whose gloss has several sense runs, and a fr-es golden — the French scenario glossed in Spanish over the committed fr-es tables — equal to it on the studied side. The English and Spanish lines the card renders from each — its grammar lines, gloss pages, headings and rows, and the whole-selection card of each phrase probe — SHALL be pinned in a snapshot per pair. The goldens and the snapshots SHALL be run by the checks that run the other baselines and re-blessed where they are; a change that moves one SHALL fail until it is re-blessed in the pull request that moves it, saying why. Until fr-es's tables are committed, the Spanish card's grammar lines SHALL be pinned on the French golden's readings.

#### Scenario: A probe of the five-reading form
- **WHEN** the French golden's probe of « parle » read as a form of « parler » is rendered with the interface in English
- **THEN** its grammar line, gloss headings, pages and row are the ones the snapshot pins, the order of its tenses included

#### Scenario: fr-es on the studied side
- **WHEN** the fr-es golden is compared with the French golden probe by probe
- **THEN** they differ only in the native side, the lines naming the packs, the notice, the licences and the backup's profile

#### Scenario: The golden moves
- **WHEN** a re-reduction of fr-en, of fr-es or of French's studied tables, or a renderer change, moves a line a golden or a snapshot pins
- **THEN** the checks fail until both are re-blessed in that pull request

#### Scenario: French readers unchanged
- **WHEN** en-fr's and es-fr's cards and rows, and es-en's and en-es's, are rendered after this change
- **THEN** every line is byte for byte what it was, and their snapshots pass as committed

## MODIFIED Requirements

### Requirement: Selection capture on any pointer
Selecting text SHALL capture that selection on every pointer — mouse, touch and keyboard shortcut alike — and open the panel with the source sentence extracted automatically.
The capture SHALL fire once the selection has settled, not on a particular input event, so
that a touch selection adjusted with the platform's own handles is captured like a mouse
drag. A selection of one word SHALL open that word's popup with the actions matching its
current status; a selection of several words, bounded in length, SHALL open the
whole-selection card, which "+ Deck" adds as a phrase card. A selection longer than the
bound SHALL capture nothing. Within an analysed block, a selection of one word whose pieces
the studied language's pre-pass gave spans of their own SHALL open the popup of the piece
the selection lies inside, as the reader made it, and SHALL open the whole-selection card
when it covers several of those pieces; a word whose pieces share one span SHALL open as a
word of one piece does.

#### Scenario: Capturing a phrase on a phone
- **WHEN** the reader selects three words on a touch device and lifts the finger
- **THEN** the panel shows the phrase and its source sentence, and "+ Deck" creates the card

#### Scenario: Adjusting a touch selection with the native handles
- **WHEN** the reader drags a selection handle to extend a one-word selection to three words
- **THEN** the panel follows the selection and ends on the three-word phrase, without any further gesture

#### Scenario: Capturing a phrase with the keyboard shortcut
- **WHEN** the user selects three words and presses the shortcut
- **THEN** the panel shows the phrase and its source sentence, and "+ Deck" creates the card

#### Scenario: Selecting a single already-known word
- **WHEN** the reader selects one word whose status is "known"
- **THEN** the popup opens for that word and does not offer "Je connais" again

#### Scenario: An over-long selection captures nothing
- **WHEN** the reader selects a passage longer than the phrase bound
- **THEN** no panel opens and the page keeps its selection

#### Scenario: A selection inside an elided word
- **WHEN** the reader drags over « homme » in « l’homme », whose pieces `le` and `homme` have spans of their own
- **THEN** the popup of `homme` opens, not the popup of `le`

#### Scenario: A selection inside an inversion
- **WHEN** the reader drags over « il » in « dit-il »
- **THEN** the popup of `il` opens, not the popup of `dit`

#### Scenario: A selection over the pieces of one word
- **WHEN** the reader double-clicks « l’homme », which the browser selects whole
- **THEN** the whole-selection card opens, with a row for `homme`

#### Scenario: An expression written as one word
- **WHEN** the reader double-clicks « D’abord » and the pack holds the expression `d'abord`
- **THEN** the card is headed `d'abord` with its gloss as the answer, and "+ Deck" creates a card that stores that gloss

#### Scenario: Pieces sharing one span
- **WHEN** the reader selects « don't », whose pieces `do` and `not` share its span
- **THEN** the popup of `do` opens, as before
