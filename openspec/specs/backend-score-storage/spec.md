# backend-score-storage Specification

## Purpose
TBD - created by archiving change add-user-score-upload. Update Purpose after archive.
## Requirements
### Requirement: Authenticated score upload with server-side validation

The backend SHALL expose an authenticated operation that accepts an uploaded
score (its bytes, original filename, chosen difficulty, and rights attestation —
declared basis + confirmation) from the resolved caller. It MUST re-validate the received bytes
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
maximum number of contributed scores created within the configured window. The
maximum and the window length in days SHALL be **resolved per request from runtime
configuration keyed by the caller's effective plan**
(`plans.scores.upload_quota.free` = today's default 5 / 7 days,
`plans.scores.upload_quota.premium` for a plan whose unlock set includes
`scores.extended_quotas`), so free keeps its current value and premium raises it
without a release. Uploading and proposing to the catalog SHALL remain available to
every plan. The quota MUST be enforced server-side before any storage or database
write, scoped to the caller's own contributions; the typed refusal SHALL tell the
app that a higher plan raises the limit so the surface can upsell.

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

#### Scenario: Premium quota applies to a plan holder

- **WHEN** a caller whose plan grants `scores.extended_quotas` uploads beyond the free
  maximum but within the premium maximum for the window
- **THEN** the upload is allowed

#### Scenario: Free plan keeps uploading and proposing

- **WHEN** a free user within quota uploads a score and proposes it to the catalog
- **THEN** both succeed exactly as before

### Requirement: Durable storage with owner-attributed record

On a valid upload the backend SHALL store the canonical (decoded) score bytes in
the object store under a per-user key, and SHALL persist a database record
attributing the score to the caller with: the owner's user id, the object key,
the chosen difficulty (Beginner / Intermediate / Advanced), the rights attestation
(the declared basis — author, public domain / free licence, or strictly personal
use (`private_use`) — and its confirmation), and a creation timestamp. The
database record MUST be the source of truth for ownership. A rights attestation
whose confirmation is not affirmative, or whose basis is outside the accepted
set, MUST be rejected.

#### Scenario: Object and record created on success

- **WHEN** a valid upload is accepted
- **THEN** the canonical bytes are stored in the object store under a per-user key
  **AND** a record is written with owner id, object key, difficulty, the rights
  attestation (basis + confirmation), and creation timestamp

#### Scenario: Missing rights confirmation rejected

- **WHEN** an upload does not carry an affirmative rights confirmation
- **THEN** it is rejected and no record is created

#### Scenario: Invalid rights basis rejected

- **WHEN** an upload declares a rights basis outside the accepted set (author,
  public domain / free licence, or strictly personal use)
- **THEN** it is rejected and no record is created

#### Scenario: Personal-use upload accepted and persisted

- **WHEN** a valid upload declares the `private_use` basis with an affirmative
  confirmation
- **THEN** it is stored and recorded exactly like other bases, with
  `private_use` persisted as the score's rights basis

#### Scenario: Difficulty is constrained to the fixed set

- **WHEN** an upload specifies a difficulty outside Beginner / Intermediate /
  Advanced
- **THEN** it is rejected

### Requirement: Descriptive metadata is derived server-side, not client-supplied

The backend SHALL derive every descriptive and musical metadata field of an
uploaded score — at least its title, composer, key, time signature, and measure
count — **from the server-side parse of the received file**. A parsed value is
authoritative and MUST NOT be overridden by any client-sent value. The **only**
exception is a **fallback title/composer**: when the parsed file carries **no**
title (resp. composer), the backend MAY use a client-supplied fallback for that
field, since there is no parsed value to protect; a fallback MUST be ignored when
the file does carry the field. Musical metadata (key, time, measures, is-piano)
is never client-settable. The stored metadata therefore always reflects the file
when the file provides it, and a client can never alter or spoof a value the file
already contains.

#### Scenario: Metadata comes from the file, not the request

- **WHEN** an authenticated caller uploads a valid file
- **THEN** the stored record's title, composer, and musical metadata are taken
  from the server's parse of the uploaded bytes

#### Scenario: A client value never overrides a parsed one

- **WHEN** the parsed file carries a title (or composer) AND the request also
  carries a fallback for that field
- **THEN** the server keeps the parsed value and ignores the client fallback

#### Scenario: Fallback fills a field the file lacks

- **WHEN** the parsed file carries no title (or composer) AND the request supplies
  a fallback for that field
- **THEN** the server stores the (trimmed, bounded) fallback and re-derives the
  search keys from it

#### Scenario: Stored metadata matches the file content

- **WHEN** a score is stored
- **THEN** every field the file provides reflects the parsed content of the stored
  bytes, with only difficulty, the rights attestation, and any fallback for a
  file-absent title/composer originating from the caller

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

### Requirement: Per-plan private score library cap

The backend SHALL bound the number of scores a user keeps in their private library by a
maximum resolved per request from runtime configuration keyed by the caller's effective plan
(`plans.scores.library_max.free`, `.premium`), independent of the rolling upload quota. An
upload that would exceed the cap MUST be refused with a typed error before any storage write;
deleting a score frees a slot. Scores already accepted into the public catalog SHALL NOT count
against the cap.

#### Scenario: Cap reached on the free plan

- **WHEN** a free user at the library cap uploads another score
- **THEN** the upload is refused with a typed error naming the cap and the plan that raises it, and nothing is stored

#### Scenario: Accepted scores do not count

- **WHEN** a user's proposed score is accepted into the catalog
- **THEN** it no longer counts against their private library cap

### Requirement: Personal-use scores are excluded from any public proposal path

The backend SHALL reject, in every server operation that materialises, submits,
or re-submits a user score toward the public catalog (now or introduced later),
any request targeting a score whose **stored** rights basis is `private_use`.
The decision SHALL rely solely on the persisted basis — never on client-supplied
data — and the rejection SHALL leave no public-catalog side effect. This guard
SHALL hold regardless of the order in which proposal features and this change
are deployed.

#### Scenario: Proposal of a personal-use score rejected

- **WHEN** a caller attempts to propose their own score whose stored rights
  basis is `private_use`
- **THEN** the operation is rejected and no public-catalog row, review entry, or
  other side effect is created

#### Scenario: Client-claimed basis is ignored

- **WHEN** a proposal request claims an eligible basis but the stored record
  says `private_use`
- **THEN** the stored basis wins and the operation is rejected

