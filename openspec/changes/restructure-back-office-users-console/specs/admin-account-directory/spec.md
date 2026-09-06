## ADDED Requirements

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

## REMOVED Requirements

### Requirement: Manage roles per account from the directory

**Reason**: The directory row was carrying five actions and two expandable panels while the
same account's subscription lived on another screen entirely. Role management (and the rest of
the per-account gestures) moves to the addressable account detail page; the directory keeps
listing, filtering and navigation. Nothing is lost — the scope-matched authorization rules,
the per-scope visibility rules and the "admins only" gate are restated by the two
requirements added above.

**Migration**: Grant/revoke now happens on `/users/{user_id}` (see "Account detail page")
instead of on the directory row; `/roles` redirects to `/users`. No API change: the same
`GrantRole`/`RevokeRole` operations are called with the same scope argument.
