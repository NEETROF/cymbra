## ADDED Requirements

### Requirement: A device pulls the cards of its reader's languages
A device SHALL accept the reader's studied languages that its package ships, or the default pair's language when none is, and SHALL name them in every card pull. It SHALL file each pulled card under the language the server returns, reading an empty value as English. It SHALL NOT apply pulled statuses or declared levels in a language it does not accept.

#### Scenario: Every reader today
- **WHEN** a reader who studies English alone syncs
- **THEN** the card pull names English, and every card and status is applied as before

#### Scenario: A Spanish card comes back as Spanish
- **WHEN** a device that accepts English and Spanish pulls a card the server returns as Spanish
- **THEN** the card is filed under Spanish, next to an English card with the same spelling

#### Scenario: A status in a language the device does not accept
- **WHEN** a device that accepts English alone pulls a Spanish status
- **THEN** the status is not applied, and no Spanish pack is loaded

### Requirement: A device pushes a non-English card only to a server that keys cards by language
A device SHALL send every card's language with it. It SHALL push a card in a language other than English only when the data state read at the start of the sync says the server understands card languages, and SHALL otherwise push its English cards only.

#### Scenario: A server that understands card languages
- **WHEN** a device holding English and Spanish cards syncs with a server that answers it understands card languages
- **THEN** both are pushed, each with its language

#### Scenario: A server that predates card languages
- **WHEN** the same device syncs with a server whose data state does not say it understands card languages
- **THEN** only the English cards are pushed

### Requirement: Widening a device's languages pulls again from the start
A device SHALL remember the languages of its last successful sync, reading a device that never stored them as having accepted English alone. When it accepts a language it did not, it SHALL pull statuses and cards again from the start, so it receives what the server withheld or it skipped before. Narrowing the languages SHALL NOT reset anything.

#### Scenario: Adding Spanish
- **WHEN** a device that synced in English alone starts accepting Spanish
- **THEN** its next sync pulls statuses and cards from the start, and the cards it already holds stay as they are

#### Scenario: Updating the extension
- **WHEN** a device that synced before this change, with no stored languages, syncs in English alone
- **THEN** its cursors are kept and nothing is pulled again
