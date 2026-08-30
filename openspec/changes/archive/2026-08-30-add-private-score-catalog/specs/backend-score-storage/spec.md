# backend-score-storage — delta (add-private-score-catalog)

## MODIFIED Requirements

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

## ADDED Requirements

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
