## ADDED Requirements

### Requirement: Job attempt history

The worker SHALL record every attempt of every job with its job id, job kind, channel, attempt number, start time, end time and outcome (`running`, `succeeded`, `failed`, `abandoned`). The start SHALL be recorded before the handler performs any side effect. A new attempt of a job whose previous attempt is still recorded as `running` SHALL close that previous attempt as `abandoned`. Attempt history and cancellation records older than 90 days SHALL be pruned.

#### Scenario: Successful job leaves a succeeded attempt

- **WHEN** a job runs and completes
- **THEN** its message leaves the queue and the history holds one `succeeded` attempt with start and end times

#### Scenario: Failed attempt is distinguishable from a running one

- **WHEN** a job's handler returns an error and the job waits for its retry
- **THEN** the history holds a `failed` attempt with an end time and no `running` attempt for that job

#### Scenario: Crashed worker's attempt is abandoned on reclaim

- **WHEN** a worker dies mid-attempt and another worker reclaims the job after the lease expires
- **THEN** the earlier attempt is closed as `abandoned` and a new `running` attempt is recorded

#### Scenario: Old history is pruned

- **WHEN** attempt or cancellation records are older than 90 days
- **THEN** the periodic sweep deletes them

### Requirement: Operator cancellation of a queued job

The queue SHALL support cancelling a job that is not currently executing: the job MUST be removed so it never runs, and the cancellation MUST be recorded with the job id, kind, channel, the cancelling operator and the time. Cancellation MUST be refused for a job whose attempt is executing (its lease is held), and for the kinds registered as non-cancellable: `purge_user`, `purge_score_object` and `purge_soundfont_object`. A cancellation racing with a worker's claim MUST end in exactly one of two ways: the cancellation is refused as running, or the handler does not run. Cancelling a job on an ordered channel MUST preserve the order of the remaining jobs: its successor waits for the cancelled job's predecessor, if any. Cancelling one occurrence of a scheduled job MUST NOT disable its schedule.

#### Scenario: Cancelled job never runs

- **WHEN** a ready job is cancelled
- **THEN** no worker ever executes it and a cancellation record names the operator

#### Scenario: Cancellation racing with a claim

- **WHEN** a worker claims a job at the same moment an operator cancels it
- **THEN** either the cancellation is refused because the job is running, or the handler is skipped because the job is gone, never both a successful cancellation and an execution

#### Scenario: Middle of an ordered chain

- **WHEN** jobs A (running), B and C are queued in order on one ordered channel and B is cancelled
- **THEN** C still waits for A and runs only after A finishes

#### Scenario: Erasure job cannot be cancelled

- **WHEN** a cancellation is requested for a queued `purge_user` job
- **THEN** it is refused and the job stays queued

#### Scenario: Scheduled occurrence cancelled

- **WHEN** an occurrence of a recurring job is cancelled
- **THEN** its schedule stays enabled and the next occurrence is enqueued as usual

### Requirement: Narrow queue-administration access

Queue administration SHALL go through a dedicated database role whose only privileges in the `jobs` schema are schema usage and execution of the queue-administration functions: list jobs, count jobs by state, compute period statistics, cancel a job. The role MUST have no table privileges, so it cannot read job payloads or modify queue tables directly. It MUST NOT be granted to any module role. Provisioning the role and migrating the queue SHALL converge to the same grants whichever runs first.

#### Scenario: Administration role lists and cancels through functions only

- **WHEN** the administration role lists jobs and cancels one
- **THEN** both succeed through the administration functions

#### Scenario: Administration role cannot touch queue tables

- **WHEN** the administration role selects from or deletes in the queue message or payload tables
- **THEN** the database refuses with a permission error

#### Scenario: Role provisioned after the migration

- **WHEN** the queue migration has already run and the role is provisioned afterwards
- **THEN** the role holds execute on the administration functions without re-running the migration

## MODIFIED Requirements

### Requirement: Bounded retries and dead-letter

Failed jobs SHALL be retried with exponential backoff up to a limit **configurable
per `(module, type)`**. On exhausting retries a job MUST be moved to a dead-letter
store and an alert raised. The system MUST NOT retry a job indefinitely. A job whose final attempt is still executing MUST NOT be dead-lettered until that attempt ends or has run past the running-attempt grace period, after which it is treated as crashed.

#### Scenario: Transient failure is retried then succeeds

- **WHEN** a job fails transiently and is retried within its configured limit
- **THEN** a later attempt succeeds and the job completes

#### Scenario: Exhausted job is dead-lettered

- **WHEN** a job fails more times than its configured limit
- **THEN** it is moved to the dead-letter store and an alert is raised, and it is no longer retried

#### Scenario: A poison job does not freeze its ordered channel

- **WHEN** a permanently-failing job is on an ordered channel
- **THEN** after its retries are exhausted it is dead-lettered and subsequent jobs in that channel proceed

#### Scenario: Final attempt still running is not dead-lettered

- **WHEN** the dead-letter sweep runs while a job's final attempt is executing within the grace period
- **THEN** the job is left in the queue, and it completes normally if the attempt succeeds

#### Scenario: Final attempt past the grace period

- **WHEN** a job's final attempt has been recorded as running for longer than the grace period
- **THEN** the sweep dead-letters the job and closes the attempt as `abandoned`
