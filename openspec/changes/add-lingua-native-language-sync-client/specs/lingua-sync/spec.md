## ADDED Requirements

### Requirement: A device sends the language of a gloss only to a server that stores it
A device SHALL send every card's gloss language with it — an absent label being French — and SHALL push a card whose gloss is not French only when the data state read at the start of the sync says the server stores the language of glosses and of statistics; otherwise it SHALL hold that card and push it at a later sync. A device SHALL pull as a client that reads gloss languages, SHALL apply a pulled card with the gloss language the server returns, and SHALL reset its card cursor and pull the cards again from the start, once, the first time it pulls so, since the server withheld non-French cards from it before; its status cursor SHALL stand, since no status was withheld. A device SHALL NOT apply the echo of a card it held this sync from a server that does not store the labels: such a server returns the card with no label, which would relabel it French.

#### Scenario: A server that stores the labels
- **WHEN** a device holding a card glossed in French and one glossed in English syncs with a server that answers it stores the labels
- **THEN** both are pushed, each with its gloss language

#### Scenario: A server that predates the labels
- **WHEN** the same device syncs with a server whose data state does not say it stores the labels
- **THEN** only the French-glossed card is pushed, and the English one is pushed at a later sync against a server that stores the labels

#### Scenario: A card pulled with its label
- **WHEN** a device pulls a card whose gloss language is `en`
- **THEN** the card is applied with `en`, and review shows a gloss the reader can read

#### Scenario: The first pull of this build
- **WHEN** a device updated to this build pulls for the first time, its card cursor past cards the server withheld from it
- **THEN** it pulls the cards from the start once, receives every card of its languages with its label, and pulls from its card cursor afterwards; its status cursor is kept

#### Scenario: A French card
- **WHEN** a device holding only French-glossed cards syncs with a server whose data state does not say it stores the labels
- **THEN** every card is pushed, as before

#### Scenario: A server that no longer stores the labels
- **WHEN** a device that pushed a card glossed in English to a server that stored the labels syncs with one that does not, and that server returns the card with no label
- **THEN** the card is held, its echo is not applied, the device's copy keeps its English gloss language, and a French-glossed card returned beside it is applied

## MODIFIED Requirements

### Requirement: Widening a device's languages pulls again from the start
A device SHALL remember the languages of its last successful sync, reading a device that never stored them as having accepted English alone. When it accepts a language it did not, it SHALL pull statuses and cards again from the start, so it receives what the server withheld or it skipped before. Narrowing the languages SHALL NOT reset anything.

#### Scenario: Adding Spanish
- **WHEN** a device that synced in English alone starts accepting Spanish
- **THEN** its next sync pulls statuses and cards from the start, and the cards it already holds stay as they are

#### Scenario: Updating the extension
- **WHEN** a device that synced before this change, with no stored languages, syncs in English alone
- **THEN** its status cursor is kept; its card cursor is reset once, by *The first pull of this build*, and nothing else is pulled again
