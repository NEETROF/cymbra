## ADDED Requirements

### Requirement: Deck prioritises un-rated scores

The app deck SHALL source its cards from the backend deck-sourcing read (the
caller's un-rated `accepted` scores, least-rated first) rather than the general
catalog search, so a score the user has already rated is not shown again and the
deck reaches its empty/last-card state once everything visible has been rated.

#### Scenario: The deck does not repeat rated scores

- **WHEN** the user has rated a score and returns to the deck later
- **THEN** that score is not presented again

#### Scenario: The deck empties after everything is rated

- **WHEN** the user has rated every score the deck can source
- **THEN** the deck shows its empty/last-card state instead of repeating cards

### Requirement: The rating invite eventually stops

The library "rate a few scores" invite SHALL stop appearing permanently once the
user has dismissed it a configured number of times, so a user who is not
interested is not nudged indefinitely. Recording a rating still resets the normal
snooze window; only repeated explicit dismissals reach the terminal stop.

#### Scenario: Repeated dismissals stop the invite for good

- **WHEN** the user has dismissed the invite the configured number of times
- **THEN** the invite is no longer shown, even after more days pass

#### Scenario: A single dismissal only snoozes

- **WHEN** the user dismisses the invite fewer times than the stop threshold
- **THEN** it is snoozed for the normal window and may appear again later
</content>
