## ADDED Requirements

### Requirement: Authenticated score upload with server-side validation

The backend SHALL expose an authenticated operation that accepts an uploaded
score (its bytes, original filename, chosen difficulty, and authorship
acknowledgement) from the resolved caller. It MUST re-validate the received bytes
server-side — decoding `.mxl` when zipped, then parsing as MusicXML and confirming
it contains playable piano notes — using the same shared validation logic as the
client, and MUST reject anything invalid **before** any storage or database write.
Unauthenticated requests MUST be rejected.

#### Scenario: Unauthenticated upload rejected

- **WHEN** an upload request arrives without a valid authenticated identity
- **THEN** the request is rejected and nothing is stored

#### Scenario: Server re-validates and accepts a valid file

- **WHEN** an authenticated caller uploads bytes that decode, parse, and contain
  playable piano notes
- **THEN** the server accepts the upload and proceeds to store it

#### Scenario: Server rejects an invalid file before storing

- **WHEN** an authenticated caller uploads bytes that fail server-side validation
- **THEN** the request is rejected with a typed error and no object or database
  record is created

### Requirement: Reject oversized or malformed uploads

The backend SHALL enforce an upload size limit and MUST guard decoding against
resource-exhaustion inputs (e.g. bounded decompressed size and parse limits for
`.mxl`). Uploads exceeding the limit or tripping a decode bound MUST be rejected
before storage.

#### Scenario: Oversized upload rejected

- **WHEN** an upload exceeds the configured size limit
- **THEN** it is rejected before any storage or database write

#### Scenario: Decompression bound enforced

- **WHEN** a zipped upload would decompress beyond the configured bound
- **THEN** decoding is aborted and the upload is rejected

### Requirement: Enforce a per-user rolling upload quota

The backend SHALL limit how many scores a user may contribute within a rolling
time window: an upload MUST be rejected once the caller already has the configured
maximum number of contributed scores created within the configured window. Both
the maximum (default 5) and the window length in days (default 7) MUST be
configurable. The quota MUST be enforced server-side before any storage or
database write, scoped to the caller's own contributions.

#### Scenario: Upload allowed under the quota

- **WHEN** the caller has fewer than the configured maximum of contributed scores
  within the current window
- **THEN** the upload is allowed to proceed to validation and storage

#### Scenario: Upload rejected at the quota

- **WHEN** the caller has already reached the configured maximum of contributed
  scores created within the current window
- **THEN** the upload is rejected with a typed quota error and nothing is stored

#### Scenario: Quota window is rolling

- **WHEN** a caller's earlier uploads fall outside the configured window
- **THEN** those uploads no longer count against the quota and new uploads are
  allowed up to the maximum again

### Requirement: Durable storage with owner-attributed record

On a valid upload the backend SHALL store the canonical (decoded) score bytes in
the object store under a per-user key, and SHALL persist a database record
attributing the score to the caller with: the owner's user id, the object key,
the chosen difficulty (Beginner / Intermediate / Advanced), the authorship
acknowledgement, and a creation timestamp. The database record MUST be the source
of truth for ownership. An authorship acknowledgement that is not affirmative MUST
be rejected.

#### Scenario: Object and record created on success

- **WHEN** a valid upload is accepted
- **THEN** the canonical bytes are stored in the object store under a per-user key
  **AND** a record is written with owner id, object key, difficulty, authorship
  acknowledgement, and creation timestamp

#### Scenario: Missing authorship acknowledgement rejected

- **WHEN** an upload does not carry an affirmative authorship acknowledgement
- **THEN** it is rejected and no record is created

#### Scenario: Difficulty is constrained to the fixed set

- **WHEN** an upload specifies a difficulty outside Beginner / Intermediate /
  Advanced
- **THEN** it is rejected

### Requirement: Descriptive metadata is derived server-side, not client-supplied

The backend SHALL derive every descriptive and musical metadata field of an
uploaded score — at least its title, composer, key, time signature, and measure
count — **from the server-side parse of the received file**, and MUST NOT accept
or store any such metadata sent by the client. The only score attributes the
caller may set are the chosen difficulty and the authorship acknowledgement; the
stored metadata MUST therefore always correspond to the actual file content, so a
client cannot alter or spoof it.

#### Scenario: Metadata comes from the file, not the request

- **WHEN** an authenticated caller uploads a valid file
- **THEN** the stored record's title, composer, and musical metadata are taken
  from the server's parse of the uploaded bytes

#### Scenario: Client-supplied metadata is ignored

- **WHEN** an upload request carries descriptive metadata fields (e.g. a title or
  composer) alongside the file
- **THEN** the server ignores them and uses only values derived from the parsed
  file

#### Scenario: Stored metadata matches the file content

- **WHEN** a score is stored
- **THEN** its recorded metadata reflects the parsed content of the stored bytes,
  with only difficulty and the authorship acknowledgement originating from the
  caller

### Requirement: List the caller's own contributed scores

The backend SHALL return the contributed scores owned by the authenticated
caller, and MUST NOT expose scores owned by other users. Each returned entry MUST
carry at least its id, title (when known), difficulty, and creation date.

#### Scenario: Caller lists only their scores

- **WHEN** an authenticated caller lists their contributed scores
- **THEN** only records whose owner id matches the caller are returned

#### Scenario: Isolation across users

- **WHEN** two different users each have contributed scores
- **THEN** neither user's list contains the other's records

### Requirement: Owner-only deletion of a contributed score

The backend SHALL let the authenticated owner delete one of their contributed
scores, removing both the database record and the stored object. A caller MUST
NOT be able to delete a score they do not own. If the stored object cannot be
removed immediately, the deletion MUST still remove the record and the orphaned
object MUST be reclaimed by an idempotent cleanup, so no record ever references a
missing object.

#### Scenario: Owner deletes their score

- **WHEN** the owner requests deletion of their contributed score
- **THEN** the database record is removed and the stored object is deleted (or
  scheduled for idempotent deletion)

#### Scenario: Non-owner deletion rejected

- **WHEN** a caller requests deletion of a score owned by a different user
- **THEN** the request is rejected and nothing is removed

#### Scenario: No dangling record on partial failure

- **WHEN** object deletion fails after the record is removed
- **THEN** the record is still gone and the orphaned object is reclaimed by a
  retryable cleanup, leaving no record pointing at a missing object

### Requirement: Contributed scores erased with the account

When a user account is deleted, the backend SHALL erase that user's contributed
scores — both their database records and their stored objects — so no orphaned
data remains attributable to the removed account.

#### Scenario: Account deletion purges contributed scores

- **WHEN** a user account is deleted
- **THEN** that user's contributed-score records are removed and their stored
  objects are deleted (or scheduled for idempotent deletion)
