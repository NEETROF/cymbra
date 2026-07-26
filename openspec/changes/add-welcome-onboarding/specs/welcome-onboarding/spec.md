## ADDED Requirements

### Requirement: First-run welcome runs without an account and never forces sign-up

The app SHALL present, at first launch and **before any sign-in**, a short welcome that states
the value and routes the user toward a first action. The welcome MUST NOT require an account and
MUST always be **skippable**; sign-in SHALL be offered only as an **option**, never as a
mandatory wall to proceed. The welcome SHALL appear before the existing post-auth handle gate in
the flow (welcome → optional sign-in → handle gate).

#### Scenario: Welcome shown before any account exists

- **WHEN** the app is launched for the first time with no account
- **THEN** the welcome is shown without requiring sign-in

#### Scenario: Welcome is skippable

- **WHEN** the user chooses to skip the welcome
- **THEN** they proceed into the app without creating an account

#### Scenario: Sign-up is not forced

- **WHEN** the user declines to sign in from the welcome
- **THEN** they can still continue, and are not blocked behind a mandatory account wall

### Requirement: The core loop is experienceable without an account

The app SHALL provide a way to experience the **core loop** — playing a piece and seeing the
live synchronization gauge and the end-of-session summary — **without an account**, so the value
lands before any sign-up. This no-account experience SHALL exercise the real player and scoring,
and MUST NOT require opening the authenticated catalog to anonymous users.

#### Scenario: Try the core loop with no account

- **WHEN** a user chooses to try from the welcome without signing in
- **THEN** they can play the offered piece and see the live gauge and end-of-session summary

#### Scenario: Trying does not require the authenticated hub

- **WHEN** the no-account try runs
- **THEN** it does not require anonymous access to the authenticated catalog/hub

### Requirement: Sign-in is invited contextually, never blocking exploration

The app SHALL invite sign-in **contextually** when a user reaches a feature that genuinely
requires an account (for example saving a library, rating to earn points, appearing on a
leaderboard, or making a profile public), stating the **benefit** and letting the user
**decline and keep exploring**. The invitation MUST NOT be a dead-end wall: declining returns the
user to what they were doing.

#### Scenario: Gated action invites sign-in with a benefit

- **WHEN** a signed-out user triggers an account-gated action
- **THEN** the app invites sign-in and names the benefit of doing so

#### Scenario: Declining keeps the user exploring

- **WHEN** the user declines the sign-in invitation
- **THEN** they are returned to what they were doing and can keep using the app

#### Scenario: Signing in resumes the intended action

- **WHEN** the user accepts and completes sign-in from a gated action
- **THEN** they can proceed with the action they intended
