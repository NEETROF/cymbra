## ADDED Requirements

### Requirement: The reader can hide the browser's bars

The reader page SHALL offer, in its reading toolbar, a control that puts the page in fullscreen, hiding the browser's address bar, tabs and toolbars, and the same control SHALL leave fullscreen. The control SHALL always say what pressing it does, including after the reader left fullscreen through the browser (Escape, the browser's own close control, the system back gesture). The control SHALL NOT be shown where the browser cannot put the page in fullscreen. Everything the reader page shows over the book — its toolbar and footer, the word popup, the table of contents, the text size panel and the in-page drawer — SHALL remain visible and usable in fullscreen, and SHALL stay clear of the screen's rounded corners and system indicators. When an action opens a surface the browser does not show in fullscreen, the page SHALL leave fullscreen before opening it. Fullscreen SHALL only be entered on the reader's own gesture: it SHALL NOT be entered when a book opens, nor remembered between books or visits.

#### Scenario: Reading in fullscreen

- **WHEN** the reader presses the fullscreen control while a book is open
- **THEN** the browser's bars disappear, the book, its toolbar and its footer fill the screen, and the control now offers to leave fullscreen

#### Scenario: Leaving through the browser

- **WHEN** the reader leaves fullscreen with Escape or the browser's own control
- **THEN** the browser's bars come back and the control offers to enter fullscreen again

#### Scenario: A browser without fullscreen

- **WHEN** the reader page opens in a browser that cannot put a page in fullscreen, such as Safari on iPhone or iPad
- **THEN** the toolbar shows no fullscreen control

#### Scenario: A word looked up in fullscreen

- **WHEN** the reader selects a word of the book in fullscreen
- **THEN** the word popup opens over the book, in fullscreen

#### Scenario: A panel the browser hides in fullscreen

- **WHEN** the reader asks for the review, the stats or the settings in fullscreen, on a browser that shows them in its side panel
- **THEN** the page leaves fullscreen and the side panel opens

#### Scenario: Opening a book does not enter fullscreen

- **WHEN** the reader opens a book, having read the previous one in fullscreen and left it
- **THEN** the book opens with the browser's bars shown, until the reader presses the control
