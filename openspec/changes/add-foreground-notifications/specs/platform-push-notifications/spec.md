## ADDED Requirements

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
