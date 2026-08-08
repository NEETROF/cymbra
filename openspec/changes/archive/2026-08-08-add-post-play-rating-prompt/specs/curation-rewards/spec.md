## MODIFIED Requirements

### Requirement: Coverage points awarded on rating

When a signed-in user rates a score from the app, the system SHALL award **coverage
points** immediately, sized by a **diminishing** function of how many ratings the score
already has (more for an under-rated score, approaching zero for a well-covered one),
subject to a **per-user daily cap**. Coverage points SHALL be awarded only when the user
**engaged** with the score first; a rating recorded without prior engagement records
normally but earns no coverage points. Engagement SHALL be recorded by **either**
previewing the score in the rating deck **or playing it**: opening a catalog score in the
player, and the ingest of a play session for that score, each record the same engagement
signal — the session ingest so that a score opened from the offline cache, which never
fetches bytes from the server, still counts. Engagement recording SHALL remain idempotent
per (user, score) and best-effort: failing to record it MUST NOT fail the preview, the
player open, or the session ingest. Point values, the diminishing curve, and the daily
cap SHALL be configuration.

#### Scenario: Rating an under-covered score earns more

- **WHEN** a user rates a score that has few existing ratings, after previewing it
- **THEN** they are awarded coverage points, more than for a score that is already well-covered

#### Scenario: Diminishing returns on well-covered scores

- **WHEN** a user rates a score that already has many ratings
- **THEN** the coverage points awarded approach zero

#### Scenario: Daily cap limits farming

- **WHEN** a user has reached the daily coverage-points cap
- **THEN** further ratings that day record normally but award no additional coverage points

#### Scenario: No engagement, no coverage points

- **WHEN** a rating is recorded without the user having previewed or played the score
- **THEN** the rating is stored but no coverage points are awarded

#### Scenario: Playing a score counts as engagement

- **WHEN** a user opens a catalog score in the player and later rates it
- **THEN** the rating is coverage-eligible exactly as if they had previewed it in the deck

#### Scenario: An offline-cached play still counts

- **WHEN** a user plays a catalog score served from the offline cache (no bytes fetched
  from the server) and the play session is ingested
- **THEN** engagement is recorded for that user and score

#### Scenario: Engagement recording is idempotent

- **WHEN** a user previews and also plays the same score, several times
- **THEN** engagement is recorded once for that (user, score) pair and the rating earns
  coverage points at most once

#### Scenario: Failing to record engagement does not fail the play path

- **WHEN** recording engagement fails while opening a catalog score in the player or
  ingesting a play session
- **THEN** the open and the ingest still succeed
