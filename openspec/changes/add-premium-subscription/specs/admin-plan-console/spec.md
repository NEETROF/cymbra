## ADDED Requirements

### Requirement: Music-admin-only plan console

The back office SHALL provide a `Plans` view and its gRPC surface, accessible only to a
music-scope admin (`require_admin_in_scope("music")`). A moderator or another scope's admin
MUST be rejected and MUST NOT see who holds which plan. All mutations SHALL be audited with the
acting admin, the target, the action and a free-text reason.

#### Scenario: Moderator is rejected

- **WHEN** a music moderator opens the plans view or calls a plan admin RPC
- **THEN** the request is rejected and no plan data is returned

#### Scenario: Mutations are audited

- **WHEN** an admin grants, revokes, creates a campaign or mints codes
- **THEN** an audit entry records who, what, whom and why

### Requirement: Entitlement lookup by handle

The console SHALL let an admin look up an account by handle and list its entitlement rows (plan,
source, start, end, status, campaign) with provider references shown only behind an explicit
copy action. It SHALL show the resulting effective plan as the server computes it.

#### Scenario: Lookup shows sources and effective plan

- **WHEN** an admin looks up a handle with a beta code row and an Apple row
- **THEN** both rows are listed and the effective plan reads `premium` via `apple`

### Requirement: Nominative grants and revocations

The console SHALL let an admin grant `beta` or `premium` to a handle with an end date and a
reason (source `admin`), and revoke any `admin` or `code` row. An open-ended grant (`ends_at` null)
SHALL require an explicit confirmation and be visibly flagged in the listing. Store and web rows
MUST NOT be revocable from the console (they end on the provider's side).

#### Scenario: Time-bounded grant

- **WHEN** an admin grants premium to a handle until a date with a reason
- **THEN** a `premium` row `source = admin` with that end date is created and audited

#### Scenario: Open-ended grant is flagged

- **WHEN** an admin confirms a grant without an end date
- **THEN** the row is created and marked as open-ended in every listing

#### Scenario: Store rows are read-only

- **WHEN** an admin attempts to revoke an `apple`, `google` or `web` row
- **THEN** the action is unavailable and the row is unchanged

### Requirement: Campaign and code management, cohort export

The console SHALL let an admin create campaigns, move their end date, open/close them, mint N
codes (clear text shown once, downloadable once), revoke codes individually or per campaign, and
list redemptions per campaign. It SHALL export the redemptions of a campaign (handles and dates)
so the beta cohort can be contacted or targeted with a store offer.

#### Scenario: Mint and show once

- **WHEN** an admin mints 20 codes for a campaign
- **THEN** the 20 clear-text codes are shown once and are not retrievable afterwards

#### Scenario: Move the campaign end

- **WHEN** an admin moves a campaign's end date
- **THEN** the listing shows every produced entitlement now ending at the new date

#### Scenario: Cohort export

- **WHEN** an admin exports a campaign's redemptions
- **THEN** a file with the redeeming handles and dates is produced
