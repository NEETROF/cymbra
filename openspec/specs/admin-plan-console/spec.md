# admin-plan-console Specification

## Purpose
TBD - created by archiving change add-premium-subscription. Update Purpose after archive.
## Requirements
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
trials), close enrolment, close a feature campaign, **reopen a closed campaign**, mint N codes
(clear text shown once, downloadable once), revoke codes individually or per campaign, and list
members per campaign with their join date and, for trials, their end date. It SHALL export a
campaign's members (handles and dates) so the group can be announced to, thanked, or targeted
with a store offer.

**Closing a campaign is a PAUSE, not an end.** It records that the campaign is closed and SHALL
NOT touch any membership: reopening therefore restores every member who was not individually
revoked. The per-person gesture already exists and is separate — a revoked membership never
returns, whatever the campaign does — so the global control stays global. The alternative,
deleting or stamping memberships at close time, would destroy the record of who took part,
which is precisely what the member list and the export exist to keep.

The console SHALL also let an admin **reopen enrolment**. Closing a campaign closes its
enrolment as a side effect (the two dates are written together), so without this second inverse a
reopened campaign would be live for its members and joinable by nobody — the operator could
restore a beta but never grow it again, which is half of what "reopen" is asked for.

The two remain **separate decisions with separate controls**: reopening the campaign SHALL NOT
reopen enrolment on its own, because a campaign may legitimately be live for its members and
closed to newcomers. Reopening a campaign whose enrolment is closed SHALL say so, rather than
leaving the admin to discover that redemptions are still refused.

Because reopening restores people, it SHALL say so before it acts: the console SHALL state how
many memberships the reopening will reactivate. Symmetrically, a closed campaign's member list
SHALL show its members as **inactive** rather than listing them as though nothing had happened —
an operator must not have to remember the campaign's state to read its members correctly.

Closing a `premium_trial` campaign SHALL NOT revoke the entitlements it already granted: a trial
given is a right acquired for its term, and it runs to its own end date. Closing therefore ends
a **feature** beta immediately but leaves a **trial**'s premium running, and reopening a trial
campaign restores no entitlement — it only makes the campaign live again. The console SHALL make
this difference visible where the two kinds are managed, rather than leaving an operator to
discover it by surprise.

#### Scenario: Mint and show once

- **WHEN** an admin mints 20 codes for a campaign
- **THEN** the 20 clear-text codes are shown once and are not retrievable afterwards

#### Scenario: Close a feature beta

- **WHEN** an admin closes the `midi-drums` campaign
- **THEN** its members are listed as ended and the campaign disappears from the flags console's beta selector

#### Scenario: Reopen it, and the members come back

- **WHEN** the admin reopens that campaign
- **THEN** every member who was not individually revoked is active again, with no flag edit and
  no re-enrolment, and the campaign reappears in the flags console's beta selector

#### Scenario: A revoked member does not come back

- **WHEN** a member's membership was revoked before the campaign was closed, and the campaign is
  reopened
- **THEN** that member stays out — the per-person revocation outlives the campaign's state

#### Scenario: Reopening announces who it restores

- **WHEN** the admin asks to reopen a campaign holding 12 unrevoked memberships
- **THEN** the console states that 12 memberships will be reactivated before the action is
  confirmed

#### Scenario: A closed campaign's members read as inactive

- **WHEN** an admin lists the members of a closed campaign
- **THEN** each row is shown as inactive, not as a live membership

#### Scenario: Reopening enrolment is a separate act

- **WHEN** a campaign whose enrolment was closed is reopened
- **THEN** its members are active again, new redemptions are still refused, and the console says
  that enrolment remains closed — until it is reopened in its own right

#### Scenario: Enrolment can be reopened

- **WHEN** an admin reopens the enrolment of a live campaign
- **THEN** a valid code for it is redeemable again

#### Scenario: A closed trial keeps its granted premium

- **WHEN** a `premium_trial` campaign is closed while a member's trial still has days to run
- **THEN** that member keeps premium until the trial's own end date, and reopening the campaign
  grants no new entitlement

#### Scenario: Member export

- **WHEN** an admin exports a campaign's members
- **THEN** a file with the member handles and dates is produced

