# lingua-apple-app — Safari container app (iOS + macOS)

## ADDED Requirements

### Requirement: Minimal host for the Safari extension
The container app SHALL host the `safari` variant of the Lingua extension for iOS and macOS under a single universal App Store listing (one Xcode project, universal purchase), and SHALL contain no learning feature of its own: reading, statuses, decks, review and sign-in live in the extension. The extension resources bundled by the app SHALL be the output of the extension's `safari` build, never a copy maintained inside the Xcode project.

#### Scenario: The app ships the current extension build
- **WHEN** the app is built after a change to the extension source
- **THEN** the bundled Safari extension is the freshly built `safari` variant, with no manual copy step

#### Scenario: One listing for both platforms
- **WHEN** the app is published
- **THEN** a single App Store listing installs it on iPhone, iPad and Mac

### Requirement: Guided activation
The app SHALL guide the user through enabling the Safari extension, which Safari ships disabled. On iOS, which exposes no extension-state API, it SHALL show the steps to enable the extension and allow websites, and where to open the extension in Safari (the address-bar menu). On macOS, it SHALL offer a button that opens Safari's settings on the extension (`SFSafariApplication.showPreferencesForExtension`) and SHALL show the real enabled state (`SFSafariExtensionManager`). The shipping copy is French.

#### Scenario: First launch on iOS
- **WHEN** the user opens the app on an iPhone
- **THEN** the app shows how to enable Cymbra Lingua in Settings, allow websites, and open the extension from Safari's address-bar menu

#### Scenario: Activation from macOS
- **WHEN** the user clicks « Activer dans Safari » on macOS
- **THEN** Safari's settings open on the extension, and the app shows « extension active » once the state API confirms it

### Requirement: No learning state in the app
The container app SHALL NOT hold learning state or a session: the Safari extension SHALL keep its state in its own extension storage, under the same versioned schema as the other variants, and SHALL sign in and synchronise on its own like them.

#### Scenario: Using the extension without opening the app again
- **WHEN** the user has enabled the extension and never reopens the app
- **THEN** highlighting, decks, review and sign-in all work from Safari alone
