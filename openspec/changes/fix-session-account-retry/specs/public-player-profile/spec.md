## ADDED Requirements

### Requirement: An unresolved own identity is reported as recoverable, not as a missing profile

The app SHALL NOT tell a signed-in user that their own profile is unavailable when
the real cause is that the app has not resolved their identity yet. "This profile
isn't available." is a statement about the *account*, and it is false here — the
account exists and the user is signed in; only the local resolution failed. Saying
it contradicts the guarantee that a user always sees their own profile, and it
sends the user looking for a problem with their account that does not exist.

When the user opens their own profile while their identity is unresolved, the app
SHALL present the state as temporary and SHALL offer an explicit way to retry.
Choosing retry SHALL re-attempt the account resolution and, on success, show the
profile. The message shown for **another** player's genuinely unavailable profile
SHALL be unchanged.

#### Scenario: Own profile with an unresolved identity offers a retry
- **WHEN** a signed-in user opens their own profile while the app has not resolved their identity
- **THEN** the screen presents the state as temporary and offers a retry, instead of reporting that the profile is not available

#### Scenario: Retry recovers the profile
- **WHEN** the user chooses retry and the account resolves
- **THEN** their own profile is shown

#### Scenario: Another player's unavailable profile is unchanged
- **WHEN** a user opens another player's profile that is private or ineligible
- **THEN** the existing "not available" message is shown, with no retry offered
