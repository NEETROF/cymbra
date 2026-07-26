## ADDED Requirements

### Requirement: Session stats are captured durably at session end

The app SHALL, at the end of a play session, capture that session's stats — a
client-generated session id, the score played, the timestamp with the client's timezone,
and the immutable session-result record produced by scoring (its **overall synchronization
percentage** and the other session metrics) — and write them to a **durable local outbox**
that **survives app restarts**. Capture SHALL happen independently of network availability,
so a session's stats are safe before any attempt to send them.

#### Scenario: Session end enqueues stats durably

- **WHEN** a play session ends
- **THEN** its stats are written to the durable outbox with a client-generated session id, before any network send

#### Scenario: Stats survive an app restart

- **WHEN** the app is killed after a session ended but before its stats were acknowledged
- **THEN** the outbox entry is still present after the app restarts

#### Scenario: Offline session is still captured

- **WHEN** a session ends while the device is offline
- **THEN** its stats are captured in the outbox and remain pending

### Requirement: Reliable, idempotent delivery with no loss

The app SHALL deliver each outbox entry to the server and remove it **only after the server
acknowledges** it. On failure (offline, timeout, or the server unable to record), the app
SHALL **retry with backoff** and keep the entry until acknowledged — an un-acknowledged
entry MUST NOT be dropped. Server ingestion SHALL be **idempotent** by the client session
id, so a retried delivery is a no-op and a session is **never double-counted**. Together
this guarantees **no session is lost and none is counted twice**.

#### Scenario: Retry until acknowledged

- **WHEN** a delivery fails because the server cannot record it
- **THEN** the entry is retained and retried with backoff until the server acknowledges it

#### Scenario: Acknowledged entry is removed

- **WHEN** the server acknowledges a session
- **THEN** its outbox entry is removed and not sent again

#### Scenario: Duplicate delivery does not double-count

- **WHEN** the same session id is delivered more than once (e.g. an ack was lost)
- **THEN** the server records it once and the duplicate is a no-op

#### Scenario: Nothing is lost across failures

- **WHEN** deliveries fail repeatedly and the app restarts between attempts
- **THEN** every captured session is eventually delivered exactly once, with none dropped

### Requirement: Server persists sessions and aggregates per day

The backend SHALL expose an authenticated, idempotent operation to record a play session
(keyed by the client session id) and SHALL persist it, then aggregate a user's sessions
**per day** into a play count and an average **overall synchronization percentage**, bucketed
by the **user's local day** (from the timezone sent with the session). Unauthenticated
requests MUST be rejected.

#### Scenario: Session is persisted idempotently

- **WHEN** an authenticated client records a session by its id
- **THEN** the session is stored, and re-recording the same id changes nothing

#### Scenario: Per-day aggregate reflects sessions

- **WHEN** a user has recorded several sessions on a given local day
- **THEN** that day's aggregate reports the play count and the average overall synchronization percentage

#### Scenario: Unauthenticated record rejected

- **WHEN** a record request arrives without a valid authenticated identity
- **THEN** it is rejected and nothing is stored

### Requirement: Play-data retention and erasure

The system SHALL bound stored play data and erase it with the account. Un-acknowledged
outbox entries MUST NOT be dropped by any retention rule (the no-loss guarantee). On the
server, the lightweight per-session summary and the per-day aggregate MAY be kept long-term,
while the heavy per-session detail (the full record used for replay) SHALL be pruned after a
configured retention period. On **account deletion**, **all** of that user's play sessions
and aggregates SHALL be deleted, so no play data outlives the account.

#### Scenario: Heavy detail pruned after the retention period

- **WHEN** a session's detailed record is older than the configured detail-retention period
- **THEN** the heavy detail is pruned while the lightweight summary/aggregate may remain

#### Scenario: Un-acked outbox entries are never pruned

- **WHEN** outbox entries remain un-acknowledged for a long time
- **THEN** they are retained (never dropped by retention) until delivered

#### Scenario: Account deletion erases play data

- **WHEN** a user's account is deleted
- **THEN** all of that user's play sessions and aggregates are deleted, leaving no orphaned play data
