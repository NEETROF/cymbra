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

### Requirement: The accounts directory shows and filters by plan and beta

The back-office accounts directory SHALL show, for each listed account, its effective plan
(`free` / `premium`, with `trial` marked when premium comes from a premium trial) and its
active beta memberships by campaign, and SHALL offer search criteria **plan** (any / free /
premium / premium trial) and **beta** (any / a specific campaign, listed by kind). The criteria
SHALL be resolved by the plan service into account ids (`ListAccountIdsByPlan`) which the
directory then lists (`ListAccounts` with `ids`), and the badges SHALL come from a batch lookup
(`GetPlansForAccounts`) for the displayed page — the identity service MUST NOT learn plan or
beta concepts. Both RPCs are music-admin only.

#### Scenario: Filter by premium trial

- **WHEN** an admin selects plan = "premium trial" in the accounts directory
- **THEN** only accounts with an active premium-trial row are listed, each showing the trial campaign and its end

#### Scenario: Filter by feature beta

- **WHEN** an admin selects beta = `midi-drums`
- **THEN** only active members of that campaign are listed, whatever their plan

#### Scenario: Badges on every row

- **WHEN** an admin lists any page of accounts
- **THEN** each row shows the effective plan and the beta memberships, fetched in one batch call for the page

#### Scenario: Moderator sees no plan data

- **WHEN** a music moderator opens the accounts directory
- **THEN** the plan/beta columns and filters are absent and the batch lookup is not called

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
