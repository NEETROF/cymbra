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

