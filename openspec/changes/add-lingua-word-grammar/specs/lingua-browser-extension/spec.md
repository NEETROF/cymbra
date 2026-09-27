## ADDED Requirements

### Requirement: The word card says what the form is
The word card SHALL say what the form the reader met is, in words of the interface language and never as a grammatical code: its readings as the card's dictionary form, the other dictionary forms it may also be, and, for a word the analyser split, the pieces it was split into.
- **What the form is.** Its readings as the card's dictionary form, beside the form as seen,
  SHALL be joined into one statement. A form spelled like its dictionary form SHALL show this
  statement only when the pack gives it a reading other than the dictionary form itself, and SHALL
  present that reading as a possibility.
- **Other dictionary forms.** Each SHALL be named, with its reading, as another possibility. It
  SHALL be text only: it SHALL NOT open a card, and it SHALL NOT change the key of this card,
  its status or its actions.
- **Pieces.** A word the analyser split SHALL show as its pieces joined.
- **Nothing to say.** A form with no reading, no other dictionary form and no pieces SHALL show
  the card as it is shown today.

These lines SHALL sit below the controls that read the word aloud and above the actions, so that a
card completing its answer moves nothing the reader can press.

The names of parts of speech, gender, number and person SHALL be the interface's own. A name that
depends on the studied language, such as the name of a tense, SHALL be provided for English.

#### Scenario: An irregular form
- **WHEN** the reader opens the card of `went`
- **THEN** the card headed `go` says that `went` is the prétérit of `go`

#### Scenario: A form with two readings
- **WHEN** the reader opens the card of `walked`
- **THEN** the card headed `walk` says, in one statement, that `walked` is its prétérit and its participe passé

#### Scenario: A form spelled like its dictionary form
- **WHEN** the reader opens the card of `put`
- **THEN** the card headed `put` says that it may also be its prétérit and its participe passé

#### Scenario: A form of two dictionary forms
- **WHEN** the reader opens the card of `leaves`
- **THEN** the card headed `leave` says that `leaves` may also be the plural of `leaf`, and its actions act on `leave`

#### Scenario: A contraction
- **WHEN** the reader opens the card of the `do` in `don't`
- **THEN** the card headed `do` shows `don't` as `do` + `not`

#### Scenario: A pack without grammar tables
- **WHEN** the reader opens the card of `went` and the pack carries no grammar tables
- **THEN** the card shows the form seen as it does today, and nothing about its reading

#### Scenario: No code reaches the reader
- **WHEN** any word card is shown
- **THEN** no Universal Dependencies code (such as `VERB` or `Tense=Past`) and no occurrence of "lemma" or "lemme" appears in it

### Requirement: The word card lays its gloss out by part of speech
The word card SHALL show a gloss whose senses the pack groups by part of speech as one group per part of speech, each under the interface's name for that part of speech — with the word's gender when the pack gives one — in the order the pack gives them.
A group whose part of speech has no name for the reader, such as a symbol, an affix or an
unclassified entry, SHALL show without a heading. Until the grammar answer has arrived, and when it
does not come, the card SHALL show the gloss as a single text, as it does today. A card created from
this card SHALL store the gloss as the pack writes it, one text, exactly as before. Word-by-word rows
SHALL keep taking the first sense of each gloss.

#### Scenario: A word with a noun sense and verb senses
- **WHEN** the reader opens the card of `can`
- **THEN** the card shows « Boîte de conserve » under the heading for a noun, then the verb senses under the heading for a verb

#### Scenario: A word with one part of speech
- **WHEN** the reader opens the card of `put`
- **THEN** its gloss shows under the heading for a verb

#### Scenario: A gendered noun
- **WHEN** a test pack gives the noun `leche` the feminine gender and the reader opens its card
- **THEN** the heading names a feminine noun

#### Scenario: A sense with no part of speech to name
- **WHEN** a word's senses include a group whose part of speech is a symbol
- **THEN** that group shows without a heading

#### Scenario: The stored gloss does not change shape
- **WHEN** the reader presses "+ Deck" on the card of `can`
- **THEN** the card is created with the pack's gloss for `can` as one text

### Requirement: A card holding its gloss waits only briefly for its grammar
A word card SHALL ask for the word's grammar when it opens, and a card that already holds the pack's gloss for its word SHALL NOT wait for that answer beyond a short bound: past it, the card SHALL complete as it would with no grammar, and an answer arriving afterwards SHALL be dropped.
The card SHALL become complete exactly once, so nothing the reader can press moves or changes
meaning afterwards. A card that already holds its gloss SHALL NOT be drawn pending: it is drawn
once, complete, when its grammar arrives or the bound passes. A card that holds no gloss yet SHALL
receive its gloss in the same answer as its grammar,
under the wait and fallback that already apply to it. Examples are a known word, whose gloss the page
analysis withholds, and a word outside the page analysis.

#### Scenario: An engine that answers in the page
- **WHEN** the reader opens a highlighted word's card and the engine runs in the content script
- **THEN** the card appears complete, with its grammar, and is never drawn pending

#### Scenario: An awake engine in the event page
- **WHEN** the reader opens a highlighted word's card and the engine in the event page answers within the bound
- **THEN** the card completes once, with its grammar, its gloss and its actions together

#### Scenario: A cold engine
- **WHEN** the reader opens a highlighted word's card and the engine's host is suspended past the bound
- **THEN** the card completes with the gloss from the page analysis and its actions, without grammar, and the late answer changes nothing

#### Scenario: A known word
- **WHEN** the reader opens the card of a known word
- **THEN** its gloss and its grammar arrive in one answer, and the card completes once with both
