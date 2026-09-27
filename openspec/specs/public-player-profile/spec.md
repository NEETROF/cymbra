# public-player-profile Specification

## Purpose
TBD - created by archiving change add-play-activity-profile. Update Purpose after archive.
## Requirements
### Requirement: A player's profile is viewable by other players

The backend SHALL expose an authenticated operation for one player to view another player's
profile, returning only an **allow-listed public field set**: the player's handle/display
name, level, badges, the play heatmap (per-day activity), and songs-played totals. The
operation MUST NOT return sensitive fields — in particular **email**, the **curator
alignment/reliability** figures, and any moderation state MUST NEVER be exposed to another
player. Unauthenticated requests MUST be rejected.

#### Scenario: Public fields are returned to another player

- **WHEN** an authenticated player views another player's public profile
- **THEN** they see the handle/display name, level, badges, play heatmap, and songs-played totals

#### Scenario: Sensitive fields are never exposed

- **WHEN** any player's public profile is read by someone else
- **THEN** the response contains no email, no curator alignment/reliability figure, and no moderation state

#### Scenario: Unauthenticated profile view rejected

- **WHEN** a profile-view request arrives without a valid authenticated identity
- **THEN** it is rejected

### Requirement: Per-user profile visibility control, private by default

The system SHALL let a user control the visibility of their profile to other players
(public or private) and SHALL default a profile to **private** — a
profile is exposed to other players only after the user **explicitly opts in**. When a user
has restricted or hidden their profile, other players MUST NOT receive the restricted
content, while the user themselves always sees their own full profile.

#### Scenario: New profile is private by default

- **WHEN** a user has never changed their visibility setting and another player requests their profile
- **THEN** the restricted content is not returned; the profile is private until the user opts in

#### Scenario: Hidden profile is not shown to others

- **WHEN** a user has set their profile to private and another player requests it
- **THEN** the restricted content is not returned to that other player

#### Scenario: Owner always sees their own profile

- **WHEN** a user views their own profile
- **THEN** they see it in full regardless of their visibility setting

#### Scenario: Visibility setting is honored on read

- **WHEN** a user changes their profile visibility
- **THEN** subsequent reads by other players honor the new setting

### Requirement: Public sharing gated by a minimum-age safeguard

Making a profile public SHALL require the user to be at least a **configured minimum age**
(`min_public_sharing_age`, default 16). Age SHALL be asked only at the point of opting in,
via a neutral prompt, and the system SHALL persist only a **derived eligibility date**
(`share_eligible_from`) — the user's date of birth MUST NOT be stored. The eligibility check
SHALL be enforced **server-side and fail-closed**: a request to make the profile public MUST
be refused when the user is not yet eligible, and a public read MUST NOT expose the profile
of a user who is not eligible, so a modified client cannot bypass the safeguard. Eligibility
SHALL be evaluated as a **date comparison in UTC with a one-day safety margin**, so a
timezone difference can never grant eligibility early; a not-yet-eligible user keeps full use
of everything else while their profile stays private.

#### Scenario: Under-age user cannot make the profile public

- **WHEN** a user below the configured minimum age tries to set their profile public
- **THEN** the request is refused and the profile stays private

#### Scenario: Eligible user can opt in

- **WHEN** a user who meets the minimum age opts in
- **THEN** their profile may be made public

#### Scenario: Date of birth is not retained

- **WHEN** a user passes the age gate
- **THEN** only the derived eligibility date is stored and the date of birth is not retained

#### Scenario: Server enforces eligibility fail-closed

- **WHEN** a client attempts to expose or read a not-yet-eligible user's public profile
- **THEN** the server refuses, regardless of what the client sends

#### Scenario: Eligibility is a conservative UTC date check

- **WHEN** the eligibility date falls on the boundary day across timezones
- **THEN** eligibility is granted only once it holds in UTC with a one-day margin, never early

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

