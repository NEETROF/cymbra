# admin-account-directory Specification

## Purpose
TBD - created by archiving change add-admin-account-directory. Update Purpose after archive.
## Requirements
### Requirement: Paginated account directory

The user service SHALL expose an admin-only `ListAccounts` operation that returns a
page of accounts, each with its `id`, `handle`, `display_name`, and its roles **grouped
by scope**, together with the total number of matching accounts. The roles returned for
each account SHALL be restricted to the scopes the **calling admin is authorized to
administer** — the scopes in which the caller holds `admin`, plus every scope when the
caller holds `global/admin` — and MUST NOT expose roles from scopes the caller may not
administer. The operation SHALL accept a `limit`, an `offset`, and an optional `query`.
The response MUST NOT include credentials, tokens, or the accounts' emails/identities.

#### Scenario: Lists a page of accounts with their per-scope roles

- **WHEN** a `global/admin` calls `ListAccounts` with `limit=25, offset=0` and no query
- **THEN** the service returns up to 25 accounts — each with `id`, `handle`, `display_name`, and its roles grouped by the `global`, `music`, and `live` scopes — plus the total account count

#### Scenario: Directory hides scopes the caller may not administer

- **WHEN** a `music/admin` (without `global/admin`) calls `ListAccounts`
- **THEN** every returned account exposes only its `music`-scope roles, and no account exposes any `live`-scope role

#### Scenario: Paginates through the directory

- **WHEN** an admin calls `ListAccounts` with `offset` advanced by `limit`
- **THEN** the service returns the next page and the same total, with no account repeated across adjacent pages

#### Scenario: Non-admin is refused

- **WHEN** a caller without the `admin` role in any scope (moderator or normal user) calls `ListAccounts`
- **THEN** the service returns `PERMISSION_DENIED` (or `UNAUTHENTICATED` when no session is present) and returns no accounts

### Requirement: Filter the directory by handle or email

`ListAccounts` SHALL accept an optional `query` that narrows the result to accounts
matching a **handle** (case-insensitively, via the normalized `handle_key`) or the
**email** of one of the account's `local` identities. An empty query returns all
accounts.

#### Scenario: Filter by handle

- **WHEN** an admin calls `ListAccounts` with `query="ada"`
- **THEN** only accounts whose normalized handle matches are returned, with the total reflecting the filter

#### Scenario: Filter by email

- **WHEN** an admin calls `ListAccounts` with `query="ada@cymbra.app"` and a local identity with that email exists
- **THEN** the owning account is returned

#### Scenario: No match returns an empty page

- **WHEN** the query matches no account
- **THEN** the service returns an empty account list and a total of 0 (not an error)

### Requirement: Filter the directory by an explicit set of account ids

`ListAccounts` SHALL accept an optional `ids` set that restricts the page to those accounts
(combined with `query` when both are given), so a product back office can pre-resolve a
product-specific criterion (a Music plan, a beta membership) into account ids **without the
identity service learning that criterion**. An `ids` set that matches nothing returns an empty
page and a total of 0. Roles exposure and admin authorization rules are unchanged.

#### Scenario: Directory page restricted to given ids

- **WHEN** an admin calls `ListAccounts` with `ids = {a, b, c}` and no query
- **THEN** only those accounts (that exist) are returned, with the total reflecting the restriction

#### Scenario: Ids combined with a handle query

- **WHEN** an admin calls `ListAccounts` with `ids = {a, b, c}` and `query = "ad"`
- **THEN** only accounts among `{a, b, c}` whose handle matches are returned

#### Scenario: Identity service stays product-agnostic

- **WHEN** the identity service's schema and RPCs are inspected
- **THEN** no plan, subscription or beta concept appears in them

### Requirement: Users directory page

The back-office **Users** page (`/users`) SHALL present the account directory as a
paginated table — handle, display name, roles and, for a music-scope admin, the effective
plan and the active beta memberships — with the existing search and filter criteria (free
text on handle/email, plan, beta) and pagination. The page is a surface for **finding** an
account, not for acting on one: every per-account action lives on the account detail page,
and **activating a row SHALL open `/users/{user_id}`** for that account. The former
`/roles` path SHALL redirect to `/users` so existing links and bookmarks keep working.

The roles column SHALL show the account's roles in **every scope the caller is authorized
to administer**, and SHALL name the scope on each role whenever more than one scope is on
offer — scoping is what these roles mean, and a bare `admin` that could be any of three
scopes tells the operator nothing. A caller authorized for a single scope sees that scope's
roles unqualified. Every role SHALL be shown with its localized name, never a raw
translation key.

#### Scenario: Opening an account from the directory

- **WHEN** an admin activates a row in the Users directory
- **THEN** the console navigates to `/users/{user_id}` for that account, without the admin ever typing or copying an account id

#### Scenario: The old path still resolves

- **WHEN** an admin opens `/roles` (a bookmark, an old link)
- **THEN** the console lands on `/users` showing the same directory

#### Scenario: Filter, paginate, and empty state

- **WHEN** an admin types a handle or email into the filter, or moves between pages
- **THEN** the table shows the matching page, and an empty result shows a localized "no accounts" message — never a raw gRPC status code or technical error string

#### Scenario: Single-scope admin sees only their scope

- **WHEN** a `music/admin` (without `global/admin`) opens the Users page
- **THEN** only `music` roles are shown, unqualified, and no other scope's roles appear anywhere on the page

#### Scenario: Multi-scope admin reads which scope a role comes from

- **WHEN** a `global/admin` lists an account holding `admin` in `music` and `user` in `global`
- **THEN** the row names the scope on each role, so the two are told apart without any selector to remember

#### Scenario: Only admins reach the directory

- **WHEN** a signed-in moderator (non-admin) reaches the console
- **THEN** the Users page — and therefore the directory — is not available to them (route- and server-guarded)

### Requirement: Account detail page

The back office SHALL provide an **account detail page at `/users/{user_id}`** that gathers
everything the console knows and can do about one account, so an admin never has to
re-identify the same person on a second screen. It SHALL show the account's identity header
(handle, display name) and, for a caller authorized to see each of them:

- the account's **roles per scope**, with grant/revoke of `moderator` and `admin` in **every
  scope the caller is authorized to administer** (not only one selected scope), applying the
  existing scope-matched authorization;
- the **role audit history** for that account;
- the read-only **curator reliability** indicator (moderator/admin only, informational — it
  never triggers a role change);
- **revocation of every session** of the account, behind an explicit confirmation.

A role change made on the page SHALL re-read **both** the account's roles and its audit
history: the change writes a row to the audit trail shown on that same screen, and leaving
the history a page-refresh behind the action the operator just took makes the trail read as
if the change had not happened. That re-read SHALL keep the page on screen — it MUST NOT
fall back to a loading state, which would unmount the page under the operator, discard
their scroll position and remount (and re-fetch) the subscription panel.

The page SHALL present the **subscription** and the **roles** as two sections the operator
switches between, rather than stacked one below the other: they are unrelated bodies of
work, each several tables deep, and stacking them makes either one a scroll away from the
other. The chosen section SHALL ride in the URL, so a reload or a shared link lands on the
section that was being read; an unknown or unavailable section falls back to the first
rather than showing nothing, and a caller with only one section available SHALL be shown
no switcher at all. The account's identity and the actions that belong to neither section
(reliability, session revocation) stay outside them.

The page SHALL be **addressable and self-sufficient**: opening the URL directly, reloading it,
or arriving from a link SHALL load the account (by its id) without requiring the directory
page to have been visited first. An unknown or malformed id SHALL show a localized
"account not found" state, never a raw error string.

#### Scenario: Direct URL loads the account

- **WHEN** an admin opens `/users/{user_id}` directly (bookmark, reload, link from elsewhere in the console)
- **THEN** the page loads that account's identity, roles and — for a music-scope admin — its plan, without any prior navigation

#### Scenario: Grant a role from the detail page

- **WHEN** an admin activates "grant moderator" for the `live` scope on the detail page
- **THEN** the role is granted for that account in the `live` scope, the page reflects it, and the change is audited

#### Scenario: The two sections are switchable and addressable

- **WHEN** an admin opens an account, switches to the roles section, and reloads the page
- **THEN** only one section is on screen at a time, and the reload lands on the roles section

#### Scenario: Only one section to show

- **WHEN** an admin without the `music` scope opens an account (no subscription to show)
- **THEN** the roles section is shown with no switcher

#### Scenario: The audit history follows the action

- **WHEN** an admin grants, then revokes, a role from the detail page
- **THEN** each change appears in that account's role history immediately, with no page refresh

#### Scenario: The page does not move under the operator

- **WHEN** an admin scrolls to the roles of a long account page and grants a role
- **THEN** the page stays where it was, keeps showing the account throughout, and the subscription is not re-fetched

#### Scenario: Multi-scope admin sees every scope at once

- **WHEN** a `global/admin` opens an account's detail page
- **THEN** the account's `global`, `music` and `live` roles are all shown and manageable on that one page

#### Scenario: Single-scope admin sees only their scope

- **WHEN** a `music/admin` (without `global/admin`) opens an account's detail page
- **THEN** only the `music` scope's roles are shown and manageable, and no other scope's roles appear

#### Scenario: Session revocation is confirmed

- **WHEN** an admin activates "revoke sessions" on the detail page
- **THEN** the action runs only after an explicit confirmation, and its outcome — success or failure — is surfaced as a localized message

#### Scenario: Unknown account

- **WHEN** an admin opens `/users/{id}` for an id that matches no account
- **THEN** the page shows a localized "account not found" state and offers a way back to the directory

