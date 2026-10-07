## ADDED Requirements

### Requirement: A card carries the language of its gloss
The server SHALL carry the language of every card's gloss as a field of the card operation, SHALL read an absent or empty value as `fr`, SHALL store it with the card, and SHALL return each card with the gloss language it stored. The key of a card SHALL NOT change: a card is (reader, studied language, client id), and when two devices of one account write the same card, the gloss language SHALL travel with the write that wins. A pull SHALL return cards of every gloss language only to a client that says it reads gloss languages, and SHALL withhold — not consume — the cards glossed in a language other than `fr` from a client that does not. The server SHALL store this label before any client that can create a non-French gloss syncs.

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

#### Scenario: A pull from a client that predates labels
- **WHEN** a client that does not say it reads gloss languages pulls, and the account holds a card glossed in French and one glossed in English
- **THEN** it receives the French-glossed card only, and the cursor returned does not pass the English one

#### Scenario: A pull from a client that reads labels
- **WHEN** a client that says it reads gloss languages pulls the same account
- **THEN** it receives both cards, each with its gloss language

### Requirement: The server states that it stores the language of glosses and of statistics
The data-state read every sync begins with SHALL say whether the server stores the gloss language of cards and the native language of daily statistics, as one flag, `language_labels`; a server that predates the flag SHALL answer false, and a server rolled back to one SHALL answer false again.

#### Scenario: A client checks before pushing
- **WHEN** a client reads the data state from this server
- **THEN** the flag is true

#### Scenario: A server that predates the flag
- **WHEN** a client reads the data state from a server built before this change
- **THEN** the flag is false, by the wire's default

## MODIFIED Requirements

### Requirement: Language values are normalised on receipt
Every language value the Lingua services receive — on statuses, declared levels, daily stats and cards — SHALL be normalised the same way before it is stored or used as a filter: trimmed, lowercased, reduced to its primary subtag, bounded in length; a studied language SHALL be read as `en` when empty, and a gloss language or a native language as `fr`. The server SHALL NOT refuse a request over its language value.

#### Scenario: Regional and upper-case codes
- **WHEN** one device pushes a status for `es-ES` and another for `ES`
- **THEN** both land on the same (user, `es`, lemma) key

#### Scenario: A language the server has never seen
- **WHEN** a client pushes a card in `pt`
- **THEN** it is stored as a `pt` card and returned only to clients that accept `pt`

#### Scenario: A regional code on a label
- **WHEN** a client sends the native language `es-MX` on a statistic, or the gloss language `EN` on a card
- **THEN** the statistic is stored with `es`, and the card with `en`

#### Scenario: A label the server has never seen
- **WHEN** a client sends the gloss language `tlh` on a card
- **THEN** the card is stored with `tlh`, and nothing is refused
