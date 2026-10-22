## ADDED Requirements

### Requirement: The manifest's Italian is committed before a pair glossed in Italian ships
The extension SHALL commit `_locales/it/messages.json` holding its description and its three keyboard commands' descriptions in Italian, addressing the reader as « tu », each command's description containing the words the Italian interface's settings give that command. The version check SHALL hold its Italian names for the studied languages equal to the Italian interface's catalogue. No package SHALL carry `_locales/it`, and no manifest SHALL read from it, while no pair glossed in Italian is listed.

#### Scenario: No pair glossed in Italian listed
- **WHEN** the packages are built while no pair glossed in Italian is listed
- **THEN** no package carries `_locales/it`, and no manifest reads from it

#### Scenario: en-it listed
- **WHEN** en-it is listed beside today's pairs and the packages are built
- **THEN** each package carries `_locales/it` beside the other shipped natives' folders, its manifest still defaults to English, and every reference to a message resolves in `_locales/it`

#### Scenario: A browser in Italian
- **WHEN** a browser in Italian shows the extension built with en-it the only pair glossed in Italian listed
- **THEN** it shows the Italian description, which names English

#### Scenario: The shortcuts page and the settings alike
- **WHEN** a command's description in `_locales/it` does not contain the words the Italian settings give that command
- **THEN** the extension's tests fail, naming the command

#### Scenario: Italian's names as the interface's
- **WHEN** the version check's Italian names and the Italian interface's catalogue differ for a studied language
- **THEN** the extension's tests fail
