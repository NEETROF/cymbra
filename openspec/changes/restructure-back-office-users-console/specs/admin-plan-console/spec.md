## MODIFIED Requirements

### Requirement: Music-admin-only plan console

The back office SHALL expose its plan surfaces — the **plan sections of the account detail
page** (`/users/{user_id}`) and the **Campaigns page** (`/campaigns`) — and their gRPC
surface only to a music-scope admin (`require_admin_in_scope("music")`). A moderator or
another scope's admin MUST be rejected and MUST NOT see who holds which plan or beta: on the
account detail page, an admin without the `music` scope SHALL see the account's roles,
history, reliability and sessions but **no plan block at all**, and the plan RPCs SHALL NOT be
called for them. All mutations SHALL be audited with the acting admin, the target, the action
and a free-text reason.

#### Scenario: Moderator is rejected

- **WHEN** a music moderator opens a plan surface or calls a plan admin RPC
- **THEN** the request is rejected and no plan data is returned

#### Scenario: Another scope's admin sees no plan block

- **WHEN** a `live`-only admin opens an account's detail page
- **THEN** the page shows roles, audit history, reliability and sessions, shows no plan, entitlement, membership or beta data, and issues no plan RPC

#### Scenario: Mutations are audited

- **WHEN** an admin grants, revokes, enrols, creates a campaign or mints codes
- **THEN** an audit entry records who, what, whom and why

### Requirement: Entitlement and membership lookup by handle

The console SHALL show, **on the account detail page of the account already being viewed**
(addressed by its `user_id` — no separate lookup field to retype a handle into), that
account's entitlement rows (source, start, end, status, campaign) with provider references
shown only behind an explicit copy action, and its beta memberships (campaign, kind, joined,
end). It SHALL show the resulting effective plan as the server computes it. The underlying
operation SHALL keep accepting **either** a handle or a user id, so an account can still be
addressed by handle by any other caller.

#### Scenario: Detail page shows sources, betas and effective plan

- **WHEN** an admin opens the detail page of an account with a trial row, an Apple row and a feature-beta membership
- **THEN** both rows and the membership are listed and the effective plan reads `premium` with the later end

#### Scenario: No handle to retype

- **WHEN** an admin reaches an account's detail page from the users directory
- **THEN** the account's plan data is already loaded for that account, without the admin entering a handle or an id anywhere

### Requirement: Nominative grants, enrolments and revocations

The console SHALL let an admin, **from the viewed account's detail page**, grant `premium`
with an end date and a reason (source `admin`), enrol the account in a campaign of either kind
(source `admin`, no code), revoke any `admin` or `code` row, and revoke a membership. An
open-ended grant (`ends_at` null) SHALL require an explicit confirmation and be visibly
flagged in the listing. Store and web rows MUST NOT be revocable from the console (they end on
the provider's side). After a successful mutation the page SHALL reflect the server's
recomputed effective plan.

**Revoking a membership SHALL NOT bar re-enrolment.** Only a *live* membership makes an
account already a member; a revoked one does not, and enrolling again SHALL revive that
account's single membership row for the campaign rather than adding a second. Without this,
revoking is a one-way door — the account is out of that campaign for good, recoverable only
by hand in the database. This does not touch the rule that reopening a **campaign** restores
nobody who was individually revoked: that is a bulk gesture which must not resurrect a
deliberate removal, whereas re-enrolling by name is explicit, per-person and audited.

The console SHALL NOT offer to enrol an account into a campaign it is already a live member
of: that campaign SHALL be listed with the reason rather than hidden, so an operator neither
hunts for a campaign that is on screen nor walks into a refusal the console could predict. A
refusal the console did not predict SHALL still read as a duplicate, never as a transient
failure inviting a retry that cannot succeed.

#### Scenario: Time-bounded grant

- **WHEN** an admin grants premium to the viewed account until a date with a reason
- **THEN** a `premium` row `source = admin` with that end date is created and audited

#### Scenario: Nominative enrolment

- **WHEN** an admin enrols the viewed account in a 90-day trial campaign
- **THEN** a membership and a premium row ending 90 days later are created with source `admin`

#### Scenario: Open-ended grant is flagged

- **WHEN** an admin confirms a grant without an end date
- **THEN** the row is created and marked as open-ended in every listing

#### Scenario: A revoked member can be enrolled again

- **WHEN** an admin enrols an account whose membership in that campaign was previously revoked
- **THEN** the enrolment succeeds, the account's single membership row for that campaign is live again, and the act is audited

#### Scenario: A live member is not offered the campaign

- **WHEN** an admin opens the enrolment dialog for an account that is already a live member of an open campaign
- **THEN** that campaign is shown as unavailable with the reason, and cannot be submitted

#### Scenario: A duplicate does not read as a transient failure

- **WHEN** an enrolment is refused because the account is already a member
- **THEN** the console says so, and does not invite the operator to try again

#### Scenario: Store rows are read-only

- **WHEN** an admin attempts to revoke an `apple`, `google` or `web` row
- **THEN** the action is unavailable and the row is unchanged

### Requirement: The accounts directory shows and filters by plan and beta

The back-office **Users** directory SHALL show, for each listed account, its effective plan
(`free` / `premium`, with `trial` marked when premium comes from a premium trial) and its
active beta memberships by campaign, and SHALL offer search criteria **plan** (any / free /
premium / premium trial) and **beta** (any / a specific campaign, listed by kind). The criteria
SHALL be resolved by the plan service into account ids (`ListAccountIdsByPlan`) which the
directory then lists (`ListAccounts` with `ids`), and the badges SHALL come from a batch lookup
(`GetPlansForAccounts`) for the displayed page — the identity service MUST NOT learn plan or
beta concepts. Both RPCs are music-admin only. A row's plan badge is a **summary**: the full
detail (entitlement rows, memberships, actions) is one click away on that account's detail
page.

#### Scenario: Filter by premium trial

- **WHEN** an admin selects plan = "premium trial" in the users directory
- **THEN** only accounts with an active premium-trial row are listed, each showing the trial campaign and its end

#### Scenario: Filter by feature beta

- **WHEN** an admin selects beta = `midi-drums`
- **THEN** only active members of that campaign are listed, whatever their plan

#### Scenario: Badges on every row

- **WHEN** an admin lists any page of accounts
- **THEN** each row shows the effective plan and the beta memberships, fetched in one batch call for the page

#### Scenario: From a filtered list to one account

- **WHEN** an admin filters the directory by a beta and activates one of the listed rows
- **THEN** that account's detail page opens with its entitlement rows and memberships already listed

#### Scenario: Moderator sees no plan data

- **WHEN** a music moderator opens the users directory
- **THEN** the plan/beta columns and filters are absent and the batch lookup is not called

## ADDED Requirements

### Requirement: Campaigns are managed on their own page

The back office SHALL host campaign administration on a page of its own — **`/campaigns`,
labelled "Campaigns"** — holding the campaign lifecycle, code minting and revocation, and the
per-campaign member list and export, and **nothing about an individual account's
subscription**: the account lookup that used to open this page is removed, because the same
work now starts from the users directory and ends on an account's detail page. Keeping an
account-lookup field here would leave two doors to the same room and let the label
"Subscriptions" cover a page that no longer manages any. The former `/plans` path SHALL
redirect to `/campaigns` so existing links and bookmarks keep working.

A campaign's member rows SHALL link to each member's account detail page, so an operator
reading a cohort can open any of its members without re-identifying them.

#### Scenario: The campaigns page holds no account lookup

- **WHEN** a music admin opens `/campaigns`
- **THEN** the page shows campaigns, codes and the selected campaign's members, and offers no account search field

#### Scenario: The old path still resolves

- **WHEN** an admin opens `/plans` (a bookmark, an old link)
- **THEN** the console lands on `/campaigns` showing the campaign administration

#### Scenario: From a member to their account

- **WHEN** an admin activates a member row in a campaign's member list
- **THEN** that member's account detail page opens

#### Scenario: Campaign administration is unchanged

- **WHEN** an admin creates a campaign, mints codes, closes or reopens a campaign or its enrolment, or exports its members from `/campaigns`
- **THEN** each behaves exactly as specified for the campaign console, including the pause-not-end semantics of closing
