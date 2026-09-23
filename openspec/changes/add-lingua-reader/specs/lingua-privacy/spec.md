## MODIFIED Requirements

### Requirement: A card's page address stays on the device
Lingua SHALL keep the source of the encounter a card was captured from — the address of a page, or the title and section of a book read in the extension's reader — on the device only: it SHALL NOT be sent to the server, and the server SHALL NOT store one.
- The extension SHALL push every card with an empty `source`.
- Applying a pulled card SHALL keep the local source when the pulled `source` is empty.
- The server SHALL ignore any `source` it receives, return an empty one, and keep no stored source, including those stored before this change.
- `CardOp.source` SHALL remain in the protocol as a deprecated field so installed clients keep working.

#### Scenario: Capturing a word from a page
- **WHEN** a signed-in reader adds a word to the deck from a web page and the extension syncs
- **THEN** the pushed card carries the lemma, surface form, sentence, gloss and review state with an empty source, and the local card still holds the page address

#### Scenario: Capturing a word from a book
- **WHEN** a signed-in reader adds a word to the deck from a book in the extension's reader and the extension syncs
- **THEN** the pushed card carries an empty source, and the local card still holds the book's title and section

#### Scenario: A card edited on another device
- **WHEN** a card captured on the Mac is reviewed on the iPhone and the Mac then pulls that change
- **THEN** the Mac's card takes the iPhone's review state and keeps its own page address

#### Scenario: An installed extension still sends an address
- **WHEN** an extension built before this change pushes a card with a page address
- **THEN** the server accepts the card, stores no address, and returns the card with an empty source on the next pull

#### Scenario: Addresses stored before the change
- **WHEN** the backend migration of this change runs
- **THEN** no page address previously stored for any card remains in the database
