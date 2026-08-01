## MODIFIED Requirements

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

### Requirement: Manage roles per account from the directory

The back-office Roles page SHALL present the account directory as a paginated table and
let an admin grant or revoke the `moderator` and `admin` roles **per account row within a
chosen scope**, reusing the existing grant/revoke operations, without entering a raw
account id. The page SHALL let the admin choose the target scope, offering **only the
scopes the admin is authorized to administer** — the scopes in which the caller holds
`admin`, which for a `global/admin` is `global`, `music`, and `live` (a `global/admin` may
therefore grant or revoke a `global` role, including promoting another `global/admin`); an
admin authorized for a single scope SHALL never see the other scopes. Grant/revoke actions
apply to the currently selected scope. After a successful change the row SHALL reflect the
account's updated roles in that scope.

#### Scenario: Grant a role from a row in the selected scope

- **WHEN** an admin with the `live` scope selected activates "grant moderator" on an account row
- **THEN** the role is granted for that account in the `live` scope and the row then shows the `moderator` role for `live`

#### Scenario: Revoke a role from a row

- **WHEN** an admin activates "revoke admin" on an account row that has it in the selected scope
- **THEN** the role is revoked and the row no longer shows the `admin` role for that scope

#### Scenario: Single-scope admin sees only their scope

- **WHEN** a `music/admin` (without `global/admin`) opens the Roles page
- **THEN** only the `music` scope is available to view and manage, with no `live` scope selector, column, or data shown

#### Scenario: Multi-scope admin switches scope

- **WHEN** a `global/admin` switches the selected scope from `music` to `live`
- **THEN** the table reflects each account's `live`-scope roles and grant/revoke actions target the `live` scope

#### Scenario: Global admin grants a global role

- **WHEN** a `global/admin` selects the `global` scope and activates "grant admin" on an account row
- **THEN** the account is granted `global/admin` and the row shows the `admin` role for `global`

#### Scenario: Non-global admin has no access to the global scope

- **WHEN** a `music/admin` (without `global/admin`) opens the Roles page
- **THEN** the `global` scope is not offered as a selectable scope and no account's `global`-scope roles are shown

#### Scenario: Filter, paginate, and empty state

- **WHEN** an admin types a handle or email into the filter, or moves between pages
- **THEN** the table shows the matching page, and an empty result shows a localized "no accounts" message — never a raw gRPC status code or technical error string

#### Scenario: Only admins reach the directory

- **WHEN** a signed-in moderator (non-admin) reaches the console
- **THEN** the Roles page — and therefore the directory — is not available to them (route- and server-guarded)
