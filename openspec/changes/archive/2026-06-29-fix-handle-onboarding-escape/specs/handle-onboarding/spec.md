## ADDED Requirements

### Requirement: The handle gate is always escapable

The post-authentication handle-selection screen SHALL always offer a way to leave
without choosing a handle. The user MUST be able to sign out and return to the entry
screen from this screen, regardless of handle availability.

#### Scenario: User leaves when every desired handle is taken

- **WHEN** a user on the handle screen cannot proceed (their desired handle is taken)
- **THEN** an escape action ("Use a different account" / "Sign out") is available that signs them out and returns them to the entry screen

#### Scenario: Escape clears the session

- **WHEN** the user triggers the escape action
- **THEN** the app performs a best-effort `Logout`, clears the stored tokens, and the next launch shows the entry screen (no valid session remains)

### Requirement: Abandoning a brand-new account cleans it up

The app SHALL delete a brand-new account — one with no handle yet, provisioned during
this sign-in — via `DeleteAccount` when the user abandons onboarding, so it does not
persist as an orphan. An account that already has a handle (an existing user) SHALL
only be signed out, never deleted, on escape.

#### Scenario: Abandon deletes the just-created account

- **WHEN** a user who just signed in (account has a null/empty handle) abandons onboarding via the escape action
- **THEN** the app deletes that account and returns to the entry screen, leaving no handle-less account behind

#### Scenario: Existing user is signed out, not deleted

- **WHEN** the escape action is used by an account that already has a handle
- **THEN** the app signs out only and does not delete the account

### Requirement: Handle-less accounts do not accumulate

The backend SHALL provide a maintenance process that removes accounts which have no
handle and were created longer ago than a configurable grace period, so that orphaned
accounts from hard app kills (where client cleanup cannot run) are eventually purged.

#### Scenario: Reaper purges an old handle-less account

- **WHEN** the maintenance process runs and finds an account with a null handle older than the grace period
- **THEN** that account is deleted

#### Scenario: Reaper preserves recent and handled accounts

- **WHEN** an account either has a handle or was created within the grace period
- **THEN** the maintenance process leaves it untouched
