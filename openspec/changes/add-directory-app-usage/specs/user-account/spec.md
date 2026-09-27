## ADDED Requirements

### Requirement: Record the apps an account signs in to

The identity service SHALL record, per account, each **app** (`music`, `lingua`, `live`)
the account has obtained an access token for, with the time of the first and of the
latest such token. The record SHALL be written whenever an access token is minted for an
app audience — at sign-in and at every refresh — and SHALL NOT be written for the
`back-office` or `web` audiences. Writing the record MUST NOT block or fail the sign-in or
refresh that triggered it. The latest time MAY be coarsened to the hour. The record SHALL
be deleted with the account.

#### Scenario: First sign-in to an app

- **WHEN** an account signs in to Lingua for the first time
- **THEN** the account is recorded as a Lingua user, with its first and latest sign-in set to now

#### Scenario: A refresh keeps the latest sign-in current

- **WHEN** a Music client refreshes its session more than an hour after the recorded latest sign-in
- **THEN** the account's latest Music sign-in is updated and its first sign-in is unchanged

#### Scenario: The back office and the site are not apps

- **WHEN** an account signs in to the back office or to the site
- **THEN** no app is recorded for that sign-in

#### Scenario: A recording failure does not block sign-in

- **WHEN** the app record cannot be written during a sign-in
- **THEN** the sign-in still succeeds and returns its tokens

#### Scenario: Deleting the account deletes its apps

- **WHEN** an account is deleted
- **THEN** no app record remains for it
