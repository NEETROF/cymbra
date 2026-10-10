## MODIFIED Requirements

### Requirement: Guided activation
The app SHALL guide the user through enabling the Safari extension, which Safari ships disabled. On iOS, which exposes no extension-state API, it SHALL show the steps to enable the extension and allow websites, and where to open the extension in Safari (the address-bar menu). On macOS, it SHALL offer a button that opens Safari's settings on the extension (`SFSafariApplication.showPreferencesForExtension`) and SHALL show the real enabled state (`SFSafariExtensionManager`). The copy SHALL be in the extension's interface language once the extension has run, and in the device's language before, among the languages the app offers — the native languages of the extension's shipped pairs, with English as the fallback once English ships and French otherwise — and each language's copy SHALL name every language the shipped pairs glossed in it study, saying where a language after the first is chosen before its level.

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

#### Scenario: Every language an English speaker can study
- **WHEN** fr-en is listed and the app is opened on a device in English before the extension has run
- **THEN** the page says the web is read in Spanish or French, and its last step has the level of Spanish chosen, then French ticked in the extension's settings, under Language, and its level chosen

#### Scenario: The French page
- **WHEN** the app is opened on a device in French
- **THEN** the page names English and Spanish, the languages a French-native reader can study, and never French

#### Scenario: Spanish speakers without fr-es
- **WHEN** fr-es is not listed and the app is opened on a device in Spanish
- **THEN** the page names English alone, as before
