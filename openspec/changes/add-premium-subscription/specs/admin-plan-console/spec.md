## ADDED Requirements

### Requirement: Music-admin-only plan console

The back office SHALL provide a `Plans` view and its gRPC surface, accessible only to a
music-scope admin (`require_admin_in_scope("music")`). A moderator or another scope's admin
MUST be rejected and MUST NOT see who holds which plan or beta. All mutations SHALL be audited
with the acting admin, the target, the action and a free-text reason.

#### Scenario: Moderator is rejected

- **WHEN** a music moderator opens the plans view or calls a plan admin RPC
- **THEN** the request is rejected and no plan data is returned

#### Scenario: Mutations are audited

- **WHEN** an admin grants, revokes, enrols, creates a campaign or mints codes
- **THEN** an audit entry records who, what, whom and why

### Requirement: Entitlement and membership lookup by handle

The console SHALL let an admin look up an account by handle and list its entitlement rows (source,
start, end, status, campaign) with provider references shown only behind an explicit copy
action, and its beta memberships (campaign, kind, joined, end). It SHALL show the resulting
effective plan as the server computes it.

#### Scenario: Lookup shows sources, betas and effective plan

- **WHEN** an admin looks up a handle with a trial row, an Apple row and a feature-beta membership
- **THEN** both rows and the membership are listed and the effective plan reads `premium` with the later end

### Requirement: Nominative grants, enrolments and revocations

The console SHALL let an admin grant `premium` to a handle with an end date and a reason (source
`admin`), enrol a handle in a campaign of either kind (source `admin`, no code), revoke any
`admin` or `code` row, and revoke a membership. An open-ended grant (`ends_at` null) SHALL
require an explicit confirmation and be visibly flagged in the listing. Store and web rows MUST
NOT be revocable from the console (they end on the provider's side).

#### Scenario: Time-bounded grant

- **WHEN** an admin grants premium to a handle until a date with a reason
- **THEN** a `premium` row `source = admin` with that end date is created and audited

#### Scenario: Nominative enrolment

- **WHEN** an admin enrols a handle in a 90-day trial campaign
- **THEN** a membership and a premium row ending 90 days later are created with source `admin`

#### Scenario: Open-ended grant is flagged

- **WHEN** an admin confirms a grant without an end date
- **THEN** the row is created and marked as open-ended in every listing

#### Scenario: Store rows are read-only

- **WHEN** an admin attempts to revoke an `apple`, `google` or `web` row
- **THEN** the action is unavailable and the row is unchanged

### Requirement: Campaign and code management, member export

The console SHALL let an admin create campaigns (key, name, kind, and `duration_days` for
trials), close enrolment, close a feature campaign, mint N codes (clear text shown once,
downloadable once), revoke codes individually or per campaign, and list members per campaign
with their join date and, for trials, their end date. It SHALL export a campaign's members
(handles and dates) so the group can be announced to, thanked, or targeted with a store offer.

#### Scenario: Mint and show once

- **WHEN** an admin mints 20 codes for a campaign
- **THEN** the 20 clear-text codes are shown once and are not retrievable afterwards

#### Scenario: Close a feature beta

- **WHEN** an admin closes the `midi-drums` campaign
- **THEN** its members are listed as ended and the campaign disappears from the flags console's beta selector

#### Scenario: Member export

- **WHEN** an admin exports a campaign's members
- **THEN** a file with the member handles and dates is produced
