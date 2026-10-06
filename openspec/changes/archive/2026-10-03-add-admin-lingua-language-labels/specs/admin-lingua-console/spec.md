## ADDED Requirements

### Requirement: The studied languages are named in the console's language
The Lingua screen SHALL name each studied language it shows — in the per-language breakdown and in the studied-language filter's options — in the console's language, and SHALL show the language's code for a language it has no name for. The filter's values SHALL remain the languages' codes.

#### Scenario: English and Spanish in French
- **WHEN** the console is in French and the usage report breaks activity down for `en` and `es`
- **THEN** the breakdown's rows and the filter's options read « Anglais » and « Espagnol »

#### Scenario: A language the console does not name yet
- **WHEN** the usage report breaks activity down for a language the console has no name for
- **THEN** that language shows as its code, in the breakdown and in the filter

#### Scenario: The selection is a code
- **WHEN** the admin chooses « Espagnol » in the filter
- **THEN** the series are requested for `es`
