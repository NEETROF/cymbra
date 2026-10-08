## MODIFIED Requirements

### Requirement: The backup records the reader's language profile
The backup SHALL record the reader's language profile: their studied languages, an ordered list, non-empty and without duplicates, whose first language is the primary one, and their native language, which is never one of the studied languages. The default profile is English studied, French native. A backup whose profile is the default and whose every per-language record is English SHALL be written with schema version 1 and the exact content it had before the profile was stored. A backup that names French as a studied language — in its profile or in any per-language record — SHALL be written with schema version 3, the profile included. Any other backup SHALL be written with schema version 2, the profile included. A backup is written in the oldest schema version that holds it, so a reader of English or Spanish keeps the version they have, byte for byte, and a French native language alone never raises it. A choice of studied languages that holds the native language SHALL be refused, and the native language SHALL be set only together with the studied languages, the whole choice being refused when it would study the native language; a refused choice SHALL leave the profile as it was. A full reset SHALL give the reader the engine's native language, studying the language of the engine's first pack alone; resetting statuses SHALL leave the profile unchanged. The profile SHALL never be sent to the server as such: the native language travels only as the label of a day's statistics, and the language of a gloss as the card's.

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

#### Scenario: A French reader's backup
- **WHEN** the profile is set to French studied with English native, and the state is backed up and restored
- **THEN** the backup has schema version 3, and the restored profile studies French with English native

#### Scenario: French in the records only
- **WHEN** the profile studies Spanish with English native, and the deck holds a French card, or the knowledge a French status, or the exposures a French counter
- **THEN** the backup has schema version 3

#### Scenario: A French-native reader cannot study French
- **WHEN** the profile is set to French native with French among the studied languages
- **THEN** it is refused and the previous profile stays

#### Scenario: A Spanish reader's backup does not move
- **WHEN** a reader whose profile studies Spanish then English with French native is backed up by a build that writes schema version 3
- **THEN** the backup has schema version 2 and is byte for byte what the previous build wrote

### Requirement: A restore reads the backup's schema version first
The restore of a backup SHALL read its schema version before the rest of the file, SHALL read schema versions 1, 2 and 3, and SHALL refuse any other version as unsupported, naming the version found and the newest version it reads. When a restore is refused, the extension SHALL not start the surface that asked for it and SHALL leave the stored backup unchanged. A build released before schema version 2 existed refuses a version 2 backup: as malformed when the backup holds another language's records, as unsupported otherwise. A build released with schema version 2 and before schema version 3 reads the version first, so it refuses a version 3 backup as unsupported version 3, never as malformed, whatever records the backup holds.

#### Scenario: Both known versions
- **WHEN** a version 1 backup, a version 2 backup and a version 3 backup are restored
- **THEN** all three are read

#### Scenario: A later version
- **WHEN** a backup of schema version 4 is restored
- **THEN** the restore fails as unsupported version 4, naming 3 as the newest it reads, and the stored backup is unchanged

#### Scenario: A build released before version 2
- **WHEN** a build released before this change opens a store whose backup has schema version 2 and Spanish cards
- **THEN** its restore fails as malformed, its reader does not start, and the backup is still there for a later build

#### Scenario: A build released before version 3
- **WHEN** a build released after schema version 2 and before schema version 3 opens a store whose backup has schema version 3 and French records
- **THEN** its restore fails as unsupported version 3, never as malformed, its reader does not start, and the backup is still there for a later build
