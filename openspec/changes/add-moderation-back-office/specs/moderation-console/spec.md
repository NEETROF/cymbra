## ADDED Requirements

### Requirement: Evaluate a score's moderation status with audit

The backend SHALL expose an authenticated operation, restricted to `moderator`/`admin`
identities, that sets a catalog score's moderation status to `accepted` or `rejected`
(and MAY set it back to `pending` to re-queue). The operation SHALL, in the same update,
record the reviewing account (`reviewed_by`) and the review time (`reviewed_at`), so a
rejection is traceable to a specific moderator and moment. A non-`moderator`/non-`admin`
caller MUST be rejected with `PERMISSION_DENIED`. Setting the status of a non-existent
score MUST be rejected.

#### Scenario: Moderator accepts a score

- **WHEN** a moderator sets a `pending` score to `accepted`
- **THEN** the score becomes `accepted` (visible in the hub) and records that moderator and the time

#### Scenario: Moderator rejects a score, traceably

- **WHEN** a moderator sets a score to `rejected`
- **THEN** the score becomes `rejected` and its `reviewed_by`/`reviewed_at` identify who rejected it and when

#### Scenario: Re-queue to pending

- **WHEN** a moderator sets an `accepted` score back to `pending`
- **THEN** the score leaves the hub and returns to the review queue

#### Scenario: Unauthorized evaluate rejected

- **WHEN** a caller without `moderator`/`admin` invokes the evaluate operation
- **THEN** it is rejected with `PERMISSION_DENIED` and no status changes

### Requirement: Back-office catalog table with hub filters and a privileged status filter

The back office SHALL present all catalog scores in a simple table, filterable by the
**same filters available in the app hub** (free-text title/composer, author, difficulty,
and the musical facets), plus a **back-office-only moderation-status filter** that is
never exposed in the Flutter app and that only `moderator`/`admin` identities may use.
The table SHALL show each score's key fields including its moderation status. Selecting a
row SHALL open the score for review.

#### Scenario: Table lists scores with the hub filters

- **WHEN** a moderator filters the table by text, author, difficulty, or a facet
- **THEN** the results match those filters exactly as the app hub would, but across all statuses the moderator is allowed to see

#### Scenario: Status filter is back-office only

- **WHEN** a moderator filters the table by moderation status (e.g. `pending`)
- **THEN** only scores of that status are listed, using the privileged filter available only to moderators/admins

#### Scenario: Row shows status and opens for review

- **WHEN** a moderator selects a table row
- **THEN** the score opens for review showing its current moderation status

### Requirement: Review queue prioritizes pending and flagged scores

The back office SHALL offer a default queue view that orders the work to do: unreviewed
(`pending`) scores first, followed by `accepted` scores flagged for re-review by the app
rating signal (when that signal is available). When the rating signal is absent, the
queue SHALL still function on `pending` scores alone.

#### Scenario: Pending scores lead the queue

- **WHEN** a moderator opens the queue view
- **THEN** `pending` scores are presented as the primary work to review

#### Scenario: Flagged accepted scores appear for re-review

- **WHEN** an `accepted` score has been flagged for re-review by ratings
- **THEN** it appears in the queue for a moderator to re-evaluate

#### Scenario: Queue works without the rating signal

- **WHEN** the rating re-review signal is not yet available
- **THEN** the queue still lists `pending` scores to review

### Requirement: Read-only score preview in the back office

Before deciding, a moderator SHALL be able to preview the selected score — its metadata
and its notation — in a **read-only** view, using score bytes served by the backend
(moderators may fetch bytes of non-`accepted` scores). The notation SHALL be rendered
faithfully to how the app renders it, so the moderator judges the score as users will
see it. The preview MUST NOT allow editing the score.

#### Scenario: Moderator previews before deciding

- **WHEN** a moderator opens a `pending` score in the console
- **THEN** its metadata and notation are shown read-only so the moderator can judge it

#### Scenario: Preview does not modify the score

- **WHEN** a moderator previews a score
- **THEN** no change is made to the score until an explicit accept/reject action

### Requirement: Back-office web application delivery

The back office SHALL be delivered as a client-rendered single-page web application
served at the back-office origin (`bo.cymbra.app`), authenticating through Cymbra
sign-in and calling the backend over the browser gRPC surface. Access to the application
SHALL be limited to `moderator`/`admin` identities: a signed-in user without those roles
MUST NOT be able to perform moderation actions, and MUST be shown an access-denied state.

#### Scenario: Authorized user reaches the console

- **WHEN** a `music/moderator` or `music/admin` signs in at the back-office origin
- **THEN** the console loads and its moderation actions are available

#### Scenario: Unauthorized signed-in user is denied

- **WHEN** a signed-in user without `moderator`/`admin` opens the back-office origin
- **THEN** the console shows an access-denied state and the moderation operations reject their calls

#### Scenario: Unauthenticated visitor cannot use it

- **WHEN** an unauthenticated visitor opens the back-office origin
- **THEN** they are prompted to sign in and cannot perform any moderation action
