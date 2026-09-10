## ADDED Requirements

### Requirement: The app offers an entry point to the community

The app SHALL offer a discoverable entry point that opens the Cymbra community server in an
external browser or in the Discord app. The entry point SHALL be reachable from a stable
location in the account/help area, and its label and description SHALL be localized in every
supported app language.

#### Scenario: Entry point opens the community

- **WHEN** the user activates the community entry point
- **THEN** the community destination is opened externally

#### Scenario: Copy is localized

- **WHEN** the app runs in any supported language
- **THEN** the entry point's label and description are shown in that language

### Requirement: The invite is resolved through a stable redirect, never hardcoded

The app SHALL open a **stable Cymbra-owned redirect URL**, not a Discord invite code. A rotated
invite, a renamed server, or a temporarily closed server MUST be handled by changing the
redirect target server-side, without shipping an app release.

#### Scenario: Invite rotation needs no app release

- **WHEN** the Discord invite is rotated
- **THEN** the app keeps working because it only ever opened the redirect URL

#### Scenario: No invite code is embedded in the app

- **WHEN** the app binary or source is inspected
- **THEN** it contains the redirect URL and no Discord invite code

### Requirement: The entry point is governed by a runtime flag

Visibility of the community entry point SHALL be governed by a runtime feature flag, so it can
be withdrawn without an app release — for instance while the server is closed or under
moderation pressure. When the flag is disabled, the entry point SHALL be absent rather than
shown-and-failing.

#### Scenario: Flag disabled hides the entry point

- **WHEN** the community flag is disabled
- **THEN** the entry point is not displayed anywhere in the app

#### Scenario: Flag enabled shows the entry point

- **WHEN** the community flag is enabled
- **THEN** the entry point is displayed

### Requirement: Opening the community can never crash the app

The external open SHALL go through the injectable launcher seam already used for legal links,
and a launch failure (no browser handler, malformed target, cancelled by the OS) SHALL be
swallowed best-effort. The screen hosting the entry point MUST remain usable, and no raw
technical error SHALL be shown to the user.

#### Scenario: Launch failure is absorbed

- **WHEN** the platform cannot open the community destination
- **THEN** the app stays on the current screen, remains usable, and shows no technical error string

#### Scenario: Seam is injectable for tests

- **WHEN** the app is tested without the native URL-launching plugin
- **THEN** the entry point can be exercised through the injected seam and the opened target asserted
