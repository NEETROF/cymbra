# score-catalog-proposal — delta (add-private-score-catalog)

## MODIFIED Requirements

### Requirement: Opt-in proposal of a private score to the public catalog

The backend SHALL expose an authenticated, owner-scoped operation (`ProposeScore`) that
proposes one of the caller's private scores to the public catalog. The operation SHALL
require, captured at proposal time, a **licence declaration** and an explicit
**right-to-distribute attestation**; a proposal missing either MUST be refused and MUST
NOT create a catalog entry. The operation SHALL also refuse a score whose **stored**
rights basis is `private_use`, deciding on the persisted basis alone and never on the
proposal's licence declaration (see `backend-score-storage`). A valid proposal SHALL
create a new public-catalog entry
carrying the private score's server-derived metadata and a catalog-owned copy of its
bytes, entering the moderation lifecycle (see `score-moderation`). The caller MUST own
the referenced private score; proposing a score the caller does not own, or a
non-existent score, MUST be refused. The created catalog entry SHALL be independent of
the private score thereafter, so later deleting the private score does not remove the
catalog entry.

#### Scenario: Proposal requires licence and attestation

- **WHEN** a user proposes a private score without a licence declaration and
  right-to-distribute attestation
- **THEN** the proposal is refused and no catalog entry is created

#### Scenario: Valid proposal enters the catalog

- **WHEN** a user proposes a private score with a licence declaration and attestation
- **THEN** a public-catalog entry is created from that score, attributed to the proposer,
  and enters the moderation lifecycle

#### Scenario: Cannot propose a score you do not own

- **WHEN** a caller proposes a score id that is not in their private library (or does not
  exist)
- **THEN** the proposal is refused and no catalog entry is created

#### Scenario: Personal-use score cannot be proposed

- **WHEN** a caller proposes their own score whose stored rights basis is
  `private_use`, with a complete licence declaration and attestation
- **THEN** the proposal is refused and no catalog entry is created

#### Scenario: Catalog entry survives deletion of the private score

- **WHEN** a user deletes a private score that they previously proposed
- **THEN** the catalog entry (and its bytes) remain as the moderation record
