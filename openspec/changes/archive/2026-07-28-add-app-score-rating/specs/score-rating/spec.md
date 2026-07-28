## ADDED Requirements

### Requirement: Submit and update a score rating

The backend SHALL expose an authenticated operation for a signed-in user to rate an
`accepted` catalog score, carrying a swipe verdict (`dislike` / `like` / `love`) and an
optional 1–5 star value. There SHALL be **at most one rating per user per score**:
re-rating the same score SHALL update (upsert) the existing rating rather than create a
duplicate. Rating a score that is not `accepted`, or that does not exist, MUST be
rejected. Unauthenticated requests MUST be rejected.

#### Scenario: First rating is stored

- **WHEN** a signed-in user rates an accepted score with a verdict and/or stars
- **THEN** a single rating is recorded for that user and score

#### Scenario: Re-rating updates in place

- **WHEN** a user who already rated a score submits a new rating for it
- **THEN** the existing rating is updated and no duplicate row is created

#### Scenario: Rating a non-visible score is rejected

- **WHEN** a normal user attempts to rate a `pending` or `rejected` score, or a
  non-existent score id
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
