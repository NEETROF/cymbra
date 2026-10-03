## ADDED Requirements

### Requirement: The backup records the reader's studied languages
The backup SHALL record the reader's studied languages: an ordered list, non-empty and without duplicates, whose first language is the primary one, English by default. A backup that holds only English, in that list and in every per-language record, SHALL be written with schema version 1 and the exact content it had before the list existed. A backup that holds any other language SHALL be written with schema version 2, the list included. A full reset SHALL return the list to English, resetting statuses SHALL leave it unchanged, and the list SHALL never be sent to the server.

#### Scenario: An English reader's backup does not change
- **WHEN** an English reader with 20 cards and 300 statuses is backed up
- **THEN** the backup has schema version 1, carries no list, and is byte for byte what the previous build wrote

#### Scenario: Another language in the list
- **WHEN** the list is set to Spanish then English, and the state is backed up and restored
- **THEN** the backup has schema version 2, and the restored list is Spanish then English

#### Scenario: Another language in the records only
- **WHEN** the list is English alone but the deck holds a Spanish card
- **THEN** the backup has schema version 2

#### Scenario: Resets
- **WHEN** the reader erases their data
- **THEN** the list is English alone again, whereas resetting statuses leaves it unchanged

#### Scenario: An invalid list
- **WHEN** a list is set that is empty, or that names a language twice
- **THEN** it is refused and the previous list stays

#### Scenario: Never synced
- **WHEN** a sync pushes and pulls
- **THEN** no request or response carries the list

### Requirement: A restore reads the backup's schema version first
A restore SHALL read the backup's schema version before the rest of the file, SHALL read schema versions 1 and 2, and SHALL refuse any other version as unsupported, naming the version found. When a restore is refused, the extension SHALL not start the surface that asked for it and SHALL leave the stored backup unchanged. A build released before schema version 2 existed refuses a version 2 backup: as malformed when the backup holds another language's records, as unsupported otherwise.

#### Scenario: Both known versions
- **WHEN** a version 1 backup and a version 2 backup are restored
- **THEN** both are read

#### Scenario: A later version
- **WHEN** a backup of schema version 3 is restored
- **THEN** the restore fails as unsupported version 3, and the stored backup is unchanged

#### Scenario: A build released before version 2
- **WHEN** a build released before this change opens a store whose backup has schema version 2 and Spanish cards
- **THEN** its restore fails as malformed, its reader does not start, and the backup is still there for a later build
