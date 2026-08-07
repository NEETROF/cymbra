## ADDED Requirements

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

## MODIFIED Requirements

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
