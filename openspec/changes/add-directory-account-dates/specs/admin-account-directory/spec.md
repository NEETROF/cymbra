## ADDED Requirements

### Requirement: Directory shows when an account signed up and last signed in

`ListAccounts` SHALL return, for each account, the time the account was created. The Users
page and the account page SHALL show, for each account, its sign-up date and its last
sign-in, the latter being the most recent last use among the apps the account has signed
in to. When the account has never signed in to an app, the last sign-in SHALL be shown as
a dash. Dates SHALL be formatted in the back office's current locale.

#### Scenario: Row carries the sign-up time

- **WHEN** an admin lists the directory
- **THEN** each row carries the creation time of its account

#### Scenario: Users table shows both dates

- **WHEN** an admin opens the Users page and an account created on 2026-03-02 last used Music on 2026-09-20 and Lingua on 2026-09-25
- **THEN** its row shows 2026-03-02 as the sign-up date and 2026-09-25 as the last sign-in

#### Scenario: Account never signed in to an app

- **WHEN** an account has no app record
- **THEN** its last sign-in is shown as a dash, and its sign-up date is still shown

#### Scenario: Account page shows both dates

- **WHEN** an admin opens an account's page
- **THEN** the page shows the account's sign-up date and last sign-in

### Requirement: Directory can be sorted by handle, name, sign-up and last sign-in

`ListAccounts` SHALL accept an optional sort key — handle, display name, sign-up time or
last sign-in — and a direction, and SHALL order the whole matching set by it before
paging, so every page follows the same order. The default SHALL be handle, ascending. An
account with no value for the key SHALL come after every account that has one, in both
directions; ties SHALL be broken by creation time then account id. An unknown sort key
SHALL be refused with `INVALID_ARGUMENT`. The Users page SHALL let the admin sort by
clicking the Handle, Name, Signed up and Last sign-in column headers: the active column
SHALL show its direction, clicking it again SHALL reverse it, a text column SHALL start
ascending and a date column newest first, and sorting SHALL return to the first page
while keeping the other search criteria.

#### Scenario: Sort by last sign-in, newest first

- **WHEN** an admin sorts the directory by last sign-in, descending, and account A last signed in on 2026-09-25, B on 2026-09-20, and C never
- **THEN** the order is A, B, C

#### Scenario: Missing values stay last when reversed

- **WHEN** the same directory is sorted by last sign-in, ascending
- **THEN** the order is B, A, C

#### Scenario: Unknown sort key is refused

- **WHEN** an admin calls `ListAccounts` with sort `email`
- **THEN** the service returns `INVALID_ARGUMENT`

#### Scenario: Clicking a header sorts and reverses

- **WHEN** an admin clicks the Signed up header, then clicks it again
- **THEN** the table is first ordered newest sign-up first from the first page, then oldest first, and the header announces the direction
