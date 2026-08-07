# soundfont-moderation Specification

## Purpose
TBD - created by archiving change add-soundfont-moderation. Update Purpose after archive.
## Requirements
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
SHALL, in the same update, record `reviewed_by` and `reviewed_at`. When the decision is
`rejected`, the operation SHALL accept and record a **rejection reason** (the moderator's
motive), surfaced back to the uploader; setting a status other than `rejected` clears any
stored reason. A non-`moderator`/non-`admin` caller MUST be rejected with
`PERMISSION_DENIED`. Setting the status of a non-existent soundfont MUST be rejected. A
`rejected` font SHALL keep its row and stored object (audit trail); it is merely un-served
and un-listed.

#### Scenario: Moderator accepts a font

- **WHEN** a moderator sets a `pending` font to `accepted`
- **THEN** the font becomes publicly visible and records that moderator and the time

#### Scenario: Moderator rejects a font with a reason, traceably

- **WHEN** a moderator sets a font to `rejected` with a reason
- **THEN** the font becomes `rejected`, its row and object are retained, its
  `reviewed_by`/`reviewed_at` identify who rejected it and when, and the rejection reason is
  recorded for the uploader

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

### Requirement: Uploader pseudo on the privileged soundfont read

The backend SHALL, on the **privileged** (music-scope moderator/admin) soundfont listing,
resolve each catalog soundfont's recorded `uploaded_by` to the uploader's **pseudo**
(display name) through the user-directory seam, so the soundfont review queue identifies
the contributor by name rather than by raw id. The uploader's user **id** and this
privileged pseudo field MUST NOT be exposed to a normal (non-moderator/non-admin) caller
through the public listing or the delivery route. The pseudo SHALL be resolved at read
time (not stored denormalised), so a later rename is reflected. A pre-existing catalog row
with no `uploaded_by` (seeded fonts) SHALL simply carry no uploader pseudo.

#### Scenario: Reviewer sees the uploader pseudo

- **WHEN** a moderator/admin lists the soundfont catalog for review
- **THEN** each font with a recorded uploader shows that uploader's pseudo, regardless of
  the uploader's profile visibility

#### Scenario: Uploader identity hidden from normal callers

- **WHEN** a normal caller lists the public soundfont catalog or fetches a font's bytes
- **THEN** no `uploaded_by` id and no privileged uploader-pseudo field are returned

### Requirement: Public contributor credit on an accepted soundfont, gated on public-profile opt-in

The system SHALL surface a **public contributor credit** (the uploader's public
handle/display name) on the public listing and delivery of an `accepted` user-contributed
soundfont, but **only** when the uploader has opted into a **public** profile; when the
uploader's profile is private (the default), carries no handle/display name, or cannot be
resolved, the credit MUST be omitted (fail-closed) and the font is still listed/served. This is distinct from
the font's **licence attribution** (the sample author, which is unchanged): the licence
attribution always shows, the contributor credit is the opt-in "proposé par @pseudo". A
bundled/seeded font with no `uploaded_by` carries no contributor credit. The raw
`uploaded_by` id MUST NOT be exposed on the public path.

#### Scenario: Public profile yields a soundfont credit

- **WHEN** a normal caller lists an `accepted` user-contributed font whose uploader has a
  public profile
- **THEN** the font carries a public contributor credit with the uploader's handle/display
  name, alongside (not replacing) its licence attribution, and no `uploaded_by` id

#### Scenario: Private profile yields no soundfont credit

- **WHEN** the uploader's profile is private, unresolvable, or handle-less
- **THEN** no contributor credit is included and the font is still listed/served

#### Scenario: Seeded font has no contributor credit

- **WHEN** a normal caller lists a seeded font that has no recorded uploader
- **THEN** it shows its licence attribution and no contributor credit

### Requirement: Rejected soundfont records and surfaces its reason

When a moderator rejects a catalog soundfont, the system SHALL record a **rejection
reason** (a moderator-supplied motive) on the soundfont, and SHALL surface that reason back
to the uploader through the private-library proposal status, so the uploader knows why it
was refused. A `pending` or `accepted` font carries no rejection reason.

#### Scenario: Rejection records a reason

- **WHEN** a moderator rejects a catalog soundfont
- **THEN** the soundfont stores the moderator's rejection reason

#### Scenario: Uploader sees why it was rejected

- **WHEN** the uploader views the proposal status of their private font that was rejected
- **THEN** the app shows the `rejected` state together with the moderator's rejection reason

### Requirement: Re-proposing a rejected soundfont reopens its row and requires a justification

The system SHALL, when a private font whose catalog entry is `rejected` is re-proposed,
**reopen that existing entry** — transition it back to `pending`, re-attribute it to the
current uploader (`uploaded_by`), and clear its prior rejection reason — rather than create
a second entry. The re-proposal MUST carry a non-empty **justification**; a re-proposal of
a `rejected` font without a justification MUST be refused and nothing changes. The
justification SHALL be recorded and surfaced to the moderator on re-review. A **first**
proposal (no prior rejected entry) SHALL NOT require a justification.

#### Scenario: Re-proposing a rejected font reopens the row

- **WHEN** an uploader re-proposes a private font whose catalog entry is `rejected`, with a
  justification
- **THEN** that same catalog entry returns to `pending`, is re-attributed to the uploader,
  its prior rejection reason is cleared, and no second entry is created

#### Scenario: Re-proposal without a justification is refused

- **WHEN** an uploader re-proposes a `rejected` font without a justification
- **THEN** the proposal is refused and the entry stays `rejected`

#### Scenario: First proposal needs no justification

- **WHEN** an uploader proposes a font that has never been in the catalog
- **THEN** the proposal succeeds without a justification (subject to licence + attestation)

