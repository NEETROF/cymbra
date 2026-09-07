## ADDED Requirements

### Requirement: The directory can be filtered to sandbox accounts

The account directory SHALL accept a filter that restricts the listing to accounts
marked as sandbox accounts, so the mark can be reviewed without knowing in advance which
accounts carry it. The filter SHALL compose with the directory's existing query and
pagination, and the returned total SHALL count only matching accounts.

#### Scenario: Filtering returns only marked accounts

- **WHEN** an admin lists accounts with the sandbox-account filter set
- **THEN** every returned account is marked as accepting sandbox purchases, and the total counts only those

#### Scenario: The filter composes with the text query

- **WHEN** an admin combines the sandbox-account filter with a handle query
- **THEN** only accounts matching both are returned

#### Scenario: Without the filter the directory is unchanged

- **WHEN** an admin lists accounts without the filter
- **THEN** the directory returns marked and unmarked accounts alike, as before
