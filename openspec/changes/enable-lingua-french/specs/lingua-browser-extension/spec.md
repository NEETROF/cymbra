## ADDED Requirements

### Requirement: French is studied by readers of English and Spanish
The extension SHALL know French as a studied language, as it knows English and Spanish: a stored profile naming French SHALL be read as studying French; French SHALL be offered, in Réglages and at onboarding, to a reader whose native language has a listed pair studying French, after that native language's default, and SHALL never be offered to a reader whose native language is French; a French document SHALL be read, highlighted, glossed, recorded and read aloud through the reader's pair of French; and every surface that names a studied language, asks or shows its level, or says that its levels are estimated SHALL say so of French in the interface language, as it does of Spanish. A French word on the card SHALL be marked as French inside the interface language's lines.

#### Scenario: An English speaker adds French
- **WHEN** fr-en is listed and a reader whose native language is English, studying Spanish, ticks French in Réglages
- **THEN** the profile holds Spanish then French, a French page is highlighted through fr-en, and the French level block is titled "Estimated French level" in the English interface

#### Scenario: A Spanish speaker adds French
- **WHEN** fr-es is listed and a reader whose native language is Spanish, studying English, ticks French in Réglages
- **THEN** the profile holds English then French, a French page is highlighted through fr-es, and the French level block is titled « Nivel de francés estimado » in the Spanish interface

#### Scenario: A Spanish speaker without fr-es
- **WHEN** fr-en is listed and fr-es is not, and a reader whose native language is Spanish opens Réglages
- **THEN** French is not offered, and the choice of studied languages stays hidden as before

#### Scenario: A French speaker
- **WHEN** a reader whose native language is French opens Réglages with fr-en and fr-es listed
- **THEN** English and Spanish are offered as before, French is not, and a French page is not read

#### Scenario: A stored profile naming French
- **WHEN** the stored backup's profile studies Spanish then French with English as the native language
- **THEN** every surface that reads the profile without an engine reads Spanish then French, and the backup is written in schema version 3

#### Scenario: A French word on the card
- **WHEN** a reader whose native language is English opens the card of a French word
- **THEN** the card's lines are English, and the French words in them are marked as French for the voices, the hyphenation and the spell-check

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
word of one piece does. When the pieces it covers are one word and elided pieces, or a French
inversion's verb and its pronouns, the selection SHALL open that word's popup — the verb's, for
an inversion — with its gloss and the actions matching its status, unless the pack holds the
selection as an expression, whose card it SHALL then open.

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
- **WHEN** the reader double-clicks « l’homme », which the browser selects whole, and the pack holds no expression `l'homme`
- **THEN** the popup of `homme` opens, with its gloss and the actions "I know it", "+ Deck" and "Ignore", not the whole-selection card

#### Scenario: A selection over an inversion's pieces
- **WHEN** the reader selects « dit-il » whole, whose pieces `dit` and `il` have spans of their own, and the pack holds no expression for it
- **THEN** the popup of `dit` opens, with its gloss of `dire` and a word's actions, the pronoun `il` set aside

#### Scenario: An expression written as one word
- **WHEN** the reader double-clicks « D’abord » and the pack holds the expression `d'abord`
- **THEN** the card is headed `d'abord` with its gloss as the answer, and "+ Deck" creates a card that stores that gloss

#### Scenario: Pieces sharing one span
- **WHEN** the reader selects « don't », whose pieces `do` and `not` share its span
- **THEN** the popup of `do` opens, as before
