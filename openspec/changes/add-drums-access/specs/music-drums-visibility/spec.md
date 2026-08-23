## ADDED Requirements

### Requirement: The drum audience is staff plus the beta campaign

The system SHALL expose the drum feature to admin and moderator identities, and to
identities holding an **active membership** of the `midi-drums` beta campaign, and
to no one else — including premium subscribers outside the campaign. This audience
SHALL be obtained by declaring one feature-flag key scoped to the `music` app,
defaulting to **off**, and giving it the existing `beta:midi-drums` rollout scope,
which the platform already resolves from the caller's identity and active
memberships. A second, parallel audience mechanism SHALL NOT be introduced.

#### Scenario: Staff reach the feature

- **WHEN** an admin or moderator identity is evaluated
- **THEN** the drum feature is in effect for them

#### Scenario: Campaign members reach the feature

- **WHEN** an identity holds an active `midi-drums` membership
- **THEN** the drum feature is in effect for them, whatever their plan

#### Scenario: Nobody else reaches it

- **WHEN** an identity is neither staff nor an active campaign member
- **THEN** the drum feature is not in effect for them, even on a premium plan

#### Scenario: Closing the campaign withdraws the feature

- **WHEN** the `midi-drums` campaign is closed
- **THEN** its former members lose the feature at the next evaluation, with no flag
  edit

### Requirement: The backend enforces the drum audience

The backend SHALL enforce the drum audience on every path that could disclose or
accept a percussion score — catalog search, catalog and user score bytes, the saved
and owned listings, upload, and the rating deck — so the restriction holds
regardless of the client. Hiding the feature in the app is defence in depth, not
the gate. Enforcement SHALL be resolved from the caller's own identity and
memberships, never from a value supplied in the request.

A caller without the feature SHALL receive the same answer as if the score did not
exist, rather than a distinguishable refusal, so the gate does not itself reveal
which scores are percussion.

#### Scenario: Search withholds percussion scores

- **WHEN** an ineligible caller searches the catalog
- **THEN** no percussion score appears in the results, whatever filter they supplied

#### Scenario: Direct fetch by id is refused

- **WHEN** an ineligible caller requests a percussion score's bytes by id
- **THEN** the backend withholds it, answering as it would for an unknown id

#### Scenario: An eligible caller is unaffected

- **WHEN** an eligible caller performs the same reads
- **THEN** percussion scores are returned normally

#### Scenario: Enforcement ignores client claims

- **WHEN** a request carries any client-supplied assertion of eligibility
- **THEN** it is disregarded and eligibility is resolved from the caller's identity
  and memberships

### Requirement: The drum gate fails closed

The drum gate SHALL deny on every uncertainty **about the caller**: an unreadable or
unreachable flag store, an unwired plan source, or memberships that cannot be
resolved SHALL each resolve to **not eligible**, never to a grant. A degraded
dependency must not silently widen the audience.

Uncertainty about a **score** is the opposite case and SHALL NOT withhold it: the
gate withholds a score whose recorded instrument **is** `percussion`, and a score
recorded as `unknown` SHALL stay as reachable as it is today. Withholding `unknown`
rows would hide the large majority of the existing corpus from the users who can
reach it now — a regression, not a safety gain.

This is only safe once every stored row carries a **derived** instrument. The
corpus already contains percussion scores — they were ingested despite the
playable-notes gate — so a row recorded as `unknown` **can** be percussion until it
has been re-derived. The instrument backfill SHALL therefore classify from the
stored bytes rather than translate the former staff-count flag, and the gate SHALL
NOT be relied upon as a boundary before that pass has completed.

#### Scenario: An unreadable flag store denies

- **WHEN** the flag store cannot be read
- **THEN** the drum feature is treated as off for every caller

#### Scenario: Unresolvable memberships deny

- **WHEN** a caller's beta memberships cannot be resolved
- **THEN** they are treated as holding none, so only staff retain the feature

#### Scenario: Unknown scores stay reachable

- **WHEN** an ineligible caller reads a score whose recorded instrument is `unknown`
- **THEN** it is served exactly as it is today, rather than withheld on suspicion

#### Scenario: Existing percussion rows are classified, not assumed absent

- **WHEN** the instrument is backfilled over a corpus that already contains
  percussion scores
- **THEN** each row's instrument is derived from its stored bytes, so a percussion
  row is recorded as `percussion` and not left as `unknown`

#### Scenario: Only recorded percussion is withheld

- **WHEN** the gate decides whether to withhold a score
- **THEN** it withholds on the instrument being `percussion`, never on the
  instrument being merely unrecorded
