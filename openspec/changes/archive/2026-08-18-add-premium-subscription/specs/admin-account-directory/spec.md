## ADDED Requirements

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
