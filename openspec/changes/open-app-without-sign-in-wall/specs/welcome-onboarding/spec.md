## MODIFIED Requirements

### Requirement: First-run welcome runs without an account and never forces sign-up

The app SHALL present, at first launch and **before any sign-in**, a short welcome that states
the value and routes the user toward a first action. The welcome MUST NOT require an account and
MUST always be **skippable**; sign-in SHALL be offered only as an **option**, never as a
mandatory wall to proceed. Leaving the welcome without signing in — by skipping it or by
continuing without an account — SHALL enter guest mode and open the app, and MUST NOT lead to
an account entry screen. The welcome's sign-in action SHALL open the contextual sign-in surface
over the welcome. The welcome SHALL follow the language step and precede the existing post-auth
handle gate (language → welcome → optional sign-in → handle gate).

#### Scenario: Welcome shown before any account exists

- **WHEN** the app is launched for the first time with no account
- **THEN** the welcome is shown without requiring sign-in

#### Scenario: Welcome is skippable

- **WHEN** the user chooses to skip the welcome
- **THEN** guest mode is entered and the app opens on the library, with no account entry screen in between

#### Scenario: Continuing without an account opens the app

- **WHEN** the user chooses "continue without an account" on the welcome
- **THEN** guest mode is entered and persisted, and the app opens on the library

#### Scenario: Sign-up is not forced

- **WHEN** the user declines to sign in from the welcome
- **THEN** they can still continue, and are not blocked behind a mandatory account wall

#### Scenario: Signing in from the welcome

- **WHEN** the user chooses to sign in on the welcome and completes authentication
- **THEN** the welcome ends and the app continues with the new session

#### Scenario: Leaving the sign-in surface returns to the welcome

- **WHEN** the user opens the sign-in surface from the welcome and leaves it without authenticating
- **THEN** they are back on the welcome, free to try, sign in or continue without an account
