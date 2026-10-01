## ADDED Requirements

### Requirement: The reader chooses the colours of the page
The reader page SHALL let the reader choose the background and text colours of the paper page and of the dark page, in the extension's colour settings, next to the highlight colours. On the paper page, the book's own text colours SHALL be kept unless the reader chose a text colour; on the dark page, the chosen text colour SHALL replace the book's, as the dark page does today. The presets SHALL set the page colours too, and the return to the default SHALL restore them. The choice SHALL be kept on the device and SHALL apply to an open book without a reload.

#### Scenario: A paper page for an e-ink screen
- **WHEN** the reader sets the paper page to white with black text
- **THEN** the open book turns to black text on white, and every book opened later follows

#### Scenario: The book's own colours kept on paper
- **WHEN** the reader changes the paper page's background and leaves its text colour unset
- **THEN** the book keeps the text colours it sets, on the new background

#### Scenario: The dark page's colours
- **WHEN** the reader changes the dark page's text colour
- **THEN** a book read on the dark page shows its text in that colour, whatever colour the book sets
