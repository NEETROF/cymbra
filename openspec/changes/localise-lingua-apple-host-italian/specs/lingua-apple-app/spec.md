## ADDED Requirements

### Requirement: The host app speaks Italian once a pair glossed in Italian ships
The host app SHALL hold Italian copy for its activation page and its sign-in sheets, addressing the reader as « tu », and SHALL accept Italian wherever it reads a language: the device's preferred localisation, the language the extension stores, and the sign-in link's `lang`. It SHALL offer Italian only when a shipped pair is glossed in Italian, as it offers every language. The activation page's Italian SHALL name the languages the shipped pairs glossed in Italian study.

#### Scenario: A device in Italian before a pair glossed in Italian ships
- **WHEN** the app is opened on a device in Italian and no shipped pair is glossed in Italian
- **THEN** the page is in the language it shows today, and the app declares no Italian localisation

#### Scenario: A device in Italian once en-it ships
- **WHEN** en-it ships and the app is opened on a device in Italian before the extension has run
- **THEN** the page is the Italian copy, its `lang` is `it`, and it names English

#### Scenario: The sign-in sheet in Italian
- **WHEN** en-it ships and the extension opens the sheet with `lang=it`
- **THEN** the sheet's heading, messages and its own buttons are the Italian copy, and Apple's button keeps the label the system gives it

#### Scenario: The extension's stored language
- **WHEN** en-it ships and the extension records Italian as its interface language
- **THEN** the host app keeps it, and the page and the app's own sign-in sheet follow it

#### Scenario: Italian named before it is offered
- **WHEN** the sheet is opened with `lang=it` while no shipped pair is glossed in Italian
- **THEN** it shows the app's preferred localisation among the languages it offers
