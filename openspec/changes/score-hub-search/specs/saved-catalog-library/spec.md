## ADDED Requirements

### Requirement: Save a catalog score to the caller's library

The backend SHALL let an authenticated caller save a catalog score (by its
catalog id) to their personal library, persisting an owner-scoped record that
references the catalog entry. Saving MUST validate that the referenced catalog id
exists before recording it. Saving the same catalog score twice for the same user
MUST be idempotent — it MUST NOT create a duplicate record and MUST NOT error.
Unauthenticated requests MUST be rejected.

#### Scenario: Save records the catalog score for the caller

- **WHEN** an authenticated caller saves an existing catalog id
- **THEN** an owner-scoped record linking the caller to that catalog id is persisted

#### Scenario: Saving is idempotent

- **WHEN** a caller saves a catalog id they have already saved
- **THEN** no duplicate record is created and the request succeeds

#### Scenario: Saving an unknown catalog id rejected

- **WHEN** a caller saves a catalog id that does not exist in the corpus
- **THEN** the request is rejected and nothing is persisted

#### Scenario: Unauthenticated save rejected

- **WHEN** a save request arrives without a valid authenticated identity
- **THEN** the request is rejected and nothing is persisted

### Requirement: Remove a saved catalog score

The backend SHALL let an authenticated caller remove one of their saved catalog
scores, deleting only the caller's own save record and never affecting the public
catalog entry or any other user's saves. Removing a catalog score the caller has
not saved MUST be a no-op success (idempotent), not an error.

#### Scenario: Owner removes their save

- **WHEN** the caller removes a catalog score they had saved
- **THEN** their save record is deleted and the catalog entry itself is untouched

#### Scenario: Removing a not-saved score is a no-op

- **WHEN** the caller removes a catalog score they had not saved
- **THEN** the request succeeds and nothing changes

#### Scenario: Removal is owner-scoped

- **WHEN** the caller removes a catalog score
- **THEN** only the caller's own save record is affected, never another user's

### Requirement: List the caller's saved catalog scores

The backend SHALL return the catalog scores the authenticated caller has saved,
and MUST NOT expose another user's saved list. Each returned entry SHALL carry at
least the catalog id, title, composer, and difficulty level (when known) so the
app can render the saved section without a second lookup, ordered
deterministically (e.g. most-recently-saved first).

#### Scenario: Caller lists only their own saves

- **WHEN** an authenticated caller lists their saved catalog scores
- **THEN** only records the caller saved are returned, each with catalog id, title,
  composer, and level (when known)

#### Scenario: Isolation across users

- **WHEN** two users have each saved different catalog scores
- **THEN** neither user's list contains the other's saves

#### Scenario: Saved list reflects a removed catalog entry

- **WHEN** a saved catalog score no longer exists in the corpus
- **THEN** the stale save is not surfaced as a broken entry (it is omitted or reconciled)

### Requirement: Saved library syncs across the account's devices

The saved-catalog library SHALL be persisted server-side and owner-scoped, so it is
the single source of truth for the account across devices. A save made from one
authenticated session SHALL be visible when the same account lists its saved scores
from another session or device, and a removal made from one session SHALL likewise be
reflected when another session lists the saved scores. The library SHALL NOT rely on
device-local-only storage.

#### Scenario: A save is visible from another session

- **WHEN** the account saves a catalog score in one session and then lists its saved
  scores from another session
- **THEN** the newly saved score appears in that other session's list

#### Scenario: A removal is reflected from another session

- **WHEN** the account removes a saved catalog score in one session and then lists its
  saved scores from another session
- **THEN** the removed score no longer appears in that other session's list

### Requirement: Saved catalog scores erased with the account

When a user account is deleted, the backend SHALL erase that user's saved-catalog
library records, so no saved-score data remains attributable to the removed
account. The public catalog entries themselves SHALL be unaffected.

#### Scenario: Account deletion purges saves

- **WHEN** a user account is deleted
- **THEN** that user's saved-catalog records are removed while the public catalog
  entries they referenced remain intact
