## MODIFIED Requirements

### Requirement: Full-text search over title and composer

The backend SHALL expose an authenticated operation that searches the public
`catalog_scores` corpus by a free-text query, matching against both the score
title and the composer (author). Matching SHALL be case- and accent-insensitive
and SHALL tolerate partial words and minor misspellings (trigram similarity), so
a search-as-you-type query returns relevant results without an exact match. An
empty query SHALL return the corpus browsable (unfiltered by text), ordered
deterministically. Unauthenticated requests MUST be rejected.

For a normal (non-admin, non-moderator) caller the operation SHALL return only
scores whose moderation status is `accepted`; `pending` and `rejected` scores MUST
NOT appear in results. This visibility gate SHALL be enforced server-side and SHALL
compose with the text query and every other filter (a result must be `accepted` AND
satisfy the query AND all filters).

#### Scenario: Query matches title

- **WHEN** an authenticated caller searches for a term contained in a score's title
- **THEN** that score is returned in the results

#### Scenario: Query matches composer

- **WHEN** an authenticated caller searches for a term contained in a score's composer
- **THEN** that score is returned in the results

#### Scenario: Matching ignores case and accents

- **WHEN** a caller searches with different case or without accents than the stored value
- **THEN** the score still matches (e.g. "faure" matches "Fauré")

#### Scenario: Empty query browses the corpus

- **WHEN** a caller searches with an empty query
- **THEN** results are returned across the corpus in a deterministic order, not an error

#### Scenario: Unauthenticated search rejected

- **WHEN** a search request arrives without a valid authenticated identity
- **THEN** the request is rejected and no results are returned

#### Scenario: Only accepted scores returned to a normal caller

- **WHEN** a normal caller runs any search (including an empty browse query)
- **THEN** only scores whose moderation status is `accepted` are returned, and `pending`
  or `rejected` scores are absent regardless of the query or filters

### Requirement: Fetch catalog score bytes for playback

The backend SHALL expose an authenticated operation that returns the canonical
(decoded) bytes of a catalog score by its id, read from the object store under
the public-corpus prefix, so the app can open it in the player. A request for a
non-existent catalog id MUST be rejected with a typed not-found error.
Unauthenticated requests MUST be rejected. For a normal (non-admin, non-moderator)
caller, a score whose moderation status is not `accepted` SHALL be treated as not
found: its bytes MUST NOT be served, so an unvalidated score cannot be opened by id.
A caller authorized as a moderator or admin SHALL be able to fetch the bytes of a
score in **any** moderation status (`pending` / `accepted` / `rejected`), so a reviewer
can open an unvalidated score to evaluate it. Until a dedicated moderator role exists,
"authorized" means an `admin` identity.

#### Scenario: Bytes returned for a known catalog score

- **WHEN** an authenticated caller requests the bytes of an existing `accepted` catalog id
- **THEN** the canonical score bytes are returned

#### Scenario: Unknown catalog id rejected

- **WHEN** a caller requests the bytes of a catalog id that does not exist
- **THEN** a typed not-found error is returned and no bytes are served

#### Scenario: Unauthenticated bytes request rejected

- **WHEN** a bytes request arrives without a valid authenticated identity
- **THEN** the request is rejected

#### Scenario: Bytes of an unvalidated score refused to a normal caller

- **WHEN** a normal caller requests the bytes of a `pending` or `rejected` catalog id
- **THEN** the request is refused as not found and no bytes are served

#### Scenario: Authorized reviewer fetches an unvalidated score's bytes

- **WHEN** a moderator/admin caller requests the bytes of a `pending` or `rejected` catalog id
- **THEN** the canonical score bytes are returned so the reviewer can evaluate it

## ADDED Requirements

### Requirement: Privileged moderation-status filter

The search operation SHALL accept an optional moderation-status filter that selects
scores by their moderation status (`pending` / `accepted` / `rejected`). This filter
is **privileged and back-office-only**: the server SHALL honor it only for a caller
authorized as a moderator or admin, and SHALL reject the request with a
permission-denied error when an unauthorized caller supplies it — the query MUST NOT
run and no results (of any status) are returned. When the filter is absent, the
operation behaves as for a normal caller (accepted-only visibility). The Flutter app
SHALL NOT send this filter; it exists for the moderation back office. Until a
dedicated moderator role exists, "authorized" means an `admin` identity.

#### Scenario: Authorized caller filters by status

- **WHEN** a moderator/admin caller searches with the moderation-status filter set to `pending`
- **THEN** only `pending` scores are returned, composing with the text query and other filters

#### Scenario: Unauthorized caller supplying the filter is rejected

- **WHEN** a normal caller supplies the moderation-status filter
- **THEN** the request is rejected with a permission-denied error and no results are returned

#### Scenario: Absent filter keeps accepted-only visibility

- **WHEN** any caller searches without supplying the moderation-status filter
- **THEN** a normal caller sees only `accepted` scores, unchanged from the default behavior

#### Scenario: Authorized caller can still request accepted only

- **WHEN** a moderator/admin caller sets the moderation-status filter to `accepted`
- **THEN** only `accepted` scores are returned
