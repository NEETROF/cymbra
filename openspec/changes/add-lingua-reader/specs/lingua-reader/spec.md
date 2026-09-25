## ADDED Requirements

### Requirement: A library of the reader's own books
The extension SHALL offer a reader page holding a library of EPUB files the reader imports through the browser's file picker; each book SHALL be kept in a store the extension owns, on the device only, identified by the hash of its file, so that importing the same file again yields one book and not two. The library SHALL show each book's title, authors and cover when the file carries them, and SHALL let the reader delete a book, removing its file and its record. Deleting a book SHALL leave every card captured from it intact.

#### Scenario: Importing a book
- **WHEN** the reader picks an EPUB file in the reader page
- **THEN** the book appears in the library with its title, authors and cover, and its file is kept by the extension on the device

#### Scenario: Importing the same file twice
- **WHEN** the reader picks a file whose hash matches a book already in the library
- **THEN** the library still holds one book for it, and its reading position is kept

#### Scenario: A file named without its extension
- **WHEN** the reader picks a file the browser reports with a generic type or no `.epub` suffix, whose content is an EPUB
- **THEN** it is imported as an EPUB

#### Scenario: Deleting a book
- **WHEN** the reader deletes a book that cards were captured from
- **THEN** the book and its file are gone from the library, and the cards keep their sentence and their source text

### Requirement: A book opens offline
The reader page and every book in its library SHALL open with no network at all: the page, the rendering, the analysis engine and the pack SHALL come from the installed bundle and the device's storage. The extension SHALL ask the browser to keep the library's storage persistent where the browser offers it, and SHALL tell the reader when that was refused.

#### Scenario: Reading with no connection
- **WHEN** the device has no network and the reader opens a book from the library
- **THEN** the book renders, its words are highlighted, and the word popup answers from the pack

#### Scenario: Persistence refused
- **WHEN** the browser refuses to keep the extension's storage persistent
- **THEN** the library says so in the reader's language, and the books still open

### Requirement: A protected book is refused
The extension SHALL refuse at import an EPUB whose encryption declaration names a rights-management scheme, SHALL say so in one sentence in the reader's language, and SHALL store nothing of it. Font obfuscation SHALL NOT count as protection.

#### Scenario: A book protected by DRM
- **WHEN** the reader picks an EPUB whose `META-INF/encryption.xml` names a DRM scheme
- **THEN** the reader is told the book is protected and cannot be opened, and the library is unchanged

#### Scenario: A book with obfuscated fonts
- **WHEN** the reader picks an EPUB whose only encryption entries are font obfuscation
- **THEN** it is imported and renders

### Requirement: The reading module reads the book
The reader page SHALL mount the extension's reading module on the document of each section the renderer shows — the same blocks, the same analysis port, the same highlights, the same word popup, the same selection capture, the same drawer and the same statistics as on a web page — with no second implementation of any of them. The section's percentage of known words SHALL reach the toolbar badge as a page's does, and the popup SHALL recognise a reader tab and show that section's figures.

#### Scenario: A section is highlighted
- **WHEN** a section of a book is shown
- **THEN** its unknown and learning words are highlighted, clicking one opens the word popup, and selecting a phrase opens the selection card, as on a web page

#### Scenario: The popup on a reader tab
- **WHEN** the reader opens the toolbar popup while a book is shown
- **THEN** the popup shows the section's percentage and counts, and does not offer to analyse the page

#### Scenario: Statistics count the reading
- **WHEN** the reader reads a section in which studied words appear
- **THEN** those exposures count in the daily statistics as they would on a web page

### Requirement: A page turn paints once
The reader page SHALL reveal a section only once it has been analysed and its highlights painted, bounded by a cap after which the section is revealed regardless, and SHALL paint a section whole rather than by viewport window, so that turning a page inside a section paints nothing new. The flow SHALL be paginated by default with tap zones to turn and no transition; a scrolled flow SHALL be available as a setting.

#### Scenario: Turning a page within a section
- **WHEN** the reader turns to the next page of the current section
- **THEN** the page appears with its highlights already painted, in one paint

#### Scenario: Moving to the next section
- **WHEN** the reader moves to the next section
- **THEN** it appears with its highlights painted, or after the cap without them if the analysis has not answered, and never twice

#### Scenario: Scrolled flow chosen
- **WHEN** the reader chooses the scrolled flow in the settings
- **THEN** the book scrolls continuously and highlights still paint per section

### Requirement: The reading position is kept on the device
The extension SHALL remember, on the device, where the reader is in each book, and SHALL reopen a book there. The position SHALL be written after the reader moves and SHALL NOT be sent anywhere by this capability.

#### Scenario: Reopening a book
- **WHEN** the reader closes a book at a page and opens it again later
- **THEN** the book opens at that page

#### Scenario: Two windows on the same book
- **WHEN** the same book is open in two reader tabs and both move
- **THEN** the most recent move is the position kept

### Requirement: A card remembers its book
A card captured in the reader SHALL carry the book's title and the section's title as its source, in the same local-only field a page address uses, and the review surfaces SHALL show it where they show a page address today.

#### Scenario: Capturing a word in a book
- **WHEN** the reader adds a word to the deck from a section of a book
- **THEN** the card holds the sentence, the book's title and the section's title, and the field is pushed empty when the card syncs

### Requirement: Where the reader opens from
The library SHALL be reachable from the popup and from the settings view rendered in the drawer and the side panel, and every entry point SHALL lead to the same reader tab: opening it while it is already open SHALL focus that tab rather than open a second.

#### Scenario: Opening the library twice
- **WHEN** the reader opens the library from the popup while a reader tab already exists
- **THEN** that tab is focused and no second reader tab is created

### Requirement: The reader sets the text size and the page
The reader page SHALL let the reader enlarge or reduce the text of a book in steps, and SHALL let them read on a paper page, in the book's own colours, or on a dark page, where the book's text and background colours are replaced so its text stays legible. The choice SHALL apply to every book, SHALL be kept on the device, and SHALL be offered both in the reader's toolbar and in the settings.

#### Scenario: Enlarging the text
- **WHEN** the reader enlarges the text of an open book
- **THEN** the book's text grows, including text the book sizes in absolute units, and the book reopens at that size later

#### Scenario: Reading on a dark page
- **WHEN** the reader switches to the dark page
- **THEN** the page turns dark and the book's text light, whatever colours the book sets for its text, and pictures keep their own

#### Scenario: One choice, two places
- **WHEN** the reader changes the text size in the settings while a book is open
- **THEN** the open book follows it, and the reader's toolbar panel shows the same size

