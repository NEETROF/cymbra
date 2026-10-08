## ADDED Requirements

### Requirement: The sign-in sheet speaks the extension's language
The host app's sign-in sheet SHALL show its copy in the interface language the extension names in the sign-in link, when the app offers that language, and in the app's preferred localisation otherwise, the languages the app offers being those *Guided activation* names. A failure SHALL name its provider and SHALL never read as a password error, in every language.

#### Scenario: Every reader today
- **WHEN** a reader whose interface language is French signs in with Apple from Safari
- **THEN** the sheet reads as before, byte for byte

#### Scenario: A Spanish-native reader
- **WHEN** en-es ships and the extension opens the sheet with its interface language, Spanish
- **THEN** the sheet's heading, messages and its own buttons are the Spanish copy; Apple's button keeps the label the system gives it

#### Scenario: A language the app does not offer
- **WHEN** the sheet is opened with `lang=en` while every shipped pair is glossed in French
- **THEN** it shows the French copy

#### Scenario: An older link
- **WHEN** the sheet is opened by a link that names no language
- **THEN** it shows the app's preferred localisation among the languages it offers

## MODIFIED Requirements

### Requirement: Guided activation
The app SHALL guide the user through enabling the Safari extension, which Safari ships disabled. On iOS, which exposes no extension-state API, it SHALL show the steps to enable the extension and allow websites, and where to open the extension in Safari (the address-bar menu). On macOS, it SHALL offer a button that opens Safari's settings on the extension (`SFSafariApplication.showPreferencesForExtension`) and SHALL show the real enabled state (`SFSafariExtensionManager`). The copy SHALL be in the extension's interface language once the extension has run, and in the device's language before, among the languages the app offers — the native languages of the extension's shipped pairs, with English as the fallback once English ships and French otherwise — and the French copy SHALL stay what it was.

#### Scenario: First launch on iOS
- **WHEN** the user opens the app on an iPhone
- **THEN** the app shows how to enable Cymbra Lingua in Settings, allow websites, and open the extension from Safari's address-bar menu

#### Scenario: Activation from macOS
- **WHEN** the user clicks « Activer dans Safari » on macOS
- **THEN** Safari's settings open on the extension, and the app shows « extension active » once the state API confirms it

#### Scenario: A device in English before English ships
- **WHEN** the app is opened on a device in English and every shipped pair is glossed in French
- **THEN** the page is in French, as before

#### Scenario: A device in English once English ships
- **WHEN** es-en ships and the app is opened on a device in English before the extension has run
- **THEN** the page is the English copy, and its `lang` is `en`
