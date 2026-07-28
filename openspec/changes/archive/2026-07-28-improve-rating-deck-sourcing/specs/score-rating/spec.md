## ADDED Requirements

### Requirement: Deck sourcing of un-rated scores, least-rated first

The backend SHALL expose an authenticated read that returns the caller's
**un-rated** `accepted` catalog scores — the scores the signed-in user has not yet
rated — for the rating deck. Results SHALL be ordered so that scores with the
**fewest existing ratings come first** (they most need community signal), with a
deterministic tiebreak so paging is stable, and SHALL be paginated. A score the
caller has already rated MUST NOT be returned. Non-`accepted` scores MUST NOT be
returned. Unauthenticated requests MUST be rejected.

#### Scenario: Only un-rated accepted scores are returned

- **WHEN** a signed-in user requests deck cards
- **THEN** every returned score is `accepted` and has not been rated by that user

#### Scenario: Already-rated scores are excluded

- **WHEN** the user has rated a score and requests deck cards again
- **THEN** that score is not offered again

#### Scenario: Least-rated scores come first

- **WHEN** deck cards are returned
- **THEN** scores with fewer existing ratings are ordered ahead of more-rated ones

#### Scenario: The deck empties when everything is rated

- **WHEN** the user has rated every accepted score they can see
- **THEN** the deck-sourcing read returns no cards
</content>
