## ADDED Requirements

### Requirement: The manifest's Italian is committed before a pair glossed in Italian ships
The extension SHALL commit `_locales/it/messages.json` holding its description and its three keyboard commands' descriptions in Italian, addressing the reader as « tu », the commands named with the nouns the Italian interface gives them, reviewed by the Italian reviewer agent; once a pair glossed in Italian ships, that description SHALL name exactly the languages the shipped pairs glossed in Italian study, within the 112 characters Apple accepts. The version check SHALL hold Italian's names for the studied languages equal to the Italian interface's catalogue. While no pair glossed in Italian is listed, no package SHALL carry `_locales/it`, and every package SHALL be byte for byte what it was.

#### Scenario: Committed, not shipped
- **WHEN** the packages are built while no pair glossed in Italian is listed
- **THEN** no package carries `_locales/it`, and every package is byte for byte what it was before this change

#### Scenario: en-it listed
- **WHEN** en-it is listed beside today's pairs and the packages are built
- **THEN** each package carries `_locales/it` beside the other shipped natives' folders, its manifest still defaults to English, and every reference to a message resolves in `_locales/it`

#### Scenario: A browser in Italian
- **WHEN** a browser in Italian shows the extension built with en-it listed
- **THEN** it shows the Italian description, which names English and no other studied language

#### Scenario: A summary naming a language Italian readers cannot study
- **WHEN** en-it is the only pair glossed in Italian listed and `_locales/it`'s description names French
- **THEN** the version check fails, naming the Italian description and French

#### Scenario: Italian's names as the interface's
- **WHEN** the version check's Italian names and the Italian interface's catalogue differ for a studied language
- **THEN** the extension's tests fail
