# license-filtering Specification

## Purpose
TBD - created by archiving change add-score-crawler. Update Purpose after archive.
## Requirements
### Requirement: Redistributable licence whitelist

The system SHALL keep a score only when its licence normalises to one of the
explicitly redistributable codes: `CC0`, a confirmed public-domain status
(`PublicDomain` — Public Domain Mark or a public-domain status asserted by the
source), `CC-BY` (any version), or `CC-BY-SA` (any version). Every other
outcome SHALL be rejected. The whitelist SHALL be the single authority consulted
before any heavy content download.

#### Scenario: Whitelisted licence is accepted
- **WHEN** an item's licence normalises to `CC0`, `PublicDomain`, a `CC-BY-<v>`,
  or a `CC-BY-SA-<v>` code
- **THEN** the item is eligible to be fetched, converted, and written to the
  corpus

#### Scenario: Non-whitelisted licence is rejected
- **WHEN** an item's licence normalises to `AllRightsReserved`, any code
  carrying an `NC` (non-commercial) or `ND` (no-derivatives) clause, or any code
  outside the whitelist
- **THEN** the item is rejected, never fetched into the safe corpus, and a
  rejection record is journalled with the reason

### Requirement: Licence normalisation

The system SHALL normalise raw licence signals (free-text labels, CC URLs, SPDX
identifiers, source-specific status strings) into a canonical code with an
explicit version where applicable (e.g. `CC-BY-4.0`, `CC-BY-SA-3.0`, `CC0-1.0`,
`PublicDomain`) together with the canonical licence URL. Normalisation SHALL be a
pure function with no I/O so it can be exhaustively unit-tested from fixtures.

#### Scenario: Creative Commons URL normalised
- **WHEN** a raw signal is a Creative Commons deed URL such as
  `https://creativecommons.org/licenses/by-sa/4.0/`
- **THEN** it normalises to code `CC-BY-SA-4.0` with the canonical licence URL

#### Scenario: Version-less CC label normalised
- **WHEN** a raw signal is a version-less label such as `CC BY`
- **THEN** it normalises to a `CC-BY` code that the whitelist accepts as "any
  version"

#### Scenario: Unknown or ambiguous signal is not guessed
- **WHEN** a raw signal is empty, unrecognised, or contradictory (multiple
  conflicting licences on one item)
- **THEN** normalisation yields an `Unknown`/`Ambiguous` outcome that the gate
  rejects, rather than defaulting to any redistributable code

### Requirement: License-first gate ordering

The system SHALL determine and evaluate the licence of an item BEFORE
downloading its heavy content (score payload). When the licence is not on the
whitelist, the item SHALL be skipped immediately with no heavy fetch and no
conversion.

#### Scenario: Heavy fetch skipped for rejected licence
- **WHEN** licence evaluation for an item returns a non-whitelisted result
- **THEN** the orchestrator does not download the score payload and does not
  invoke any converter for that item

### Requirement: Low-confidence classification

The system SHALL classify user-declared or unverified public-domain material as
`unverified` confidence. Sources whose public-domain status is self-declared by
uploaders without independent verification (e.g. musetrainer, the non-vetted
subset of PDMX) SHALL be marked `unverified`. All other whitelisted outcomes
SHALL be `verified`. Confidence SHALL never upgrade a rejected licence into an
accepted one.

#### Scenario: Self-declared PD marked unverified
- **WHEN** an item's only licence evidence is a public-domain status declared by
  a user of an upload-based source without independent confirmation
- **THEN** the item is classified `unverified` and routed to the low-confidence
  corpus, never the safe corpus

#### Scenario: Confirmed source PD marked verified
- **WHEN** an item carries a Public Domain Mark or a public-domain status
  asserted by an authoritative source
- **THEN** the item is classified `verified`

### Requirement: Rejection journalling

The system SHALL record every rejected item with its source, identifying URL,
the raw licence signal observed, and the normalised reason for rejection, so a
human can audit why material was excluded.

#### Scenario: Rejected item is auditable
- **WHEN** an item is rejected for licence reasons
- **THEN** a line is appended to `rejected.log` containing the source, the item
  URL, the raw licence signal, and the rejection reason

