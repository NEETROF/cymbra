## MODIFIED Requirements

### Requirement: The backup records the reader's language profile
The backup SHALL record the reader's language profile: their studied languages, an ordered list, non-empty and without duplicates, whose first language is the primary one, and their native language, which is never one of the studied languages. The default profile is English studied, French native. A backup whose profile is the default and whose every per-language record is English SHALL be written with schema version 1 and the exact content it had before the profile was stored. Any other backup SHALL be written with schema version 2, the profile included. A choice of studied languages that holds the native language SHALL be refused, and the native language SHALL be set only together with the studied languages, the whole choice being refused when it would study the native language; a refused choice SHALL leave the profile as it was. A full reset SHALL give the reader the engine's native language, studying the language of the engine's first pack alone; resetting statuses SHALL leave the profile unchanged. The profile SHALL never be sent to the server as such: the native language travels only as the label of a day's statistics, and the language of a gloss as the card's.

#### Scenario: An English reader's backup does not change
- **WHEN** an English reader with 20 cards and 300 statuses is backed up
- **THEN** the backup has schema version 1, carries no profile, and is byte for byte what the previous build wrote

#### Scenario: Another studied language
- **WHEN** the studied languages are set to Spanish then English, and the state is backed up and restored
- **THEN** the backup has schema version 2, and the restored studied languages are Spanish then English

#### Scenario: Another language in the records only
- **WHEN** the profile is the default but the deck holds a Spanish card
- **THEN** the backup has schema version 2

#### Scenario: Another native language
- **WHEN** the profile is set to Spanish studied with English native, and the state is backed up and restored
- **THEN** the backup has schema version 2, and the restored profile studies Spanish with English native

#### Scenario: A backup written before the native language could change
- **WHEN** a version 2 backup written by the previous build, whose profile studies Spanish then English, is restored
- **THEN** its native language is French and its studied languages are Spanish then English

#### Scenario: Resets
- **WHEN** a reader who studies English and Spanish erases their data, on an engine whose first pack is en-fr
- **THEN** the profile is the default again, whereas resetting statuses leaves it unchanged

#### Scenario: A full reset on an engine glossed in English
- **WHEN** a reader erases their data on an engine whose first pack studies Spanish glossed in English
- **THEN** their native language is English and they study Spanish alone

#### Scenario: An invalid choice of studied languages
- **WHEN** studied languages are set as an empty list, as a list naming a language twice, or as a list holding the native language
- **THEN** they are refused and the previous studied languages stay

#### Scenario: A native language the reader would study
- **WHEN** the profile is set to English native with English among the studied languages
- **THEN** it is refused and the previous profile stays

#### Scenario: Never synced
- **WHEN** a sync pushes and pulls
- **THEN** no request or response carries the profile; a statistic carries the native language of its day, and a card the language of its gloss
