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
The backup SHALL record the reader's language profile: their studied languages, an ordered list, non-empty and without duplicates, whose first language is the primary one, and their native language, which is never one of the studied languages. The default profile is English studied, French native. A backup whose profile is the default and whose every per-language record is English SHALL be written with schema version 1 and the exact content it had before the profile was stored. Any other backup SHALL be written with schema version 2, the profile included. A choice of studied languages that holds the native language SHALL be refused, and the native language SHALL be set only together with the studied languages, the whole choice being refused when it would study the native language; a refused choice SHALL leave the profile as it was. A full reset SHALL give the reader the engine's native language, studying the language of the engine's first pack alone; resetting statuses SHALL leave the profile unchanged. The profile SHALL never be sent to the server.

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

### Requirement: A review is in the language being read
A review session and its counts SHALL cover one studied language at a time, never several mixed. The review SHALL open in the language of the page or book it is opened beside, when the reader studies that language. Otherwise it SHALL open in the last language the reader chose in the review on this device, and otherwise in the reader's first studied language. A language the reader chooses in the review SHALL be remembered on the device as the last one chosen, and SHALL hold until the review is opened again or the page or book beside it changes; the review SHALL then open in that page's language again, and an open review SHALL follow the page or book beside it. The review's count of cards and of cards due SHALL be that language's, and the popup's count of cards to review SHALL follow the same language.

#### Scenario: Beside a Spanish page
- **WHEN** a reader of English and Spanish opens the review beside a page in Spanish
- **THEN** the review and its counts hold the Spanish cards only, and no choice of all languages is offered

#### Scenario: A book that opens on its cover
- **WHEN** the reader opens a Spanish book on its cover, or on front matter in English, beside the review
- **THEN** the review is in Spanish

#### Scenario: Away from a page
- **WHEN** the review opens beside no page in a studied language, and the reader last chose English in the review
- **THEN** it opens in English

#### Scenario: Choosing another language
- **WHEN** the reader chooses English in the review beside a Spanish page
- **THEN** the review and its counts switch to English, and English is remembered as the last language chosen

#### Scenario: Back from the statistics
- **WHEN** the reader chose English in the review beside a Spanish page, then shows the statistics and the review again
- **THEN** the review is still in English

#### Scenario: Another page or book
- **WHEN** the reader chose English in the review beside a Spanish page, then moves to another page or book in Spanish with the review open, or opens the review again
- **THEN** the review is in Spanish

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** no choice of language is shown, and the review and its counts are as before

