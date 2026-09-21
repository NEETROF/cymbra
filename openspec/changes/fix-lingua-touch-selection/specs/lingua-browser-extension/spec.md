## ADDED Requirements

### Requirement: Selection capture on any pointer
Selecting text SHALL capture that selection on every pointer — mouse, touch and keyboard shortcut alike — and open the panel with the source sentence extracted automatically.
The capture SHALL fire once the selection has settled, not on a particular input event, so
that a touch selection adjusted with the platform's own handles is captured like a mouse
drag. A selection of one word SHALL open that word's popup with the actions matching its
current status; a selection of several words, bounded in length, SHALL open the
whole-selection card, which "+ Deck" adds as a phrase card. A selection longer than the
bound SHALL capture nothing.

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

### Requirement: The reader never fights the platform's text selection
The reader SHALL NOT clear, suppress or pre-empt the host platform's native text selection.
It SHALL NOT cancel `selectstart`, SHALL NOT suppress the platform's selection callout or
context menu, and SHALL NOT remove the document's ranges on its own. Where the platform's
selection gesture and a reader gesture are the same physical gesture, the platform's
selection SHALL win, and the reader SHALL derive its behaviour from the resulting selection.
A platform-drawn selection menu appearing alongside the panel is accepted; the panel SHALL
be positioned so it does not sit under that menu.

#### Scenario: Press-and-hold on a phone
- **WHEN** the reader presses and holds a word on a touch device
- **THEN** the platform selects that word and its own menu appears, and the panel opens for that word with the actions matching its status

#### Scenario: Reclassifying a non-highlighted word on a phone
- **WHEN** the reader presses and holds a word marked "known" or "ignored"
- **THEN** the popup opens offering to reclassify it, reached through the platform selection rather than a competing long-press gesture

## REMOVED Requirements

### Requirement: Selection capture on a keyboard shortcut
**Reason**: The keyboard shortcut was the only way to capture a selection, which made phrase
capture unreachable on phones and tablets. Replaced by "Selection capture on any pointer",
which keeps the shortcut and adds the pointer-agnostic trigger.

**Migration**: None for the reader — the shortcut keeps working unchanged, and its scenario
is carried over verbatim into the new requirement.
