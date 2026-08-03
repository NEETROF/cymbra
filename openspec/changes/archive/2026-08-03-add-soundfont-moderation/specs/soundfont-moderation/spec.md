## ADDED Requirements

### Requirement: Moderation status on catalog soundfonts

Every public catalog soundfont (`music.soundfonts`) SHALL carry a moderation status
drawn from the fixed set `pending` / `accepted` / `rejected`, defaulting to `pending`.
`accepted` means the font is validated and publicly visible; `pending` means it has not
yet been reviewed; `rejected` means a reviewer has refused it. The store SHALL persist,
per entry, the reviewer who last set the status (`reviewed_by`) and when
(`reviewed_at`) — nullable and unset while `pending` — and the uploader
(`uploaded_by`). The status SHALL be constrained to the allowed set at the storage
layer so no other value can be persisted.

#### Scenario: Status defaults to pending

- **WHEN** a catalog soundfont is created without an explicit moderation status
- **THEN** its moderation status is `pending` and its reviewer/timestamp are unset

#### Scenario: Only the allowed status values are accepted

- **WHEN** a write attempts to set a moderation status outside `pending` / `accepted` / `rejected`
- **THEN** the store rejects it rather than persisting an unknown value

#### Scenario: Uploader and review attribution are recorded

- **WHEN** a soundfont is uploaded and later set to `accepted` or `rejected`
- **THEN** the entry records who uploaded it (`uploaded_by`) and, once decided, which
  reviewer set the status and at what time; a `pending` entry has no reviewer or timestamp

### Requirement: Only validated soundfonts are publicly visible

The system SHALL treat `accepted` as the only publicly visible moderation status through
the public read paths: a soundfont that is `pending` or `rejected` MUST NOT be exposed to
a normal (non-admin, non-moderator) caller through the **catalog listing**
(`ListSoundFonts`) or the **delivery route** (`GET /soundfonts/:id`). Visibility SHALL be
enforced server-side, so a client unaware of the status still cannot reach an unvalidated
font. A caller authorised as a music-scope moderator or admin SHALL be able to list and
fetch the bytes of soundfonts in any moderation status, so a reviewer can audition
unvalidated material.

#### Scenario: Pending font hidden from a normal caller

- **WHEN** a normal caller lists the catalog or requests the bytes of a `pending` font
- **THEN** the font is not returned (absent from the listing, or its bytes refused as if not found)

#### Scenario: Rejected font hidden from a normal caller

- **WHEN** a normal caller would otherwise encounter a `rejected` font
- **THEN** the font is not returned through any public read path

#### Scenario: Accepted font is visible

- **WHEN** a normal caller lists the catalog
- **THEN** `accepted` fonts are returned and downloadable

#### Scenario: Authorized reviewer sees unvalidated fonts

- **WHEN** a moderator/admin caller lists or fetches the bytes of a `pending` or `rejected` font
- **THEN** the font is returned and its bytes can be fetched, so the reviewer can audition it

### Requirement: Upload status branches on uploader role

When a soundfont enters the public catalog via upload, the server SHALL derive its
initial moderation status from the uploader's role rather than from any client-supplied
value: an `admin` identity's upload SHALL be recorded `accepted`; any other authenticated
identity's upload (moderator or plain user) SHALL be recorded `pending`. The client SHALL
NOT be able to set the status directly. The uploader SHALL be recorded as `uploaded_by`.

#### Scenario: Admin upload is auto-accepted

- **WHEN** an `admin` identity uploads a soundfont to the catalog
- **THEN** the font is recorded `accepted` and immediately publicly visible

#### Scenario: Moderator or user upload is pending

- **WHEN** a moderator or a plain user contributes a soundfont to the catalog
- **THEN** the font is recorded `pending` and is not publicly visible until reviewed

#### Scenario: Client cannot self-assign a status

- **WHEN** an upload request attempts to set its own moderation status
- **THEN** the server ignores it and derives the status from the uploader's role

### Requirement: Evaluate a soundfont's moderation status with audit

The backend SHALL expose an authenticated operation (`SetSoundFontModerationStatus`),
restricted to music-scope `moderator`/`admin` identities, that sets a catalog soundfont's
status to `accepted` or `rejected` (and MAY set it back to `pending`). The operation
SHALL, in the same update, record `reviewed_by` and `reviewed_at`. A
non-`moderator`/non-`admin` caller MUST be rejected with `PERMISSION_DENIED`. Setting the
status of a non-existent soundfont MUST be rejected. A `rejected` font SHALL keep its row
and stored object (audit trail); it is merely un-served and un-listed.

#### Scenario: Moderator accepts a font

- **WHEN** a moderator sets a `pending` font to `accepted`
- **THEN** the font becomes publicly visible and records that moderator and the time

#### Scenario: Moderator rejects a font, traceably

- **WHEN** a moderator sets a font to `rejected`
- **THEN** the font becomes `rejected`, its row and object are retained, and its
  `reviewed_by`/`reviewed_at` identify who rejected it and when

#### Scenario: Unauthorized evaluate rejected

- **WHEN** a caller without `moderator`/`admin` invokes the operation
- **THEN** it is rejected with `PERMISSION_DENIED` and no status changes

### Requirement: Existing catalog soundfonts are seeded on migration

Introducing moderation SHALL seed existing rows: on migration, the bundled default font
(`upright-piano-kw`, which also ships inside the app) SHALL become `accepted`, and every
other pre-existing catalog row SHALL become `pending`, so no previously listed font
remains publicly visible until explicitly validated. The migration MUST be additive and
reversible at the schema level.

#### Scenario: Bundled default is pre-accepted

- **WHEN** the moderation migration is applied
- **THEN** `upright-piano-kw` is `accepted` and remains available out of the box

#### Scenario: Other existing fonts become pending

- **WHEN** the migration is applied to a catalog with other rows
- **THEN** those rows become `pending` and drop out of the public listing until reviewed

### Requirement: Identical soundfont content is detected across uploads

The system SHALL compute an exact-byte content digest (SHA-256) of each uploaded
soundfont and record it on the catalog entry. Before adding a font to the public
catalog, the server SHALL check this digest so that byte-identical content is not stored
or listed twice: an upload or proposal whose bytes match a non-`rejected` catalog entry
MUST be refused as a duplicate, reporting the existing font's id, rather than creating a
second entry. Content identity — not only the client-facing id — SHALL be the uniqueness
guard.

#### Scenario: Byte-identical upload is refused as duplicate

- **WHEN** a soundfont whose bytes match an existing non-`rejected` catalog font is uploaded or proposed
- **THEN** the request is refused as a duplicate and identifies the existing font, and no second entry is created

#### Scenario: Different content is accepted

- **WHEN** a soundfont whose bytes differ from every catalog entry is contributed
- **THEN** it is admitted (subject to the role-based status branching) with its own content digest recorded
