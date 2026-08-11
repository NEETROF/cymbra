## ADDED Requirements

### Requirement: Foreground presentation is a per-category choice

A notification arriving while the app is **in the foreground** SHALL be surfaced
only if the category that owns it declares foreground presentation. A category that
declares nothing SHALL NOT be surfaced in the foreground — the safe default,
matching every other gate on this platform.

The declaration SHALL live with the category (alongside its id and label), so a
feature decides for its own notification type without touching the platform.

#### Scenario: A category that opts in is surfaced

- **WHEN** a notification for a category declaring foreground presentation arrives while the app is open
- **THEN** it is surfaced in the app

#### Scenario: A category that says nothing stays silent

- **WHEN** a notification for a category with no foreground declaration arrives while the app is open
- **THEN** nothing is surfaced

#### Scenario: Background delivery is unchanged

- **WHEN** a notification arrives while the app is backgrounded or killed
- **THEN** the operating system displays it as before, whatever the category declares

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
the category's declaration is the only thing that decides.

#### Scenario: macOS matches the others

- **WHEN** a notification arrives while the app is in the foreground on macOS
- **THEN** no system banner is shown, and the category's declaration decides as it does on iOS and Android

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
