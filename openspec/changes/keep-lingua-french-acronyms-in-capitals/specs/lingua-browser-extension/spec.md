## ADDED Requirements

### Requirement: A word's status and card keep its dictionary form's capitals
The extension SHALL key a word's status, its card and the gloss the card carries by the dictionary form the engine gave the word, as given, capitals included, and SHALL lowercase only the key of an expression built from a selection's text, as before; a word the engine reads as an acronym in capitals (`CA`) SHALL keep a status and a card of its own, apart from the lowercase word it is spelled like (`ça`).
Every dictionary form the engine gives a word in lowercase SHALL be keyed exactly as before, so no
status, card, sync record or backup of English, Spanish or a lowercase French word moves.

#### Scenario: Marking the acronym known
- **WHEN** the reader clicks « Je connais » on « CA » in « Le CA a voté le budget. », read as the acronym `CA`
- **THEN** the status is stored under `CA`, the page shows `CA` known, and `ça` keeps the status it had

#### Scenario: Adding the acronym to the deck
- **WHEN** the reader clicks « + Deck » on that « CA »
- **THEN** a card is created under `CA`, carrying the pack's gloss of `CA`, with the source sentence

#### Scenario: A lowercase word
- **WHEN** the reader marks `maison` known
- **THEN** its status is stored under `maison`, as before

#### Scenario: An expression
- **WHEN** the reader adds the expression « Au revoir » to the deck
- **THEN** its card is keyed by its name in lowercase, as before
