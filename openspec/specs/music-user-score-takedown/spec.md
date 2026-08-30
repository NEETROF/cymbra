# music-user-score-takedown Specification

## Purpose
TBD - created by archiving change add-private-score-catalog. Update Purpose after archive.
## Requirements
### Requirement: Admin lookup of private user scores

The backend SHALL expose to music-scope administrators a paged lookup of private
user scores by owner identifier and/or title fragment, returning the minimal
fields needed to act on a takedown notice (score id, owner id, title, composer,
size, creation date, rights basis). Callers without the music admin role MUST be
rejected. The lookup MUST NOT expose score bytes and MUST NOT be available to
regular users in any form.

#### Scenario: Admin finds a reported score

- **WHEN** a music-scope admin searches by owner and title fragment
- **THEN** matching private scores are returned with their identifying metadata

#### Scenario: Non-admin rejected

- **WHEN** a caller without the music admin role invokes the lookup
- **THEN** the request is rejected

### Requirement: Reasoned, audited, irreversible removal

The backend SHALL let a music-scope administrator remove a private user score
given its id and a **mandatory non-empty reason**. The operation MUST write an
audit record — acting admin, owner id, score id, content SHA-256, title, reason,
timestamp — **before** deleting the database row and the stored object. The
audit record MUST survive the deletion indefinitely. A request without a reason
MUST be rejected. After removal the score MUST no longer appear in the owner's
library nor be deliverable.

#### Scenario: Takedown removes score and leaves audit

- **WHEN** an admin removes a score with a reason
- **THEN** an audit record with admin, owner, score identity, and reason is
  persisted, and the score's row and object are deleted

#### Scenario: Missing reason rejected

- **WHEN** an admin submits a removal without a reason
- **THEN** the operation is rejected and nothing is deleted

#### Scenario: Owner no longer sees the score

- **WHEN** the owner reloads their library after a takedown
- **THEN** the removed score is absent

### Requirement: Back-office takedown surface with explicit confirmation

The back-office SHALL provide music-scope admins a screen to run the lookup and
trigger a removal. The removal action MUST require entering the reason and MUST
present an explicit confirmation step stating the action is irreversible before
invoking the backend. Admins without the music scope MUST NOT see the surface.

#### Scenario: Removal requires reason and confirmation

- **WHEN** an admin triggers a removal in the back-office
- **THEN** they must enter a reason and confirm an irreversible-action prompt
  before the backend call is made

#### Scenario: Out-of-scope admin sees nothing

- **WHEN** an admin without the music scope opens the back-office
- **THEN** the takedown surface is not reachable

