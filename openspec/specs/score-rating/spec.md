# score-rating Specification

## Purpose
TBD - created by archiving change improve-rating-deck-sourcing. Update Purpose after archive.
## Requirements
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

### Requirement: Per-score rating aggregate

The system SHALL derive, per catalog score, an aggregate of its ratings: an average
effective value on a single comparable scale and the number of ratings, plus a
breakdown of verdicts. When a rating has an explicit star value, that value SHALL feed
the average; when it has only a swipe verdict, the verdict's implied value SHALL be
used, where a `dislike` contributes a low value (pulling the average down), `like` a
mid-high value, and `love` the maximum. The aggregate SHALL update as ratings are added or changed and SHALL be readable
for hub ranking/recommendation.

#### Scenario: Aggregate reflects ratings

- **WHEN** several users rate the same score
- **THEN** the score's aggregate reports the average effective value and the rating
  count derived from those ratings

#### Scenario: Updated rating changes the aggregate

- **WHEN** a user changes their existing rating of a score
- **THEN** the score's aggregate reflects the new value, not the old one

#### Scenario: Stars and verdicts fold into one scale

- **WHEN** the aggregate is computed over ratings that mix explicit stars and swipe-only verdicts
- **THEN** each rating contributes its effective value (explicit stars, else the verdict's implied value)

### Requirement: Hybrid re-review flag from ratings

Ratings SHALL NOT change a score's moderation status. Instead, when a validated
(`accepted`) score accumulates at least a configured minimum number of ratings and its
aggregate falls at or below a configured threshold, the system SHALL flag that score as
**eligible for moderator re-review**, so the moderation back office (a later change) can
surface it. The thresholds SHALL be configuration, not hard-coded schema, and clearing
or acting on the flag is the moderator's decision — the flag itself never sets
`moderation_status`.

#### Scenario: Low-rated validated score is flagged for re-review

- **WHEN** an accepted score reaches the minimum rating count and its aggregate is at or
  below the configured threshold
- **THEN** the score is flagged as eligible for moderator re-review

#### Scenario: Flag does not change moderation status

- **WHEN** a score becomes eligible for re-review
- **THEN** its `moderation_status` remains `accepted` until a moderator decides otherwise

#### Scenario: Insufficient votes do not flag

- **WHEN** a score's aggregate is low but it has fewer than the configured minimum ratings
- **THEN** the score is not flagged for re-review

