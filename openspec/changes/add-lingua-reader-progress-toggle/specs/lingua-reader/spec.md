## ADDED Requirements

### Requirement: The reader chooses what the footer says of the progress
The reader page SHALL show at its footer, in turn at each tap, the percentage of the book read, the pages left in the chapter on screen (French UI copy: « Chapitre : N pages restantes »), and an estimate of the pages left in the book (French UI copy: « Livre : ≈ N pages restantes »), back to the percentage after the third. The pages left in the chapter SHALL be counted on the chapter as laid out on the reader's screen at the reader's text size. The pages left in the book SHALL be marked as an estimate. The choice SHALL be kept on the device and SHALL apply to every book; the percentage SHALL be shown until the reader changes it.

#### Scenario: Pages left in the chapter
- **WHEN** the reader, on the sixth of the ten pages of a chapter, taps the percentage at the footer
- **THEN** the footer says « Chapitre : 4 pages restantes »

#### Scenario: Pages left in the book
- **WHEN** the reader taps the footer while it shows the pages left in the chapter
- **THEN** the footer shows the pages left in the book, preceded by « ≈ »

#### Scenario: Back to the percentage
- **WHEN** the reader taps the footer while it shows the pages left in the book
- **THEN** the footer shows the percentage of the book read

#### Scenario: The last page of a chapter
- **WHEN** the footer shows the pages left in the chapter and the reader turns to its last page
- **THEN** the footer says it is the last page of the chapter

#### Scenario: Another book, another day
- **WHEN** the reader chose the pages left in the chapter, then opens another book after reloading the page
- **THEN** the footer shows the pages left in that book's chapter

#### Scenario: Larger text
- **WHEN** the footer shows the pages left in the chapter and the reader enlarges the text
- **THEN** the count follows the chapter as laid out at the new size
