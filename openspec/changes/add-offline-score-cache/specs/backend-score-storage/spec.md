## ADDED Requirements

### Requirement: Expose a score's content hash as an ETag

The backend SHALL expose the stored content hash of a score's canonical bytes to
the client as an opaque ETag — on the bytes response and/or the score's metadata,
for both contributed uploads and catalog scores. The hash SHALL be stable for a
given score id: identical bytes always yield the same ETag, so a client can tell
whether the bytes behind an id it already holds are unchanged. The hash MUST NOT
require any new server-side computation per request when it is already persisted
(reuse the stored `sha256`).

#### Scenario: Bytes response carries the content hash

- **WHEN** an authenticated caller fetches a score's bytes
- **THEN** the response includes the content hash (ETag) of those bytes

#### Scenario: Same id yields a stable hash

- **WHEN** the same score id is fetched twice with unchanged stored bytes
- **THEN** the same content hash is returned both times

### Requirement: Conditional bytes fetch by content hash

The backend SHALL support a conditional bytes fetch: a caller MAY supply a
content hash it already holds, and when that hash still matches the stored bytes
the response MUST omit the bytes and signal "unchanged" instead of re-sending the
payload. When the supplied hash does not match (or none is supplied), the full
bytes and the current hash MUST be returned. This applies to both contributed and
catalog bytes fetches and MUST remain authenticated and access-scoped exactly as
the unconditional fetch.

#### Scenario: Unchanged returns no bytes

- **WHEN** a caller fetches bytes and supplies a content hash equal to the stored
  hash
- **THEN** the response signals "unchanged" and does not include the bytes

#### Scenario: Changed or absent hash returns full bytes

- **WHEN** a caller supplies a non-matching hash, or supplies none
- **THEN** the response returns the full bytes and the current content hash

#### Scenario: Conditional fetch keeps existing access rules

- **WHEN** a conditional fetch is made for a score the caller may not access
- **THEN** it is rejected exactly as the unconditional fetch would be, revealing
  nothing via the hash
