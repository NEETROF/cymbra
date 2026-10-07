## ADDED Requirements

### Requirement: A card carries the language of its gloss
The server SHALL carry the language of every card's gloss as a field of the card operation, SHALL read an absent or empty value as `fr`, SHALL store it with the card, and SHALL return each card with the gloss language it stored. The key of a card SHALL NOT change: a card is (reader, studied language, client id), and when two devices of one account write the same card, the gloss language SHALL travel with the write that wins. The server SHALL store this label before any client that can create a non-French gloss syncs.

#### Scenario: An installed client that sends no gloss language
- **WHEN** a client built before the field pushes a card
- **THEN** the card is stored with the gloss language `fr`, and comes back with it

#### Scenario: A card glossed in English
- **WHEN** a client pushes a card whose gloss language is `en`
- **THEN** the card is stored with `en`, and every device that pulls it receives `en`

#### Scenario: Two devices of different native languages
- **WHEN** a reader's Spanish-native device and French-native device each push the same card, the French one last
- **THEN** one card row holds the French device's gloss, labelled `fr`, and both devices pull it so

#### Scenario: Every stored gloss today
- **WHEN** the migration runs on a database written before this change
- **THEN** every card reads the gloss language `fr`, and no row is rewritten

### Requirement: The server states that it stores the language of glosses and of statistics
The data-state read every sync begins with SHALL say whether the server stores the gloss language of cards and the native language of daily statistics, as one flag; a server that predates the flag SHALL answer false, and a server rolled back to one SHALL answer false again.

#### Scenario: A client checks before pushing
- **WHEN** a client reads the data state from this server
- **THEN** the flag is true, and the client may push cards glossed in another language and statistics with a native language

#### Scenario: A server that predates the flag
- **WHEN** a client reads the data state from a server built before this change
- **THEN** the flag is false, and the client withholds its non-French cards
