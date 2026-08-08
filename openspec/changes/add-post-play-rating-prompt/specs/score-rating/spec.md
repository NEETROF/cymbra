## ADDED Requirements

### Requirement: Read the caller's own rating of a score

The backend SHALL expose an authenticated read that returns **the caller's own** rating
of a single catalog score: whether they have rated it and, when they have, the recorded
verdict and star value. The read SHALL be identity-scoped — it never exposes another
user's rating — and SHALL answer for a `pending` or `accepted` score. An unknown or
`rejected` score SHALL be reported as *not rated* rather than leaking its existence
through a distinct error. Unauthenticated requests MUST be rejected.

#### Scenario: An existing rating is returned

- **WHEN** a signed-in user reads their own rating of a score they have rated
- **THEN** the read reports it as rated and returns the recorded verdict and star value

#### Scenario: An un-rated score reports no rating

- **WHEN** a signed-in user reads their own rating of a score they have never rated
- **THEN** the read reports it as not rated

#### Scenario: The read is identity-scoped

- **WHEN** another user has rated the same score but the caller has not
- **THEN** the caller's read still reports it as not rated and exposes nothing of the
  other user's rating

#### Scenario: Unknown or rejected scores leak nothing

- **WHEN** the read is made for a `rejected` or non-existent score id
- **THEN** it reports *not rated* without distinguishing that case from an un-rated score

#### Scenario: Unauthenticated read rejected

- **WHEN** the read arrives without a valid authenticated identity
- **THEN** the request is rejected
