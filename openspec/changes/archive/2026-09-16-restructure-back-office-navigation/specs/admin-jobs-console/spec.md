## MODIFIED Requirements

### Requirement: Global-admin-only access to the jobs console

The jobs console operations SHALL be available only to callers holding `admin` in the `global` scope. These operations are: listing queued jobs, listing finished job history, reading job statistics and their per-kind breakdown, listing job kinds with their schedules, and cancelling a job. A caller without that role MUST receive `PERMISSION_DENIED`, or `UNAUTHENTICATED` without a session, and no data. The back office SHALL show the Jobs navigation entry (Administration section) and serve its route, `/admin/jobs`, only to such admins; the former `/jobs` path SHALL redirect to it. The server-side check remains authoritative.

#### Scenario: Global admin reaches the console

- **WHEN** a `global/admin` opens `/admin/jobs`, or the former `/jobs`
- **THEN** the Jobs page loads with the queue table and statistics

#### Scenario: Module admin is refused

- **WHEN** a `music/admin` without `global/admin` calls any jobs console operation
- **THEN** the service returns `PERMISSION_DENIED` and no job data

#### Scenario: Module admin cannot read the history

- **WHEN** a `music/admin` without `global/admin` requests the finished job history
- **THEN** the service returns `PERMISSION_DENIED` and no attempt rows

#### Scenario: Module admin sees no entry point

- **WHEN** a `music/admin` without `global/admin` is signed in to the back office
- **THEN** no Jobs navigation entry is shown and navigating to `/admin/jobs` redirects away
