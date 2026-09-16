## ADDED Requirements

### Requirement: Navigation grouped by product

The back-office sidebar SHALL present its entries in three headed sections, in this order: **Music** (Review queue, Catalog, Private scores, Instrument sounds, Campaigns, Usage), **Lingua** (Overview) and **Administration** (Users, Feature flags, Notifications, Jobs). Each section SHALL be exposed as a labelled group whose heading names it, and a section with no entry the operator can open MUST NOT be shown, heading included. Section headings and entry labels SHALL be localized in every supported locale.

#### Scenario: A global admin sees the three sections

- **WHEN** an account holding `admin` in the `global` scope opens the console
- **THEN** the sidebar shows the Music, Lingua and Administration sections in that order, each under its heading, with every entry listed above

#### Scenario: A music moderator sees only what they moderate

- **WHEN** an account holding only `moderator` in the `music` scope opens the console
- **THEN** the sidebar shows the Music section with Review queue and Catalog only, and no Lingua or Administration heading

#### Scenario: Sections are announced as groups

- **WHEN** a screen-reader user moves into the sidebar
- **THEN** each section is announced as a group named by its heading

### Requirement: An entry is shown exactly when its page opens

Every back-office page except the public ones (sign-in, access denied) SHALL declare one access rule — a role (`moderator` or `admin`) and optionally a scope — and both the route guard and the sidebar MUST evaluate that same rule, so an entry is shown if and only if navigating to its page is admitted. A `moderator` rule admits a moderator or an admin of the scope; an `admin` rule admits an admin of the scope; a rule without a scope admits an admin of any scope; a role held in the `global` scope counts in every scope. The rules SHALL be: Review queue, Catalog, score review and score detail — `moderator` in `music`; Private scores, Instrument sounds, Campaigns and Usage — `admin` in `music`; Overview — `admin` in `lingua`; Users, account detail, Feature flags and Notifications — `admin` in any scope; Jobs — `admin` in `global`. A non-public page that declares no rule MUST be treated as not openable. These checks are a convenience only: every RPC stays independently gated server-side.

#### Scenario: A lingua-only admin is not offered Music administration

- **WHEN** an account holding `admin` only in the `lingua` scope opens the console
- **THEN** the sidebar shows no Instrument sounds, Campaigns or Usage entry, and navigating to `/music/soundfonts`, `/music/campaigns` or `/music/usage` redirects them away

#### Scenario: A music admin reaches every Music page

- **WHEN** an account holding `admin` only in the `music` scope opens the console
- **THEN** the Music section lists all six Music entries and each of them opens

#### Scenario: A page without a rule stays closed

- **WHEN** a non-public route is declared without an access rule
- **THEN** no operator can open it and the automated tests fail on the missing rule

### Requirement: Page paths are prefixed by their section

Every page path SHALL start with its section's prefix: `/music/` (`/music/queue`, `/music/catalog`, `/music/review`, `/music/score/{id}`, `/music/private-scores`, `/music/soundfonts`, `/music/campaigns`, `/music/usage`), `/lingua/` (`/lingua/overview`) or `/admin/` (`/admin/users`, `/admin/users/{user_id}`, `/admin/flags`, `/admin/notifications`, `/admin/jobs`). A section root (`/music`, `/lingua`, `/admin`) SHALL redirect to that section's first page, subject to the same access rule.

#### Scenario: Paths name the product

- **WHEN** a music admin opens the Instrument sounds entry
- **THEN** the address bar shows `/music/soundfonts`

#### Scenario: A section root opens its first page

- **WHEN** a global admin opens `/admin`
- **THEN** the console shows the Users page at `/admin/users`

### Requirement: Former paths keep resolving

Each path that a page had before this layout SHALL redirect to the page's current path, carrying its path parameters and its query string: `/takedowns` → `/music/private-scores`, `/soundfonts` → `/music/soundfonts`, `/campaigns` and `/plans` → `/music/campaigns`, `/usage` → `/music/usage`, `/lingua` → `/lingua/overview`, `/users` and `/roles` → `/admin/users`, `/users/{user_id}` → `/admin/users/{user_id}`, `/flags` → `/admin/flags`, `/notifications` → `/admin/notifications`, `/jobs` → `/admin/jobs`. The redirected page SHALL then apply its own access rule.

#### Scenario: A bookmark with a query still works

- **WHEN** an admin opens `/takedowns?owner=0190aa00-0000-7000-8000-000000000001`
- **THEN** the console lands on `/music/private-scores?owner=0190aa00-0000-7000-8000-000000000001`

#### Scenario: A former path does not bypass access

- **WHEN** a music moderator who is not an admin opens `/jobs`
- **THEN** the redirect to `/admin/jobs` is refused by its access rule and the moderator lands on their landing page

### Requirement: Operators land on a page they can open

The root path, any unknown path, a page the operator may not open, and the end of sign-in SHALL all lead to the operator's landing page: the first sidebar entry, in sidebar order, that the operator can open. An account signed in with no openable page SHALL be shown the access-denied state, whose localized message says the account has no page in this console and to ask an administrator — it MUST NOT claim the account is neither moderator nor admin, since a moderator of another scope reaches it too. The guard MUST NOT redirect in a loop.

#### Scenario: A music moderator lands on the review queue

- **WHEN** a music moderator opens `/`
- **THEN** the console shows `/music/queue`

#### Scenario: A lingua-only admin lands on Lingua

- **WHEN** an account holding `admin` only in the `lingua` scope signs in or opens `/`
- **THEN** the console shows `/lingua/overview`, not a Music page

#### Scenario: A moderator of another product has no page here

- **WHEN** an account holding only `moderator` in the `live` scope signs in
- **THEN** the console shows the access-denied state with the no-page message

#### Scenario: A refused page lands on the landing page

- **WHEN** a music admin without the `global` scope opens `/admin/jobs`
- **THEN** the console shows `/music/queue`
