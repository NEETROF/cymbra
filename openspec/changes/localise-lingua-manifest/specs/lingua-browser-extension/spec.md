## ADDED Requirements

### Requirement: The manifest speaks the browser's language among the shipped natives
The extension's description and its keyboard commands' descriptions SHALL be read by the browser from `_locales` in each native language a shipped pair is glossed in, with English as the default for any other browser language once an English-glossed pair ships, and French otherwise; the brand name SHALL stay « Cymbra Lingua » in every language. While every shipped pair is glossed in French, the packages SHALL carry today's literal French manifest, with no `_locales`. Every committed description SHALL fit the 112 characters Apple accepts.

#### Scenario: Every reader today
- **WHEN** the packages are built with only French-native pairs listed
- **THEN** each manifest is byte for byte what it was, with no `_locales` and no `default_locale`

#### Scenario: An English-glossed pair ships
- **WHEN** es-en is listed and the packages are built
- **THEN** each package's manifest names its description and commands by `__MSG_` reference and defaults to English, and each package carries `_locales/fr` and `_locales/en` and no other

#### Scenario: A browser in German
- **WHEN** a browser in German shows the extension built with es-en listed
- **THEN** it shows the English description

#### Scenario: A description too long for Apple
- **WHEN** a committed description holds more than 112 characters
- **THEN** the version check fails, naming its language
