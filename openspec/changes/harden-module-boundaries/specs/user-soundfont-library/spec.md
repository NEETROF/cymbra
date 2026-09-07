## ADDED Requirements

### Requirement: The private library is erased with the account

Deleting an account SHALL erase that account's private soundfont library completely: the
library rows **and** the stored `.sf2` objects they reference. The objects live in the
private soundfont bucket, which is distinct from the score bucket — the erasure SHALL
target that bucket, and a cleanup that deletes from the score bucket SHALL NOT be
considered to satisfy this requirement.

Row deletion and object cleanup SHALL be atomic with the rest of the account erasure in
the sense that the cleanup cannot be lost: the object keys SHALL be captured and their
cleanup scheduled in the same transaction that removes the rows, and each object cleanup
SHALL be independently retryable if the store is transiently unavailable.

#### Scenario: Private fonts do not survive deletion

- **WHEN** an account holding private soundfonts is deleted
- **THEN** its private library rows are gone and its `.sf2` objects are removed from the
  private soundfont bucket

#### Scenario: A transient store failure does not orphan objects

- **WHEN** the object store is unavailable while an account is being erased
- **THEN** the rows are still removed and the object cleanup is retried until it succeeds

#### Scenario: The cleanup targets the private bucket

- **WHEN** a private font's object is cleaned up
- **THEN** the deletion is issued against the private soundfont bucket, not the score
  bucket

### Requirement: Personal-data tables are provably covered by erasure

The system SHALL fail its test suite when a table holding account-scoped personal data is
not covered by the account erasure path. Adding such a table without extending erasure
SHALL NOT be able to merge silently.

#### Scenario: A new uncovered personal table fails the build

- **WHEN** a table keyed by account is added and the erasure path is not extended
- **THEN** a test fails, naming the uncovered table

#### Scenario: A covered table passes

- **WHEN** a table keyed by account is added and the erasure path is extended
- **THEN** the test passes
