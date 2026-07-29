## MODIFIED Requirements

### Requirement: Deck sourcing of un-rated scores, least-rated first

The backend SHALL expose an authenticated read that returns the caller's
**un-rated** catalog scores whose moderation status is **`pending` or `accepted`**
(never `rejected`) — the scores the signed-in user has not yet rated — for the rating
deck. Results SHALL be ordered so that scores with the **fewest existing ratings come
first** (they most need community signal), with a deterministic tiebreak so paging is
stable, and SHALL be paginated. A score the caller has already rated MUST NOT be
returned. `rejected` scores MUST NOT be returned. Unauthenticated requests MUST be
rejected.

#### Scenario: Un-rated pending and accepted scores are returned

- **WHEN** a signed-in user requests deck cards
- **THEN** every returned score is `pending` or `accepted` and has not been rated by that user

#### Scenario: Rejected scores are never sourced

- **WHEN** the deck-sourcing read runs
- **THEN** no `rejected` score is ever returned, whatever its rating count

#### Scenario: Already-rated scores are excluded

- **WHEN** the user has rated a score and requests deck cards again
- **THEN** that score is not offered again

#### Scenario: Least-rated scores come first

- **WHEN** deck cards are returned
- **THEN** scores with fewer existing ratings are ordered ahead of more-rated ones

#### Scenario: The deck empties when everything is rated

- **WHEN** the user has rated every pending/accepted score they can see
- **THEN** the deck-sourcing read returns no cards

### Requirement: Submit and update a score rating

The backend SHALL expose an authenticated operation for a signed-in user to rate a
catalog score whose moderation status is **`pending` or `accepted`**, carrying a swipe
verdict (`dislike` / `like` / `love`) and an optional 1–5 star value. There SHALL be
**at most one rating per user per score**: re-rating the same score SHALL update
(upsert) the existing rating rather than create a duplicate. Rating a `rejected` score,
or a score that does not exist, MUST be rejected. Unauthenticated requests MUST be
rejected.

#### Scenario: First rating is stored

- **WHEN** a signed-in user rates a `pending` or `accepted` score with a verdict and/or stars
- **THEN** a single rating is recorded for that user and score

#### Scenario: Re-rating updates in place

- **WHEN** a user who already rated a score submits a new rating for it
- **THEN** the existing rating is updated and no duplicate row is created

#### Scenario: Rating a rejected or unknown score is rejected

- **WHEN** a user attempts to rate a `rejected` score, or a non-existent score id
- **THEN** the request is rejected and no rating is recorded

#### Scenario: Unauthenticated rating rejected

- **WHEN** a rating request arrives without a valid authenticated identity
- **THEN** the request is rejected
