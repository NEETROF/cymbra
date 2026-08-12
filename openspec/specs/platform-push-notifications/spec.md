# platform-push-notifications Specification

## Purpose
TBD - created by archiving change add-push-notifications. Update Purpose after archive.
## Requirements
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

### Requirement: Foreground presentation is a per-category, hot-reloadable choice

A notification arriving while the app is **in the foreground** SHALL be surfaced
only if its category is configured to be. That configuration SHALL be a
per-category **feature flag**, beside the per-category enable and schedule-hour
keys, so it is changed from the back office without an app release. Its default
SHALL be "not surfaced" — the safe direction, matching every other gate on this
platform.

The client SHALL NOT carry the decision as a compiled-in constant.

#### Scenario: A category configured for the foreground is surfaced

- **WHEN** a notification for a category whose foreground flag is on arrives while the app is open
- **THEN** it is surfaced in the app

#### Scenario: An unconfigured category stays silent

- **WHEN** a notification for a category with no foreground flag set arrives while the app is open
- **THEN** nothing is surfaced

#### Scenario: The policy changes without shipping an app

- **WHEN** an administrator turns a category's foreground flag on
- **THEN** subsequent notifications for that category are surfaced by already-installed apps

#### Scenario: Background delivery is unchanged

- **WHEN** a notification arrives while the app is backgrounded or killed
- **THEN** the operating system displays it as before, whatever the flag says

### Requirement: The foreground decision travels with the notification

The dispatch SHALL resolve the category's foreground flag and carry the result on
the notification itself, so the client presents what it is told rather than
deriving it. A notification carrying no such indication SHALL NOT be surfaced.

#### Scenario: An unknown category still behaves correctly

- **WHEN** a notification arrives for a category this build of the app has never heard of
- **THEN** it is surfaced or not according to what the notification carries, not according to what the app knows

#### Scenario: An indication-less notification is silent

- **WHEN** a notification arrives in the foreground carrying no foreground indication
- **THEN** nothing is surfaced

### Requirement: Foreground notifications are presented in-app

A foreground notification SHALL be presented as an **in-app** surface, not as an
operating-system notification. The app SHALL NOT raise a local OS notification for a
message it received while in the foreground.

#### Scenario: No system banner over the app it came from

- **WHEN** a foreground notification is surfaced
- **THEN** it is rendered inside the app, and no OS notification is posted

#### Scenario: The user can dismiss it

- **WHEN** a foreground notification is surfaced
- **THEN** the user can dismiss it, and dismissing it surfaces nothing further

### Requirement: One foreground baseline across platforms

Foreground behaviour SHALL be identical on iOS, Android and macOS: the operating
system SHALL NOT display its own banner while the app is in the foreground, so that
the category's configuration is the only thing that decides.

#### Scenario: macOS matches the others

- **WHEN** a notification arrives while the app is in the foreground on macOS
- **THEN** no system banner is shown, and the category's configuration decides as it does on iOS and Android

### Requirement: A foreground notification routes like a background one

Acting on a surfaced foreground notification SHALL use the same routing payload the
platform already transports with the message, so a category describes its
destination once for both surfaces.

#### Scenario: Tapping goes where the notification points

- **WHEN** the user taps a surfaced foreground notification carrying a routing payload
- **THEN** the app navigates to the same destination it would have from a background tap

#### Scenario: A notification with no routing payload is inert

- **WHEN** the user taps a surfaced foreground notification carrying no routing payload
- **THEN** it is dismissed and no navigation occurs

