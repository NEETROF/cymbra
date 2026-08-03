## ADDED Requirements

### Requirement: Soundfont review queue and decision in the back office

The back office SHALL present a soundfont moderation surface, restricted to music-scope
`moderator`/`admin` identities, that lists catalog soundfonts with their moderation
status and offers a **pending review queue**. For a selected font the moderator SHALL be
able to **audition** it (play it, using the moderator's privilege to fetch bytes of a
non-`accepted` font) before deciding, and then **accept** or **reject** it, which invokes
the audited `SetSoundFontModerationStatus` operation. A non-`moderator`/non-`admin`
identity MUST NOT reach this surface or perform its actions.

#### Scenario: Pending fonts form the review queue

- **WHEN** a moderator opens the soundfont review queue
- **THEN** `pending` fonts are listed as the work to review, each showing its status, licence, attribution, and uploader

#### Scenario: Moderator auditions before deciding

- **WHEN** a moderator selects a `pending` font
- **THEN** they can play it (its bytes are served despite it being unvalidated) before choosing accept or reject

#### Scenario: Accept or reject records the decision

- **WHEN** a moderator accepts or rejects a font
- **THEN** its status is updated and `reviewed_by`/`reviewed_at` record who decided and when

#### Scenario: Unauthorized user cannot review soundfonts

- **WHEN** a signed-in user without `moderator`/`admin` opens the soundfont surface
- **THEN** it shows an access-denied state and its accept/reject calls are rejected
