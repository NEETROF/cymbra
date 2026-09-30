# lingua-sync — a deck card carries its studied language

## ADDED Requirements

### Requirement: A card belongs to one studied language
`DeckService` SHALL carry the studied language of every card as a field of the card operation, SHALL read an absent or empty language as `en`, and SHALL identify a card on the server by (user, language, client id), so that the same client id in two languages is two cards. The server SHALL return each card with the language it stored.

#### Scenario: An installed client that sends no language
- **WHEN** a client built before this change pushes a card without a language
- **THEN** the server stores it as an English card, under the same key the client has always targeted, and last-write-wins applies as before

#### Scenario: The same lemma in two languages
- **WHEN** a client pushes a Spanish card and an English card whose client ids are both `son`
- **THEN** the server holds two cards, and an update to one leaves the other untouched

#### Scenario: A card comes back with its language
- **WHEN** a client that accepts Spanish pulls its cards
- **THEN** every Spanish card returned carries `es` and every English card carries `en`

### Requirement: Non-English cards reach only the clients that accept them
A pull SHALL name the languages the client accepts; the server SHALL return only cards in those languages and SHALL treat an empty list as English only, so that a client that predates card languages never receives a card it would mis-file. The cursor returned SHALL be the highest change sequence among the cards returned, or the request's cursor when none was; the server SHALL keep no memory of what it withheld, so a client that later accepts more languages can pull again from an earlier cursor.

#### Scenario: An installed client never sees a Spanish card
- **WHEN** an account holds Spanish cards and a client built before this change pulls with its usual request
- **THEN** it receives the English cards only, and its cursor advances no further than the last English card returned

#### Scenario: A client that accepts both languages
- **WHEN** a client pulls with `languages` set to `en` and `es`
- **THEN** it receives the cards of both languages in sequence order, tombstones included

#### Scenario: Widening the accepted set
- **WHEN** a client that used to pull English only pulls again from an earlier cursor with `es` added
- **THEN** it receives the Spanish cards it had never been sent, and re-receiving English cards it already holds changes nothing (idempotent last-write-wins)

### Requirement: The server states that it understands card languages
The data-state read every sync begins with SHALL say whether the server understands card languages, so that a client can withhold its non-English cards from a server that would store them under the old key.

#### Scenario: A client checks before pushing
- **WHEN** a client that holds Spanish cards reads the data state and the server answers that it understands card languages
- **THEN** the client may push them; had the answer been absent or negative, it pushes its English cards only

### Requirement: Language values are normalised on receipt
Every language value the Lingua services receive — on statuses, declared levels, daily stats and cards — SHALL be normalised the same way before it is stored or used as a filter: trimmed, lowercased, reduced to its primary subtag, bounded in length, and read as `en` when empty. The server SHALL NOT refuse a request over its language value.

#### Scenario: Regional and upper-case codes
- **WHEN** one device pushes a status for `es-ES` and another for `ES`
- **THEN** both land on the same (user, `es`, lemma) key

#### Scenario: A language the server has never seen
- **WHEN** a client pushes a card in `pt`
- **THEN** it is stored as a `pt` card and returned only to clients that accept `pt`
