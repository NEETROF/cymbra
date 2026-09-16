## MODIFIED Requirements

### Requirement: Music-admin-only plan console

The back office SHALL expose its plan surfaces — the **plan sections of the account detail
page** (`/admin/users/{user_id}`) and the **Campaigns page** (`/music/campaigns`) — and their gRPC
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

### Requirement: Campaigns are managed on their own page

The back office SHALL host campaign administration on a page of its own — **`/music/campaigns`,
labelled "Campaigns" in the Music section** — holding the campaign lifecycle, code minting and revocation, and the
per-campaign member list and export, and **nothing about an individual account's
subscription**: the account lookup that used to open this page is removed, because the same
work now starts from the users directory and ends on an account's detail page. Keeping an
account-lookup field here would leave two doors to the same room and let the label
"Subscriptions" cover a page that no longer manages any. The former `/campaigns` and `/plans`
paths SHALL redirect to `/music/campaigns` so existing links and bookmarks keep working.

A campaign's member rows SHALL link to each member's account detail page, so an operator
reading a cohort can open any of its members without re-identifying them.

#### Scenario: The campaigns page holds no account lookup

- **WHEN** a music admin opens `/music/campaigns`
- **THEN** the page shows campaigns, codes and the selected campaign's members, and offers no account search field

#### Scenario: The old path still resolves

- **WHEN** an admin opens `/campaigns` or `/plans` (a bookmark, an old link)
- **THEN** the console lands on `/music/campaigns` showing the campaign administration

#### Scenario: From a member to their account

- **WHEN** an admin activates a member row in a campaign's member list
- **THEN** that member's account detail page opens

#### Scenario: Campaign administration is unchanged

- **WHEN** an admin creates a campaign, mints codes, closes or reopens a campaign or its enrolment, or exports its members from `/music/campaigns`
- **THEN** each behaves exactly as specified for the campaign console, including the pause-not-end semantics of closing
