## ADDED Requirements

### Requirement: An unresolved account is re-resolved without user action

The app SHALL keep re-attempting account resolution for as long as a session is
authenticated but its account is unresolved. Keeping the user signed in after a
transient `GetAccount` failure is correct, but that degraded state MUST NOT be
terminal: it hides the user's handle, leaves the app with no user identity, and
silently disables everything keyed on it (play-session capture, feature-flag
targeting, the favorites index, the offline cache, plan resolution). It SHALL NOT
require the user to sign out, restart, or take any action to recover.

Re-attempts SHALL use a bounded backoff with a maximum interval, so an
unreachable backend is never hot-looped. Resolution SHALL be single-flight: at
most one account resolution is in flight at a time, and a concurrent trigger
joins the pending attempt rather than issuing a second call. Re-attempts SHALL
stop as soon as the account resolves or the session ends.

A **terminal** failure encountered during a re-attempt (the session is revoked,
or the account no longer exists) SHALL clear the stored session and route the
user to the entry screen, exactly as it does on the first attempt.

#### Scenario: A transient failure at sign-in recovers on its own
- **WHEN** the account cannot be fetched after a successful sign-in because of a transient failure, and the backend becomes reachable again
- **THEN** the app re-resolves the account with no user action, and the handle, user identity and every identity-keyed feature become available

#### Scenario: A transient failure at launch recovers on its own
- **WHEN** the app launches with a stored session, the account cannot be fetched because of a transient failure, and the backend becomes reachable again
- **THEN** the app re-resolves the account with no user action

#### Scenario: An unreachable backend is not hot-looped
- **WHEN** account resolution keeps failing transiently
- **THEN** successive re-attempts are spaced by a growing delay up to a maximum interval, rather than repeating immediately

#### Scenario: Concurrent resolutions are coordinated
- **WHEN** a re-attempt is triggered while an account resolution is already in flight
- **THEN** exactly one resolution call is made and the trigger reuses its outcome

#### Scenario: Re-attempts stop once the account resolves
- **WHEN** an account resolution succeeds
- **THEN** no further re-attempt is scheduled

#### Scenario: Re-attempts stop when the session ends
- **WHEN** the user signs out, deletes the account, or the session is otherwise torn down while re-attempts are scheduled
- **THEN** the pending re-attempt is cancelled and no further account resolution is issued

#### Scenario: A terminal failure during a re-attempt signs the user out
- **WHEN** a re-attempt fails because the session is revoked or the account no longer exists
- **THEN** the stored session is cleared and the entry screen is shown

### Requirement: Account re-resolution runs only in the foreground

The app SHALL suspend account re-resolution whenever it is not in the foreground
and SHALL resume it on returning to the foreground. A pending re-attempt SHALL be
cancelled when the app is paused, hidden, or detached, so neither a timer nor a
network call outlives the foreground session. This is not implied by the host
platform: a backgrounded desktop app keeps executing (unlike a mobile process,
which is frozen), so an ungated backoff loop would keep spending battery and
issuing calls against a backend the user is not looking at.

On returning to the foreground with a still-unresolved account, the app SHALL
attempt resolution immediately and reset the backoff, rather than waiting out the
delay accumulated before it was suspended.

#### Scenario: Leaving the foreground cancels the pending re-attempt
- **WHEN** the app is paused, hidden, or detached while a re-attempt is scheduled
- **THEN** the scheduled re-attempt is cancelled and no account resolution call is issued while the app is out of the foreground

#### Scenario: Returning to the foreground retries immediately
- **WHEN** the app returns to the foreground and the session is still authenticated with an unresolved account
- **THEN** account resolution is attempted immediately and the backoff is reset

#### Scenario: Returning to the foreground with a resolved account does nothing
- **WHEN** the app returns to the foreground and the account is already resolved
- **THEN** no account resolution is issued

#### Scenario: A resume during an in-flight attempt does not double-fire
- **WHEN** the app returns to the foreground while an account resolution is already in flight
- **THEN** no second resolution is issued and the in-flight one is awaited
