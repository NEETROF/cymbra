## ADDED Requirements

### Requirement: Directory exposes and filters by the apps an account uses

`ListAccounts` SHALL return, for each account, the apps it has signed in to, each with the
time of its last use. It SHALL accept an optional `apps` set that restricts the page
to accounts that have signed in to **every** app in the set, combined with the `query`,
`ids` and `exclude_ids` criteria, with the total reflecting the restriction. An app value
outside the recognized app vocabulary SHALL be refused with `INVALID_ARGUMENT`. The apps
are returned to every caller allowed to list the directory, independently of the scopes
that caller administers.

#### Scenario: Rows carry their apps

- **WHEN** an admin lists the directory and account A has signed in to Music and Lingua while account B has signed in to neither
- **THEN** A's row lists `music` and `lingua` with their last-use times, and B's row lists no app

#### Scenario: Filter by one app

- **WHEN** an admin calls `ListAccounts` with `apps = {lingua}`
- **THEN** only accounts that have signed in to Lingua are returned, and the total counts only them

#### Scenario: Filter by both apps

- **WHEN** an admin calls `ListAccounts` with `apps = {music, lingua}`
- **THEN** only accounts that have signed in to both Music and Lingua are returned

#### Scenario: Unknown app is refused

- **WHEN** an admin calls `ListAccounts` with `apps = {chess}`
- **THEN** the service returns `INVALID_ARGUMENT`

### Requirement: Users page shows app icons and an app filter

The Users page SHALL show, on each row, one icon per app the account uses — Music and
Lingua — each with a localized accessible name and the last use in its tooltip, and
a dash when the account uses neither. The page SHALL offer an **App** filter with the
choices any, Music, Lingua, and Music and Lingua, applied together with the other search
criteria and resetting to the first page.

#### Scenario: Icons on the row

- **WHEN** an admin opens the Users page and an account has signed in to Music and Lingua
- **THEN** its row shows the Music icon and the Lingua icon, each announced by its app name

#### Scenario: Filtering by app

- **WHEN** an admin chooses "Lingua" in the App filter
- **THEN** the table lists only accounts that use Lingua, from the first page

#### Scenario: Account with no app

- **WHEN** an account has never signed in to an app
- **THEN** its app cell shows a dash
