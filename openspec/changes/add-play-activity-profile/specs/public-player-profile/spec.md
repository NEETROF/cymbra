## ADDED Requirements

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

### Requirement: Per-user profile visibility control

The system SHALL let a user control the visibility of their profile to other players
(for example public, limited, or private). When a user has restricted or hidden their
profile, other players MUST NOT receive the restricted content, while the user themselves
always sees their own full profile.

#### Scenario: Hidden profile is not shown to others

- **WHEN** a user has set their profile to private and another player requests it
- **THEN** the restricted content is not returned to that other player

#### Scenario: Owner always sees their own profile

- **WHEN** a user views their own profile
- **THEN** they see it in full regardless of their visibility setting

#### Scenario: Visibility setting is honored on read

- **WHEN** a user changes their profile visibility
- **THEN** subsequent reads by other players honor the new setting
