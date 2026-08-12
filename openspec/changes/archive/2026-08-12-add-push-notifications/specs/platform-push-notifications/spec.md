## ADDED Requirements

### Requirement: FCM device-token registry

The server SHALL let an authenticated client register and refresh its **FCM device
token** with its platform, and unregister it. Only FCM-capable platforms (iOS,
Android, macOS) register; Windows and Linux clients SHALL NOT register a token. A
token SHALL be invalidated (removed) when a send reports it as unregistered/invalid,
and on explicit unregister (logout). A user MAY have multiple device tokens.

#### Scenario: Registering a token
- **WHEN** an iOS/Android/macOS client calls register with its FCM token and platform
- **THEN** the token is stored for that user (upserted, refreshing its last-seen)

#### Scenario: Desktop does not register
- **WHEN** a Windows or Linux client runs
- **THEN** it does not register a token and is never a push recipient

#### Scenario: Invalid token is pruned
- **WHEN** a send to a stored token reports it unregistered/invalid
- **THEN** that token is removed from the registry

#### Scenario: Logout unregisters
- **WHEN** a user logs out
- **THEN** that device's token is removed

### Requirement: Per-user timezone for local-time scheduling

The server SHALL store each user's timezone (or UTC offset), set and refreshed by
the client, so a scheduled send fires at the user's **local** time. If a user's
timezone is unknown, the send SHALL fall back to a defined default rather than
firing at an unintended hour.

#### Scenario: Scheduled send respects local time
- **WHEN** a send is scheduled for a local hour (e.g. 20:00) and users span timezones
- **THEN** each user is targeted at that hour in their own timezone

#### Scenario: Unknown timezone uses the default
- **WHEN** a user has no stored timezone
- **THEN** the send uses the defined fallback rather than an arbitrary hour

### Requirement: Consent and per-category preferences

Notifications SHALL be opt-in: a user without OS permission is never a recipient.
The server SHALL keep a per-user, per-**category** preference, and a send for a
category SHALL skip any user who has disabled that category. A global **kill-switch**
and a **per-category enable** SHALL exist as hot-reloadable feature flags; when the
kill-switch is off, or a category is disabled, no sends occur for it.

#### Scenario: Opted-out user is skipped
- **WHEN** a category send runs and a user has disabled that category
- **THEN** that user receives no notification

#### Scenario: Kill-switch halts all sends
- **WHEN** the global kill-switch is off
- **THEN** no notifications are sent for any category

#### Scenario: Category disabled by flag
- **WHEN** a category's enable flag is off
- **THEN** no sends occur for that category regardless of user preferences

### Requirement: Mockable send seam over a single FCM provider

The platform SHALL send through a `PushSender` port with an **FCM (HTTP v1)**
implementation that reaches iOS, Android and macOS via the one FCM API (Apple via
APNs bridging). The port SHALL be mockable so tests never call FCM. A send SHALL
return an outcome distinguishing delivered, retryable failure, and invalid-token, so
the caller can retry or prune.

#### Scenario: One provider reaches the three platforms
- **WHEN** recipients span iOS, Android and macOS
- **THEN** all are sent through the single FCM sender

#### Scenario: Tests use a mock sender
- **WHEN** the platform is under test
- **THEN** a mock `PushSender` is used and no real FCM call is made

#### Scenario: Invalid-token outcome prunes the token
- **WHEN** a send returns invalid-token for a stored token
- **THEN** the token is pruned from the registry

### Requirement: Recipient selection is a host-testable core

The decision of **which tokens receive a given category send** SHALL be a pure,
host-testable function of the candidate rows (user, token, timezone), the users'
category preferences, the feature flags, the category, and the current hour — so
every gate (kill-switch, category enable, opt-out, local-hour match, token presence)
is unit-tested without a DB or FCM.

#### Scenario: Only eligible tokens are selected
- **WHEN** selection runs with a mix of opted-in/out users, flags, and timezones
- **THEN** it returns exactly the tokens of opted-in users, for the enabled category, at the matching local hour, with the kill-switch on

### Requirement: Worker-driven dispatch entry point

The platform SHALL provide a worker dispatch entry point that, for a given category,
selects recipients and sends. Concrete notification **types** (their schedule/trigger,
candidate query, and message content) are added by the features that own them; this
platform SHALL NOT ship any concrete type itself.

#### Scenario: A feature registers a type on the platform
- **WHEN** a feature declares a category with a schedule/trigger, candidate query, and message
- **THEN** the worker dispatch runs selection + send for it without changes to the platform core
