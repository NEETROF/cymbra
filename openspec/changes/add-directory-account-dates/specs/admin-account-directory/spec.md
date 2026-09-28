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
