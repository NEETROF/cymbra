## ADDED Requirements

### Requirement: Each document is read in its own language
When the reader accepts several languages, the reading session SHALL choose each document's language among them, with the document's declared language as a hint, and SHALL ask every language-bound question about that document in it: its analysis, glosses, cards, statuses and exposures. With one accepted language, it SHALL read every document in that language without asking for a detection.

#### Scenario: A Spanish page
- **WHEN** a reader who accepts English and Spanish opens a page in Spanish
- **THEN** the page is analysed and highlighted in Spanish, and a word marked there is recorded in Spanish

#### Scenario: An English page for the same reader
- **WHEN** the same reader opens a page in English
- **THEN** the page is read in English

#### Scenario: Every reader today
- **WHEN** a reader who accepts English alone opens any page
- **THEN** no detection is asked for, and every request names English, as before

## MODIFIED Requirements

### Requirement: Each surface reads in the reader's language
Every surface of the extension SHALL ask the engine in its reading language: the first of the reader's studied languages that a shipped pair studies, or the default pair's language when none is. A reading session SHALL instead ask about each document it reads in that document's language (see "Each document is read in its own language"). A surface SHALL read its reading language when it starts, and again after another context changes the stored backup. A store migrated from the reading-only format SHALL be migrated in English.

#### Scenario: Every reader today
- **WHEN** a reader who studies English alone opens a page, the side panel, the statistics or the settings
- **THEN** every request names English, as before

#### Scenario: A language the package does not ship
- **WHEN** the studied languages are Spanish then English and the package ships en-fr only
- **THEN** every surface reads in English, and the backup keeps the studied languages as they were

#### Scenario: The studied languages changed in another context
- **WHEN** the package ships en-fr and es-fr, the side panel is open, and another context stores a backup whose studied languages start with Spanish
- **THEN** the side panel's next requests name Spanish

#### Scenario: A reading-only store
- **WHEN** a store in the reading-only format is migrated
- **THEN** its statuses and cards are recorded under English
