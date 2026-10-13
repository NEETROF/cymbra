## MODIFIED Requirements

### Requirement: A missed card comes back in the same session
A card answered as missed SHALL come back in the same session after three other cards, or at the end when fewer remain, until it is recalled or has been asked three times in that session. Only a card's first answer in a session SHALL update its FSRS state and count as a review; a later answer in the same session SHALL only keep the card in the session or take it out. A card shown again in a session SHALL say so above its sentence, in place of the count of cards remaining, with the time it is being asked (French UI copy: « À revoir · 2ᵉ passage », « À revoir · 3ᵉ passage »); its first showing SHALL say nothing of the kind.

#### Scenario: A word recovered
- **WHEN** the reader misses `seldom`, then recalls it when it comes back
- **THEN** `seldom` is graded `again` once, leaves the session, and counts as recovered

#### Scenario: Missed three times
- **WHEN** the reader misses the same card three times in one session
- **THEN** the card leaves the session and waits for its next due date

#### Scenario: One review counted
- **WHEN** a missed card comes back and is answered again
- **THEN** the day's count of reviews holds one review for that card, not two

#### Scenario: A card that comes back says why
- **WHEN** the reader misses `seldom` and it comes back three cards later
- **THEN** the card shows « À revoir · 2ᵉ passage » above its sentence instead of the count of cards remaining

#### Scenario: Its third showing
- **WHEN** the reader misses `seldom` again when it comes back
- **THEN** its next showing says « À revoir · 3ᵉ passage »

#### Scenario: A first showing
- **WHEN** a card is shown for the first time in a session
- **THEN** it shows the count of cards remaining, as before

#### Scenario: In every host
- **WHEN** a missed card comes back in the side panel, in the in-page drawer or in the Cymbra Lingua app
- **THEN** each says it the same way, in the reader's interface language
