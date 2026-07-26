## ADDED Requirements

### Requirement: Result count reflects the server total

The Score Hub SHALL display, for the applied filters, the **total number of
matching scores on the server**, not merely the number of results currently
loaded in memory. When results are paged in incrementally, the displayed count
SHALL remain the server-reported total (plus any matching local uploads the hub
prepends), and SHALL NOT grow as further pages are appended. When the hub is
scoped to the user's own uploads ("my scores" mode), where no server total
applies, the displayed count SHALL reflect the matching local list.

#### Scenario: Count shows the server total, not the loaded page

- **WHEN** the applied filters match more scores than fit in the first page and
  only that page is loaded
- **THEN** the hub displays the server-reported total match count, not the number
  of entries currently in memory

#### Scenario: Count is stable across load-more

- **WHEN** the user loads further pages of the same filtered result set
- **THEN** the displayed count stays the server total and does not increase as
  pages are appended

#### Scenario: My-scores mode counts the local list

- **WHEN** the hub is scoped to the user's own uploads
- **THEN** the displayed count reflects the matching local uploads, since no server
  total applies
