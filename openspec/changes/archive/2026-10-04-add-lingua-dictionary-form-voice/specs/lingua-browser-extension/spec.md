## ADDED Requirements

### Requirement: The card reads a word's dictionary form when it differs from the form seen
When a word card shows a form seen that differs from its dictionary form, letter case aside, it SHALL offer two listen buttons for the word, each labelled with the text it reads: first the form as it appears on the page, then the dictionary form. Otherwise it SHALL offer one listen button for the word, as before. A selection of several words SHALL keep its one selection button. The dictionary form's button SHALL stop, switch and fall silent as the card's other listen buttons do.

#### Scenario: A form of another word
- **WHEN** the reader opens the card of `Es`, whose dictionary form is `ser`
- **THEN** the card offers « ▶ Es », which speaks `Es`, then « ▶ ser », which speaks `ser`, then the sentence button

#### Scenario: A word that is its own dictionary form
- **WHEN** the reader opens the card of `casa`, or of `Casa` at the start of a sentence
- **THEN** the card offers one « ▶ Mot » button and the sentence button, as before

#### Scenario: Several words
- **WHEN** the reader selects several words
- **THEN** the card offers « ▶ Sélection » and the sentence button, as before

#### Scenario: Switching to the dictionary form
- **WHEN** « ▶ Es » is speaking and the reader presses « ▶ ser »
- **THEN** `Es` stops and `ser` is spoken
