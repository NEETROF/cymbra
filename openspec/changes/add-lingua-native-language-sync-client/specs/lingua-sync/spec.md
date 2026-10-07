## ADDED Requirements

### Requirement: A device sends the language of a gloss only to a server that stores it
A device SHALL send every card's gloss language with it — an absent label being French — and SHALL push a card whose gloss is not French only when the data state read at the start of the sync says the server stores the language of glosses and of statistics; otherwise it SHALL hold that card and push it at a later sync. A device SHALL pull as a client that reads gloss languages, SHALL apply a pulled card with the gloss language the server returns, and SHALL pull again from the start, once, the first time it pulls so, since the server withheld non-French cards from it before.

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
- **WHEN** a device updated to this build pulls for the first time, its cursor past cards the server withheld from it
- **THEN** it pulls from the start once, receives every card of its languages with its label, and pulls from its cursor afterwards

#### Scenario: A French card
- **WHEN** a device holding only French-glossed cards syncs with a server whose data state does not say it stores the labels
- **THEN** every card is pushed, as before
