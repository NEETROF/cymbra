## ADDED Requirements

### Requirement: The reader can end a session in progress
While a review or practice session is under way, the review SHALL offer to end it (French UI copy: « Terminer », its accessible name « Terminer la séance »), as a small link beside the count of cards remaining at the top of the card, on every card of the session and nowhere else, without asking for confirmation. Ending SHALL keep every answer already given in the session, SHALL leave every card not yet shown unanswered and due as it was, SHALL NOT ask again a missed card waiting to come back, its miss standing, and SHALL leave the current card unanswered, revealed or not. The review SHALL then show the end of the session, as when its last card is answered: what the session did, continuing (French UI copy: « Encore 10 ») when cards remain due, else practice when it applies; a session ended before any answer SHALL say it ended, without counts and without saying that nothing is due. The link SHALL add no block and no transition to the card, and SHALL be offered the same way in the side panel, the in-page drawer and the Cymbra Lingua app, in the reader's interface language (English UI copy: "End"; Spanish UI copy: « Terminar »).

#### Scenario: Ending after a few cards
- **WHEN** the reader answers 3 cards of a session of 10, recalling 2 and missing 1, then ends the session
- **THEN** the end says 3 reviewed, the 3 answers are kept, the 7 cards not shown stay due, and continuing is offered

#### Scenario: A missed card waiting to come back
- **WHEN** the reader misses `seldom` and ends the session before it comes back
- **THEN** `seldom` is not shown again in that session and keeps its `again` answer

#### Scenario: A card revealed, not answered
- **WHEN** the reader reveals a card and ends the session without answering it
- **THEN** the card keeps its schedule and is still due

#### Scenario: Ending at once
- **WHEN** the reader ends a session on its first card, before any answer
- **THEN** the end says the session ended, with no count, and offers to continue

#### Scenario: Continuing after ending
- **WHEN** the reader ends a session early and chooses to continue
- **THEN** a new session holds the cards still due, those not shown in the ended session among them

#### Scenario: Ending a practice session
- **WHEN** the reader ends a practice session
- **THEN** no card's schedule changes and the end of the practice session is shown

#### Scenario: Only during a session
- **WHEN** the review shows its start, or the end of a session
- **THEN** it offers no link to end a session
