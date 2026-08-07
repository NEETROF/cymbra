## ADDED Requirements

### Requirement: Soundfont review queue shows the uploader pseudo

The back-office soundfont review queue SHALL show, for each user-contributed font, the
**uploader's pseudo** (display name) resolved from the recorded `uploaded_by` via the
privileged read, rather than a raw id. A seeded font with no recorded uploader SHALL show
no uploader pseudo. The uploader identity comes from the privileged read only and MUST NOT
appear in any non-`moderator`/non-`admin` surface.

#### Scenario: Soundfont row shows the uploader pseudo

- **WHEN** a moderator views the soundfont review queue and a font has a recorded uploader
- **THEN** the row shows that uploader's pseudo

#### Scenario: Seeded soundfont shows no uploader

- **WHEN** a moderator views a seeded font with no recorded uploader
- **THEN** the row shows no uploader pseudo

### Requirement: Soundfont review captures a rejection reason and surfaces resubmissions

When rejecting a soundfont in the back office, the moderator SHALL be able to record a
**rejection reason**, passed to the evaluate operation. When a reopened (re-proposed)
soundfont appears in the review queue, the back office SHALL show the uploader's
**resubmission justification**, so the moderator sees why it is being resubmitted after a
prior rejection. A font that has never been rejected/re-proposed SHALL show no such
justification.

#### Scenario: Reject captures a reason

- **WHEN** a moderator rejects a soundfont with a reason entered in the console
- **THEN** the reason is sent with the evaluate operation and recorded

#### Scenario: Reopened soundfont shows its resubmission justification

- **WHEN** a moderator views a reopened soundfont in the queue
- **THEN** the uploader's resubmission justification is shown
