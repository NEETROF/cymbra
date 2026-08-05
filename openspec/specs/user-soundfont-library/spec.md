# user-soundfont-library Specification

## Purpose
TBD - created by archiving change add-soundfont-moderation. Update Purpose after archive.
## Requirements
### Requirement: Private per-user soundfont library

The system SHALL let an authenticated user import a `.sf2` soundfont into a **private,
per-user** library stored server-side, distinct from the public catalog. A private
library font SHALL be visible and deliverable only to its owner; it MUST NOT appear in
any other user's listing nor in the public catalog, and it SHALL NOT be subject to
moderation. The import path SHALL be open to any authenticated identity (not only
moderator/admin) and SHALL be distinguished from a back-office catalog upload by the
caller's token audience (app audience → private library).

#### Scenario: User imports a private font

- **WHEN** an authenticated user imports a `.sf2` from the app
- **THEN** it is stored in that user's private library and is immediately usable by them,
  without entering moderation or the public catalog

#### Scenario: Private font is owner-only

- **WHEN** a different user lists soundfonts or requests that font's bytes
- **THEN** the font is not returned — it is reachable only by its owner

#### Scenario: Back-office origin cannot write the private library

- **WHEN** a back-office-audience caller attempts the private-library import path
- **THEN** the request is refused; back-office uploads target the public catalog only

### Requirement: Private library is synced across the owner's devices

A user's private soundfont library SHALL be server-backed so it is available on every
device where that user is signed in, rather than local to a single device. Signing in on
another device SHALL surface the same library.

#### Scenario: Library follows the user to a new device

- **WHEN** a user who imported private fonts signs in on another device
- **THEN** their private library lists the same fonts, downloadable on the new device

### Requirement: Duplicate private import is idempotent

The system SHALL compute the exact-byte SHA-256 of an imported font and, within a user's
private library, treat a byte-identical re-import as the existing entry rather than
creating a second copy.

#### Scenario: Re-importing the same bytes returns the existing font

- **WHEN** a user imports a `.sf2` whose bytes already exist in their private library
- **THEN** the existing library entry is returned and no duplicate is stored

### Requirement: Opt-in proposal of a private font to the public catalog

A user SHALL be able to explicitly propose one of their private fonts to the public
catalog. Proposing SHALL require a **licence declaration** and an explicit
**right-to-distribute attestation** captured at proposal time; the general CGU authorship
attestation SHALL NOT by itself satisfy this, because a soundfont is third-party sample
data. A proposed font SHALL enter the public catalog as `pending` (subject to the
soundfont-moderation lifecycle), recorded with the proposer as `uploaded_by`. A proposal
whose content is byte-identical to a non-`rejected` catalog font MUST be refused as a
duplicate.

#### Scenario: Proposal requires licence and attestation

- **WHEN** a user proposes a private font without a licence declaration and right-to-distribute attestation
- **THEN** the proposal is refused and the font does not enter the public catalog

#### Scenario: Valid proposal enters the catalog as pending

- **WHEN** a user proposes a private font with a licence declaration and attestation
- **THEN** a public catalog entry is created `pending`, attributed to the proposer, awaiting review

#### Scenario: Proposal of already-cataloged content is refused

- **WHEN** a user proposes a font whose bytes match a non-`rejected` catalog entry
- **THEN** the proposal is refused as a duplicate and identifies the existing font

### Requirement: Per-user private library quota

The system SHALL bound each user's private library by a quota — a maximum font count and
a maximum total size — in addition to the existing per-file size cap. An import that
would exceed the quota MUST be refused with a typed error, and no bytes stored.

#### Scenario: Import within quota succeeds

- **WHEN** a user imports a font and their library is within the count and size quota
- **THEN** the import succeeds

#### Scenario: Import over quota is refused

- **WHEN** an import would push the user's library past its font-count or total-size quota
- **THEN** the import is refused with a typed error and nothing is stored

