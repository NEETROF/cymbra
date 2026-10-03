## ADDED Requirements

### Requirement: Each surface reads in the reader's language
Every surface of the extension SHALL ask the engine in its reading language: the first of the reader's studied languages that a shipped pair studies, or the default pair's language when none is. A surface SHALL read its reading language when it starts, and again after another context changes the stored backup. A store migrated from the reading-only format SHALL be migrated in English.

#### Scenario: Every reader today
- **WHEN** a reader who studies English alone opens a page, the side panel, the statistics or the settings
- **THEN** every request names English, as before

#### Scenario: A language the package does not ship
- **WHEN** the studied languages are Spanish then English and the package ships en-fr only
- **THEN** every surface reads in English, and the backup keeps the studied languages as they were

#### Scenario: The studied languages changed in another context
- **WHEN** the package ships en-fr and es-fr, a page is open in English, and another context stores a backup whose studied languages start with Spanish
- **THEN** the page's next requests name Spanish

#### Scenario: A reading-only store
- **WHEN** a store in the reading-only format is migrated
- **THEN** its statuses and cards are recorded under English
