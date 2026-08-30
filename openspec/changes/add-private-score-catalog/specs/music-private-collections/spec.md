# music-private-collections — spec (add-private-score-catalog)

## ADDED Requirements

### Requirement: Owner-scoped collection lifecycle

The backend SHALL let an authenticated user create, rename, and delete named
**collections** in their private score library. Collections are strictly
owner-scoped: only their owner can see or mutate them. Collection names MUST be
unique per owner case-insensitively; creating or renaming to a name that would
collide MUST be rejected with a distinguishable error. Deleting a collection
MUST NOT delete or alter any score. Unauthenticated requests MUST be rejected.

#### Scenario: Create, rename, delete

- **WHEN** an authenticated user creates a collection, renames it, then deletes it
- **THEN** each operation succeeds and is visible only to that user

#### Scenario: Case-insensitive name collision rejected

- **WHEN** a user creates a collection named "Chopin" and then another named
  "chopin"
- **THEN** the second creation is rejected with a name-conflict error

#### Scenario: Deleting a collection keeps its scores

- **WHEN** a user deletes a collection containing scores
- **THEN** the collection disappears and every score remains in the library

### Requirement: Many-to-many membership between scores and collections

The backend SHALL let the owner add one of their private scores to one of their
collections and remove it, idempotently in both directions. A score MAY belong
to several collections simultaneously. Adding MUST validate that both the score
and the collection exist and belong to the caller. Deleting a score MUST
silently remove it from every collection it belonged to.

#### Scenario: Score in several collections

- **WHEN** the owner adds the same score to two of their collections
- **THEN** the score appears in both, and removing it from one leaves the other

#### Scenario: Membership is idempotent

- **WHEN** the owner adds a score to a collection it is already in
- **THEN** the operation succeeds without duplicate membership

#### Scenario: Cross-owner assignment rejected

- **WHEN** a caller tries to add a score or use a collection they do not own
- **THEN** the operation is rejected

#### Scenario: Deleting a score cleans its memberships

- **WHEN** the owner deletes a score that belongs to collections
- **THEN** the score's memberships disappear with it and the collections remain

### Requirement: Library filtering by collection

The app's private library SHALL let the owner filter their scores by one
collection, listing only that collection's scores, and SHALL offer an
unfiltered view of the whole library. The filter UI MUST reflect the user's
collections as fetched from the backend.

#### Scenario: Filter shows only the collection's scores

- **WHEN** the owner selects a collection as filter
- **THEN** only scores belonging to that collection are listed

#### Scenario: Clearing the filter restores the full library

- **WHEN** the owner clears the collection filter
- **THEN** the full private library is listed again

### Requirement: Collections sync across devices

Collections and their memberships SHALL be persisted server-side as the single
source of truth and reflected on every device of the same account when the
library is (re)loaded. A collection created, renamed, deleted, or re-composed on
one device SHALL appear accordingly on another device after its next library
load, with no device-local divergence.

#### Scenario: Collection changes appear on another device

- **WHEN** a user creates a collection and adds scores on device A, then loads
  the library on device B
- **THEN** device B shows the collection and its scores
