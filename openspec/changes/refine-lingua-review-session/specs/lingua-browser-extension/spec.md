## MODIFIED Requirements

### Requirement: Two review surfaces
The extension SHALL offer review in the **browser's native panel** where one exists (Side Panel on Chromium, sidebar on Firefox — the page is pushed, the panel survives navigation) and in a collapsible **injected panel** (shadow DOM) for micro-reviews. On Safari, which has no panel API, the injected panel SHALL carry in-browser review on its own. Every surface SHALL operate on the same local state. Lossless backup/restore (defined by `lingua-decks-review`) SHALL be reachable from Réglages › Données in the side panel and the injected panel, and SHALL NOT be part of the review page. The toolbar popup's Réglages SHALL offer the backup and point to the panel for a restore, since Firefox closes a popup when a file picker opens.

#### Scenario: Side panel during navigation
- **WHEN** the user opens the side panel and then navigates to another page
- **THEN** the side panel stays open and its review session continues

#### Scenario: Backup from the side panel
- **WHEN** the user opens Réglages › Données in the side panel and triggers a backup
- **THEN** a versioned backup file is downloaded containing the complete state (cards field by field, statuses, calibration, FSRS parameters), and re-importing it restores the state identically

#### Scenario: The toolbar popup
- **WHEN** the user opens Réglages › Données in the toolbar popup
- **THEN** it offers « Sauvegarder », and instead of « Restaurer » it says to open Réglages from the panel

#### Scenario: The review page
- **WHEN** the user opens the review page in any surface
- **THEN** it shows the due count, the language filter when several languages are studied, and the card, without backup, restore or the pack's sources

## ADDED Requirements

### Requirement: The review card fits the screen it is on
The review card SHALL keep the sentence as its main element, set in a reading typeface at the reader's text size and in the surfaces' current look, and SHALL reserve the space of its answer from the start, so that revealing the answer changes nothing else on the card. The two answers SHALL be two large zones, « Pas su » on the left and « Su » on the right. On a narrow screen held upright they SHALL sit at the bottom, above the system's gesture area; on a screen held sideways with little height they SHALL be bands along the left and right edges; on a wide screen the card SHALL be a centred column of reading width. While the review has the focus, Space or Enter SHALL reveal the answer and the left and right arrow keys SHALL answer « Pas su » and « Su ». The review card SHALL NOT animate.

#### Scenario: On an e-ink reader
- **WHEN** a reader using the e-ink colours reveals an answer
- **THEN** only the answer space changes: the sentence and the answers stay where they are, and nothing animates

#### Scenario: On a phone held upright
- **WHEN** the review is shown on a phone held upright
- **THEN** « Pas su » and « Su » fill the bottom of the screen, left and right, above the gesture area

#### Scenario: On a phone held sideways
- **WHEN** the same phone is turned sideways
- **THEN** the two answers become bands along the left and right edges, with the sentence between them

#### Scenario: On a tablet or a wide panel
- **WHEN** the review is shown on a wide screen
- **THEN** the card is a centred column of reading width, with « Pas su » on the left and « Su » on the right

#### Scenario: With a keyboard
- **WHEN** a reader on a computer presses Space, then the right arrow key, while the review has the focus
- **THEN** the answer is revealed, then the card is answered « Su »

#### Scenario: Keys on the page being read
- **WHEN** the injected panel is open over a web page but the focus is on the page
- **THEN** the arrow keys and Space act on the page, not on the review

### Requirement: The review card can be heard
The review card SHALL offer to hear its word, as its front shows it, and its sentence, before the answer is revealed and after it, with the voices and under the rules of the word card's read-aloud; once revealed, it SHALL also offer to hear the dictionary form when the sentence shows another form. Without a voice the word card could use, the review card SHALL offer nothing to hear. Hearing SHALL change nothing on the card but the button that reads, and another card SHALL stop what is being read.

#### Scenario: Before the reveal
- **WHEN** a card shows « They seldom ship. » with its answer hidden
- **THEN** « ▶ Mot » reads `seldom` and « ▶ Phrase » reads the sentence, in the card's language, and the answer stays hidden

#### Scenario: After the reveal
- **WHEN** the card of `grinning`, whose dictionary form is `grin`, is revealed
- **THEN** « ▶ Mot » and « ▶ Phrase » are still there, and « ▶ grin » reads the dictionary form beside the answer

#### Scenario: An expression
- **WHEN** a card holds several words
- **THEN** « ▶ Expression » reads them as one

#### Scenario: Another card
- **WHEN** the reader answers while a text of the card is being read
- **THEN** the reading stops

#### Scenario: No voice
- **WHEN** no voice of the card's language may speak on the device
- **THEN** the card offers nothing to hear
