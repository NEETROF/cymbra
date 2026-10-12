## ADDED Requirements

### Requirement: The words looked up in a book are remembered on the device
The reader SHALL remember, on the device and per book, each word or expression whose card the reader opened while reading it, with the form met, its sentence, the chapter it was met in and its language, once per word and chapter. This list SHALL NOT be synchronised, SHALL NOT be part of the backup, and SHALL be deleted with the book.

#### Scenario: A word looked up
- **WHEN** the reader opens the card of `countenance` in the third chapter of a book
- **THEN** `countenance` is remembered for that book with its sentence and that chapter

#### Scenario: Deleting the book
- **WHEN** the reader deletes the book from the library
- **THEN** the words looked up in it are deleted with it

#### Scenario: Another device
- **WHEN** the reader's other device synchronises
- **THEN** it receives nothing of the words looked up

### Requirement: The end of a chapter offers to sort the words looked up in it
When the reader moves from a chapter to the next one, the reader SHALL offer a recap of the words looked up in the chapter just left when at least three of them are still to sort. A word still to sort is one not in the deck, not known and not ignored. Each row SHALL show the word as met, its gloss and its sentence, and SHALL let the reader add it to the deck (French UI copy: « + Deck »), mark it known (French UI copy: « Je connais ») or leave it. The recap SHALL be dismissed in one action and SHALL NOT be offered when a book is opened. Once dismissed, it SHALL NOT be offered again until at least three more words have been looked up in that book.

#### Scenario: End of a chapter
- **WHEN** the reader turns from chapter 3 to chapter 4 after looking up five words in chapter 3, one of them already in the deck
- **THEN** a recap lists the four others, each with its gloss and its sentence

#### Scenario: Few words
- **WHEN** the reader leaves a chapter in which two words were looked up
- **THEN** no recap is offered

#### Scenario: Adding from the recap
- **WHEN** the reader adds `countenance` to the deck from the recap
- **THEN** the card holds the sentence and the chapter `countenance` was met in, as if added from its word card, and `countenance` leaves the list

#### Scenario: Dismissed
- **WHEN** the reader dismisses the recap and turns to the next chapter having looked up two more words
- **THEN** no recap is offered there

#### Scenario: Opening a book
- **WHEN** the reader opens a book with words still to sort
- **THEN** no recap is offered at the opening

### Requirement: The words to sort can be reached at any time
The reader's bar SHALL show how many words of the book are still to sort (French UI copy: « N à trier ») when there is at least one, and the Révision page SHALL show the same count when it is shown beside the book; either SHALL open the list of every word of the book still to sort, by chapter.

#### Scenario: From the reader's bar
- **WHEN** the reader taps « 6 à trier » in the reader's bar
- **THEN** the list of the six words opens, grouped by the chapter they were met in

#### Scenario: Nothing to sort
- **WHEN** every word looked up in the book is in the deck, known or ignored
- **THEN** neither the bar nor the Révision page shows a count
