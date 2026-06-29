## ADDED Requirements

### Requirement: Transactional enqueue

A producer SHALL be able to enqueue a job **atomically within its own business
transaction**, so that the job exists if and only if the business write commits.
If the transaction rolls back, the job MUST NOT be enqueued.

#### Scenario: Job committed with the business write

- **WHEN** a module enqueues a job inside the same transaction as a successful business write
- **THEN** the job is visible to workers only after that transaction commits

#### Scenario: Rollback drops the job

- **WHEN** the producing transaction rolls back after enqueuing a job
- **THEN** no job is left in the queue (no dual-write, no orphan job)

### Requirement: Jobs execute on the worker, not the request path

Jobs SHALL be executed by the dedicated worker process. The user-facing service
SHALL only enqueue jobs and MUST NOT perform job side effects inline on the request
path.

#### Scenario: Sign-up no longer blocks on email

- **WHEN** a user signs up
- **THEN** the verification email is enqueued as a job and the sign-up request returns without waiting for SMTP

#### Scenario: Email failure does not fail sign-up

- **WHEN** the email provider is unavailable at sign-up time
- **THEN** the account is created, the request succeeds, and the email job is retried later (no error returned for an account that exists)

### Requirement: Per-type ordering

The system SHALL run job types that require strict ordering sequentially, while
other types run in parallel. Sequential types MUST NOT be processed concurrently
within their type.

#### Scenario: Sequential type runs one at a time

- **WHEN** multiple jobs of a strictly-ordered type are enqueued
- **THEN** they are processed one at a time, in order

#### Scenario: Parallel types are not blocked by each other

- **WHEN** a parallel type and a sequential type both have pending jobs
- **THEN** the parallel type is processed concurrently and is not blocked by the sequential type

### Requirement: Bounded retries and dead-letter

Failed jobs SHALL be retried with exponential backoff up to a limit **configurable
per `(module, type)`**. On exhausting retries a job MUST be moved to a dead-letter
store and an alert raised. The system MUST NOT retry a job indefinitely.

#### Scenario: Transient failure is retried then succeeds

- **WHEN** a job fails transiently and is retried within its configured limit
- **THEN** a later attempt succeeds and the job completes

#### Scenario: Exhausted job is dead-lettered

- **WHEN** a job fails more times than its configured limit
- **THEN** it is moved to the dead-letter store and an alert is raised, and it is no longer retried

#### Scenario: A poison job does not freeze its ordered channel

- **WHEN** a permanently-failing job is on an ordered channel
- **THEN** after its retries are exhausted it is dead-lettered and subsequent jobs in that channel proceed

### Requirement: Recurring tasks run exactly once per occurrence across pods

Recurring tasks SHALL be defined by a schedule (cron + timezone) and enqueued by the
worker. For a given scheduled occurrence, exactly one job MUST be created even when
multiple worker replicas evaluate the same schedule.

#### Scenario: One job per occurrence despite multiple replicas

- **WHEN** several worker replicas evaluate the same due schedule at the same occurrence
- **THEN** exactly one job is enqueued for that occurrence (duplicates are discarded by an idempotency key)

#### Scenario: Disabling a schedule stops enqueuing

- **WHEN** a schedule is disabled
- **THEN** no further jobs are enqueued for it until it is re-enabled

### Requirement: At-least-once delivery with reclaim

Delivery SHALL be at-least-once: a job claimed by a worker that dies MUST become
claimable again so another worker can process it. Handlers SHALL be idempotent so a
re-delivered job causes no duplicate effect.

#### Scenario: Crashed worker's job is reclaimed

- **WHEN** a worker claims a job and dies before completing it
- **THEN** the job's claim expires and another worker processes it

#### Scenario: Re-delivery is side-effect-safe

- **WHEN** a job is delivered more than once
- **THEN** the handler's idempotency key prevents a duplicate effect

### Requirement: Module isolation is preserved

Enabling transactional enqueue SHALL NOT let a module read another module's data.
Module roles SHALL have only write (insert) access to the shared queue and MUST NOT
gain read access to other module schemas.

#### Scenario: Producer can enqueue but not read others' data

- **WHEN** a module enqueues a job into the shared queue
- **THEN** the insert succeeds, but the module still cannot read any other module's schema

### Requirement: Queue observability

The system SHALL expose queue state for monitoring: pending, in-flight, and failed/
dead-lettered jobs, with per-channel depth and age, so a dashboard and alerting can
be built on it.

#### Scenario: Operators can see queue health

- **WHEN** jobs are pending, in-flight, and dead-lettered
- **THEN** each state is observable (e.g. via SQL views surfaced in a dashboard), and dead-letters can drive an alert
