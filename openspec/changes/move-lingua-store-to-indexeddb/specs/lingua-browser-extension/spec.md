## MODIFIED Requirements

### Requirement: Versioned local state

The reader's state — word statuses, cards and their review history, declared level and calibration, daily statistics, and the cursors that describe what has been synchronised — SHALL live in a browser store whose capacity grows with the reader (IndexedDB), under a versioned schema with forward migration. Preferences, session tokens and transient marks MAY stay in the extension's settings storage.

One context SHALL own that store; every surface SHALL read and write it through the same seam, and SHALL be told which of its keys changed, so all surfaces show the same state. A store that cannot be opened SHALL NOT take the extension down: it SHALL fall back to the settings storage and keep working. Moving state from a previous store SHALL lose nothing, and SHALL leave the previous copy readable by the build that wrote it. A full reset SHALL be offered.

#### Scenario: Schema migration
- **WHEN** the extension starts on state from an earlier version
- **THEN** the state is migrated without loss and the stored version is updated

#### Scenario: Moving to the store that grows
- **WHEN** the extension starts for the first time after the state's home changes
- **THEN** every piece of the reader's state is readable from the new store, and the previous copy is still there for the build that wrote it

#### Scenario: A reader who reads a great deal
- **WHEN** the reader's state grows past what the settings storage would have accepted
- **THEN** marking a word, reviewing, synchronising and erasing all keep working

#### Scenario: Surfaces follow the same state
- **WHEN** a word is marked in the page while the panel is open
- **THEN** the panel shows the change without being reopened

#### Scenario: The store cannot be opened
- **WHEN** the browser refuses to open the store
- **THEN** the extension keeps reading, marking and reviewing on the settings storage
