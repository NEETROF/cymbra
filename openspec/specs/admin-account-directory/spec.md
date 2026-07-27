# admin-account-directory Specification

## Purpose
TBD - created by archiving change add-admin-account-directory. Update Purpose after archive.
## Requirements
### Requirement: Paginated account directory

The user service SHALL expose an admin-only `ListAccounts` operation that returns a
page of accounts, each with its `id`, `handle`, `display_name`, and its roles in the
`music` scope, together with the total number of matching accounts. The operation
SHALL accept a `limit`, an `offset`, and an optional `query`. The response MUST NOT
include credentials, tokens, or the accounts' emails/identities.

#### Scenario: Lists a page of accounts with their roles

- **WHEN** an admin calls `ListAccounts` with `limit=25, offset=0` and no query
- **THEN** the service returns up to 25 accounts — each with `id`, `handle`, `display_name`, and its `music`-scope roles — plus the total account count

#### Scenario: Paginates through the directory

- **WHEN** an admin calls `ListAccounts` with `offset` advanced by `limit`
- **THEN** the service returns the next page and the same total, with no account repeated across adjacent pages

#### Scenario: Non-admin is refused

- **WHEN** a caller without the `admin` role (moderator or normal user) calls `ListAccounts`
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

### Requirement: Manage roles per account from the directory

The back-office Roles page SHALL present the account directory as a paginated table
and let an admin grant or revoke the `moderator` and `admin` roles **per account
row**, reusing the existing grant/revoke operations, without entering a raw account
id. After a successful change the row SHALL reflect the account's updated roles.

#### Scenario: Grant a role from a row

- **WHEN** an admin activates "grant moderator" on an account row
- **THEN** the role is granted for that account in the `music` scope and the row then shows the `moderator` role

#### Scenario: Revoke a role from a row

- **WHEN** an admin activates "revoke admin" on an account row that has it
- **THEN** the role is revoked and the row no longer shows the `admin` role

#### Scenario: Filter, paginate, and empty state

- **WHEN** an admin types a handle or email into the filter, or moves between pages
- **THEN** the table shows the matching page, and an empty result shows a localized "no accounts" message — never a raw gRPC status code or technical error string

#### Scenario: Only admins reach the directory

- **WHEN** a signed-in moderator (non-admin) reaches the console
- **THEN** the Roles page — and therefore the directory — is not available to them (route- and server-guarded)

