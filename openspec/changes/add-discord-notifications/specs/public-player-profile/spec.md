## ADDED Requirements

### Requirement: Dedicated Discord-visibility consent, off by default

The system SHALL provide a **separate consent** authorising the player to be **named on the
Cymbra community Discord server**, distinct from the public/private profile setting. It SHALL
default to **off** and SHALL be changeable by the player at any time. The two settings are
**independent and cumulative**: making a profile public MUST NOT grant Discord visibility, and
granting Discord visibility MUST NOT make a profile public. A player MAY be named on Discord
only when the Discord consent is on **and** the player is publicly listable (public profile and
age-eligible), evaluated server-side and **fail-closed**.

#### Scenario: Consent is off for a new account

- **WHEN** a player has never touched the setting
- **THEN** they are never named on Discord

#### Scenario: Public profile alone grants nothing

- **WHEN** a player sets their profile public without enabling Discord visibility
- **THEN** they are still never named on Discord

#### Scenario: Discord consent alone grants nothing

- **WHEN** a player enables Discord visibility while their profile is private or they are not age-eligible
- **THEN** they are still never named on Discord

#### Scenario: Both conditions allow naming

- **WHEN** a player has Discord visibility enabled and is publicly listable
- **THEN** announcements about them may include their public display name

#### Scenario: Client cannot bypass the gate

- **WHEN** a modified client claims Discord visibility for a player who has not consented
- **THEN** the server refuses to name that player

### Requirement: Revoking Discord visibility is forward-only and disclosed as such

Turning the Discord consent off SHALL suppress **future** announcements naming the player, and
the system MUST NOT claim to remove messages already published — a published Discord message
cannot be retracted by Cymbra. The setting's own copy in the app SHALL state this
irreversibility plainly **before** the player opts in, and the privacy documentation SHALL
describe the Discord publication and its permanence.

#### Scenario: Revocation stops future announcements

- **WHEN** a player turns Discord visibility off
- **THEN** no subsequent announcement names them, including announcements already enqueued

#### Scenario: Irreversibility is stated before opting in

- **WHEN** the player is shown the Discord-visibility setting
- **THEN** the copy states that messages already published on Discord stay published

### Requirement: Discord consent is erased with the account

The Discord-visibility consent SHALL be part of the account data erased when the account is
deleted, so no residual consent survives the account. After erasure, the player MUST NOT be
nameable by any pending or future announcement.

#### Scenario: Account erasure removes the consent

- **WHEN** an account is deleted and its erasure completes
- **THEN** the stored Discord consent is gone

#### Scenario: Pending announcement about an erased account names nobody

- **WHEN** an announcement job runs after the account it concerns was erased
- **THEN** it publishes nothing naming that account
