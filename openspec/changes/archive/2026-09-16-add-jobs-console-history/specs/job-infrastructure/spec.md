## MODIFIED Requirements

### Requirement: Narrow queue-administration access

Queue administration SHALL go through a dedicated database role. In the `jobs` schema, that role's only privileges MUST be schema usage and the right to execute the queue-administration functions. These functions:

- list jobs;
- count jobs by state;
- compute period statistics, in total and per job kind;
- list finished attempts;
- count finished attempts;
- list recurring schedules;
- cancel a job.

These functions MUST NOT return a job payload, a schedule payload or attempt error text. The role MUST have no table privileges, so it cannot read job payloads, schedule payloads or dead-letter records, nor modify queue tables directly. It MUST NOT be granted to any module role. Provisioning the role and migrating the queue SHALL converge to the same grants whichever runs first, including for functions added by a later migration.

#### Scenario: Administration role lists and cancels through functions only

- **WHEN** the administration role lists jobs and cancels one
- **THEN** both succeed through the administration functions

#### Scenario: Administration role reads history and schedules through functions only

- **WHEN** the administration role lists and counts finished attempts, reads the per-kind period statistics and lists the recurring schedules
- **THEN** each call succeeds through an administration function, and none returns a payload or error text

#### Scenario: Administration role cannot touch queue tables

- **WHEN** the administration role selects from or deletes in the queue message, payload, attempt history, schedule or dead-letter tables
- **THEN** the database refuses with a permission error

#### Scenario: Role provisioned after the migration

- **WHEN** the queue migrations have already run and the role is provisioned afterwards
- **THEN** the role holds execute on every administration function, including the history and schedule functions, without re-running the migrations
