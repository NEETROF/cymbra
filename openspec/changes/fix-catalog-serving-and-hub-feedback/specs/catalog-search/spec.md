## MODIFIED Requirements

### Requirement: Paginated, attribution-complete results

The search operation SHALL return results in bounded pages (a caller-supplied
limit clamped to a server maximum, plus an offset or cursor) and SHALL report
enough information to page through the full result set. Each returned entry SHALL
carry at least the catalog id, title, composer, difficulty level (when known),
and the licence/source attribution required to lawfully display the score
(licence code and source). Results SHALL NOT include raw score bytes.

Each page SHALL additionally report the **total number of entries matching the
query and filters**, independent of the page's limit/offset, so a caller can show
the full match count without loading every page. The total SHALL reflect the same
filter predicates as the returned entries.

#### Scenario: Results are bounded per page

- **WHEN** a caller requests results with a limit
- **THEN** at most that many entries (clamped to the server maximum) are returned in one page

#### Scenario: Paging reaches later results

- **WHEN** a caller requests the next page using the returned offset/cursor
- **THEN** the following entries are returned without duplicating or skipping items

#### Scenario: Page reports the full match total

- **WHEN** a query matches more entries than fit in one page
- **THEN** the response reports the total count of matching entries (independent of
  the page limit/offset), consistent with the applied filters

#### Scenario: Entry carries attribution

- **WHEN** a search result entry is returned
- **THEN** it includes the catalog id, title, composer, level (when known), and the
  licence code and source needed to attribute the score

#### Scenario: Search never returns bytes

- **WHEN** a search returns entries
- **THEN** no entry includes the score's raw MusicXML/.mxl bytes

### Requirement: Fetch catalog score bytes for playback

The backend SHALL expose an authenticated operation that returns the canonical
(decoded) bytes of a catalog score by its id, read from the object store under
the public-corpus prefix, so the app can open it in the player. A request for a
non-existent catalog id MUST be rejected with a typed not-found error.
Unauthenticated requests MUST be rejected.

When the catalog row exists but its bytes are **not yet present** in the object
store (e.g. a corpus not synced to the serving store yet), the operation MUST be
rejected with a typed **precondition-failed** error distinct from both not-found
and a generic internal error, so the app can tell the user the score is not
available yet rather than surfacing an opaque failure.

#### Scenario: Bytes returned for a known catalog score

- **WHEN** an authenticated caller requests the bytes of an existing catalog id
- **THEN** the canonical score bytes are returned

#### Scenario: Unknown catalog id rejected

- **WHEN** a caller requests the bytes of a catalog id that does not exist
- **THEN** a typed not-found error is returned and no bytes are served

#### Scenario: Known score with missing bytes is a precondition failure

- **WHEN** a caller requests the bytes of an existing catalog id whose object is
  not in the store yet
- **THEN** a typed precondition-failed error is returned (distinct from not-found
  and from a generic internal error), and no bytes are served

#### Scenario: Unauthenticated bytes request rejected

- **WHEN** a bytes request arrives without a valid authenticated identity
- **THEN** the request is rejected
