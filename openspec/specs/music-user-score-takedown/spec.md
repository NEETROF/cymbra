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
trigger a removal: the **Private scores** page of the Music section, at
`/music/private-scores`, whose title states that it removes reported private scores —
scores that are never part of the catalog. The former `/takedowns` path SHALL redirect to
it, keeping its query. The removal action MUST require entering the reason and MUST
present an explicit confirmation step stating the action is irreversible before
invoking the backend. Admins without the music scope MUST NOT see the surface.

#### Scenario: Removal requires reason and confirmation

- **WHEN** an admin triggers a removal in the back-office
- **THEN** they must enter a reason and confirm an irreversible-action prompt
  before the backend call is made

#### Scenario: Out-of-scope admin sees nothing

- **WHEN** an admin without the music scope opens the back-office
- **THEN** the Private scores entry is not shown and `/music/private-scores` redirects them away

### Requirement: Private scores are reachable from and link back to an account

The account detail page SHALL offer a music-scope admin a link to that account's private scores, opening the Private scores page with the lookup already run for that owner, so the operator never copies an account id by hand. The link MUST NOT be shown to an admin without the `music` scope. The lookup criteria (owner id, title fragment) SHALL ride in the page URL: arriving on the page with criteria in the URL runs the lookup, and submitting the form updates the URL, so a lookup can be reloaded and shared. In the results, each owner id SHALL link to that owner's account detail page.

#### Scenario: From an account to its private scores

- **WHEN** a music-scope admin activates "Private scores" on an account's detail page
- **THEN** the Private scores page opens with that account as the owner criterion and lists its private scores without any further input

#### Scenario: Out-of-scope admin gets no link

- **WHEN** an admin without the `music` scope opens an account's detail page
- **THEN** no link to the account's private scores is shown

#### Scenario: A lookup survives a reload

- **WHEN** an admin searches by a title fragment and reloads the page
- **THEN** the same criteria are filled in and the same results are listed

#### Scenario: From a result to its owner

- **WHEN** an admin activates the owner id of a result
- **THEN** that owner's account detail page opens

