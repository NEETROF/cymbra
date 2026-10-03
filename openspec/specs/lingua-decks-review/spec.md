# lingua-decks-review Specification

## Purpose
TBD - created by archiving change add-lingua-decks-review. Update Purpose after archive.
## Requirements
### Requirement: Card schema with provenance
A card SHALL carry at minimum: the lemma, the encountered form, the originating context
sentence, the source of the encounter (URL or agent session identifier, timestamp), the
gloss, and an optional media slot (`media`, with `source: capture|stock|generated` and
`sync_policy`) — not populated in this change but present in the schema. Multi-word
expressions SHALL be cards in their own right.

#### Scenario: Card created from the popup
- **WHEN** the user adds `conundrum` to the deck from a web page
- **THEN** the card holds the lemma, the complete originating sentence and the page URL

#### Scenario: Expression card
- **WHEN** the user captures the selection "compelling starting point" with the keyboard shortcut
- **THEN** an expression card is created with the originating sentence

### Requirement: FSRS review scheduling
Review SHALL be scheduled by FSRS in the core: each card carries its own state
(stability, difficulty, due date), and the four answers (`again`, `hard`, `good`, `easy`)
SHALL update that state and the due date. The number of due cards SHALL be computable at
any moment.

#### Scenario: A "good" answer pushes the due date out
- **WHEN** a due card is graded `good`
- **THEN** its due date becomes strictly later than now and its FSRS state is updated

### Requirement: Moving to known from review
Marking a card "I know this" during review SHALL move the lemma to the `known` status
(provenance `srs`) and remove it from the due cards, without deleting the card or its
history.

#### Scenario: Word learned
- **WHEN** the user answers "I know this" on the `seldom` card
- **THEN** `seldom` moves to the `known` status (provenance `srs`) and the card leaves the review queue

### Requirement: Lossless backup and restore
The system SHALL export the complete state (cards with all their fields — lemma,
encountered form, sentence, gloss, source, status, due date, FSRS state — word statuses,
calibration, FSRS parameters) to a versioned backup file, and SHALL restore that file
identically. No populated field SHALL be omitted from the backup; a restore onto a clean
state SHALL reproduce the original state (a lossless round trip). The card schema SHALL
stay entirely serialisable field by field, so that an Anki-format export (deferred to a
later change) remains a plain serialiser.

#### Scenario: Backup round trip
- **WHEN** the user backs up a state of 20 cards and 300 statuses, then restores it onto a fresh install
- **THEN** the restored state is identical to the original — cards, statuses, calibration and review due dates included

### Requirement: Review present right next to the reading
Review SHALL be reachable without leaving the browser: the due-card count SHALL be
visible in the icon popup and in the side panel, and a review session SHALL be
launchable from the side panel or the injected panel. A card's answer SHALL stay hidden
until an explicit reveal action.

#### Scenario: Micro-session from the side panel
- **WHEN** the user opens the side panel and starts a review with 3 cards due (French UI copy: « Réviser maintenant »)
- **THEN** the cards come one by one, answer hidden then revealed, and the due count goes down

### Requirement: Level-targeted deck seeding
The deck SHALL support seeding cards for a chosen set of lemmas — typically the lemmas of a
selected CEFR level (or frequency band) — without a real web encounter. Seeded cards SHALL
record the reserved `import` encounter source rather than fabricating a URL or an agent
session. Seeding SHALL be capped per operation, SHALL skip lemmas that already have a card
or an explicit status (idempotent), and SHALL let the caller choose the order in which
lemmas are taken from the level (commonest-first by default).

#### Scenario: Seed a level into the deck
- **WHEN** the user asks to add 20 words of level B2 with 5 of them already tracked
- **THEN** at most 15 new cards are created, each with encounter source `import`, and no already-tracked lemma is duplicated

#### Scenario: Cap respected
- **WHEN** the user requests more words than the per-operation cap allows
- **THEN** only up to the cap are seeded and the caller is told how many were added

### Requirement: The backup records the reader's language profile
The backup SHALL record the reader's language profile: their studied languages, an ordered list, non-empty and without duplicates, whose first language is the primary one, and their native language. The default profile is English studied, French native. A backup whose profile is the default and whose every per-language record is English SHALL be written with schema version 1 and the exact content it had before the profile was stored. Any other backup SHALL be written with schema version 2, the profile included. A full reset SHALL return the profile to the default, resetting statuses SHALL leave it unchanged, and the profile SHALL never be sent to the server.

#### Scenario: An English reader's backup does not change
- **WHEN** an English reader with 20 cards and 300 statuses is backed up
- **THEN** the backup has schema version 1, carries no profile, and is byte for byte what the previous build wrote

#### Scenario: Another studied language
- **WHEN** the studied languages are set to Spanish then English, and the state is backed up and restored
- **THEN** the backup has schema version 2, and the restored studied languages are Spanish then English

#### Scenario: Another language in the records only
- **WHEN** the profile is the default but the deck holds a Spanish card
- **THEN** the backup has schema version 2

#### Scenario: Resets
- **WHEN** the reader erases their data
- **THEN** the profile is the default again, whereas resetting statuses leaves it unchanged

#### Scenario: An invalid choice of studied languages
- **WHEN** studied languages are set as an empty list, or as a list naming a language twice
- **THEN** they are refused and the previous studied languages stay

#### Scenario: Never synced
- **WHEN** a sync pushes and pulls
- **THEN** no request or response carries the profile

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

