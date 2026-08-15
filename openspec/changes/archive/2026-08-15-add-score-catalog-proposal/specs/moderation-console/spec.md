## ADDED Requirements

### Requirement: Review queue distinguishes and attributes user-proposed scores

The back-office review queue and catalog table SHALL visually **distinguish** a
user-proposed score from a crawler-ingested one (an origin indicator on the row), and for
a user-proposed score SHALL show the **proposer's pseudo** (display name), using the
privileged proposer fields the backend exposes only to `moderator`/`admin` reads. A
crawler-ingested score SHALL show its dataset origin as before and no proposer pseudo. The
proposer's identity MUST come from the privileged read only, so it never appears in any
surface available to a non-`moderator`/non-`admin` caller.

#### Scenario: User-proposed row shows origin and proposer

- **WHEN** a moderator views the queue and a row is a user-proposed score
- **THEN** the row is marked as a user proposal and shows the proposer's pseudo

#### Scenario: Crawler row is distinguished from a proposal

- **WHEN** a moderator views the queue and a row is a crawler-ingested score
- **THEN** the row shows its dataset origin and no proposer pseudo

#### Scenario: Proposer pseudo is not shown outside the console

- **WHEN** the same catalog is read by a non-`moderator`/non-`admin` caller
- **THEN** no proposer pseudo or id is present in the response

### Requirement: Catalog table defaults to all statuses and filters by source

The back-office catalog table SHALL default its moderation-status filter to **all
statuses** (a "Tous" option showing `pending` + `accepted` + `rejected` together), rather
than a single status, so a moderator sees the whole corpus by default; selecting a
specific status still narrows to it. The table SHALL also offer a **source filter** so the
moderator can restrict to an origin (e.g. user proposals vs a crawler dataset). Each row
SHALL show its own moderation status and its source, and — for a user-proposed row — the
**proposer's pseudo**. The score **detail** view SHALL likewise show the score's source
(and the proposer's pseudo when user-proposed). Both the all-statuses read and the source
filter are privileged (moderator/admin), authorised server-side.

#### Scenario: Catalog defaults to every status

- **WHEN** a moderator opens the catalog table without choosing a status
- **THEN** scores of every moderation status are listed (the "Tous" default)

#### Scenario: Filtering by source narrows the origin

- **WHEN** a moderator selects the user-proposal source
- **THEN** only user-proposed scores are listed

#### Scenario: Detail shows the source and proposer

- **WHEN** a moderator opens a user-proposed score's detail
- **THEN** its source and the proposer's pseudo are shown

### Requirement: Review queue surfaces a re-proposal justification

When a reopened (re-proposed) score appears in the review queue, the back office SHALL
show the proposer's **resubmission justification** so the moderator sees why it is being
resubmitted after a prior rejection. A score that has never been rejected/re-proposed
SHALL show no such justification.

#### Scenario: Reopened score shows its resubmission justification

- **WHEN** a moderator views a reopened score in the queue
- **THEN** the proposer's resubmission justification is shown

## MODIFIED Requirements

### Requirement: Evaluate a score's moderation status with audit

The backend SHALL expose an authenticated operation, restricted to `moderator`/`admin`
identities, that sets a catalog score's moderation status to `accepted` or `rejected`
(and MAY set it back to `pending` to re-queue). The operation SHALL, in the same update,
record the reviewing account (`reviewed_by`) and the review time (`reviewed_at`), so a
rejection is traceable to a specific moderator and moment. When the decision is
`rejected`, the operation SHALL accept and record a **rejection reason** (the moderator's
motive), which is surfaced back to the proposer of a user-proposed score; setting a status
other than `rejected` clears any stored reason. A non-`moderator`/non-`admin` caller MUST
be rejected with `PERMISSION_DENIED`. Setting the status of a non-existent score MUST be
rejected.

#### Scenario: Moderator accepts a score

- **WHEN** a moderator sets a `pending` score to `accepted`
- **THEN** the score becomes `accepted` (visible in the hub) and records that moderator and the time

#### Scenario: Moderator rejects a score with a reason, traceably

- **WHEN** a moderator sets a score to `rejected` with a reason
- **THEN** the score becomes `rejected`, its `reviewed_by`/`reviewed_at` identify who
  rejected it and when, and the rejection reason is recorded for the proposer

#### Scenario: Re-queue to pending

- **WHEN** a moderator sets an `accepted` score back to `pending`
- **THEN** the score leaves the hub and returns to the review queue

#### Scenario: Unauthorized evaluate rejected

- **WHEN** a caller without `moderator`/`admin` invokes the evaluate operation
- **THEN** it is rejected with `PERMISSION_DENIED` and no status changes
