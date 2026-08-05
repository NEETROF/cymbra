## ADDED Requirements

### Requirement: Scoreless practice sessions are captured and delivered

The app SHALL, at the end of a **practice (selective) session**, capture a **scoreless
activity record** — a client-generated session id, the score practiced, and the timestamp with
the client's timezone, and **no** `SessionResult`/synchronization data — and write it to the
same **durable local outbox** that survives app restarts. Delivery SHALL reuse the reliable,
**idempotent, no-loss** pipeline: the record is removed only after the server acknowledges it,
retried with backoff otherwise, and de-duplicated by the client session id so a retry is a
no-op. A practice record SHALL be captured **once per practice session** (not once per loop
lap) and SHALL NOT enter the scored-session/leaderboard ingestion path.

#### Scenario: Practice session end enqueues a scoreless record
- **WHEN** a practice session ends
- **THEN** a scoreless activity record (session id, score, timestamp) is written to the durable
  outbox, carrying no synchronization percentage or session-result

#### Scenario: Practice record survives an app restart
- **WHEN** the app is killed after a practice session ended but before its record was acknowledged
- **THEN** the outbox entry is still present after the app restarts

#### Scenario: Practice delivery is idempotent
- **WHEN** a practice record is retried after a network failure
- **THEN** the server records it at most once (de-duplicated by client session id) and it is
  never double-counted

#### Scenario: Practice does not enter the scored path
- **WHEN** a practice record is delivered
- **THEN** it is aggregated only as activity and never produces a scored session or leaderboard entry
