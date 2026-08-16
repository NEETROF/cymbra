# moderation-console Specification

## Purpose

Defines the moderation back office: the operation to evaluate a score's moderation
status with audit, the catalog table and prioritized review queue, the read-only
score preview, the per-row MusicXML download, and how the back-office web application
is delivered and access-gated.
## Requirements
### Requirement: Evaluate a score's moderation status with audit

The backend SHALL expose an authenticated operation, restricted to `moderator`/`admin`
identities, that sets a catalog score's moderation status to `accepted` or `rejected`
(and MAY set it back to `pending` to re-queue). The operation SHALL, in the same update,
record the reviewing account (`reviewed_by`) and the review time (`reviewed_at`), so a
rejection is traceable to a specific moderator and moment. When the decision is
`rejected`, the operation SHALL accept and record a **rejection reason** (the moderator's
motive), which is surfaced back to the proposer of a user-proposed score; setting a status
other than `rejected` clears any stored reason. A non-`moderator`/non-`admin` caller MUST
be rejected with `PERMISSION_DENIED`. Setting the status of a non-existent score MUST be
rejected.

#### Scenario: Moderator accepts a score

- **WHEN** a moderator sets a `pending` score to `accepted`
- **THEN** the score becomes `accepted` (visible in the hub) and records that moderator and the time

#### Scenario: Moderator rejects a score with a reason, traceably

- **WHEN** a moderator sets a score to `rejected` with a reason
- **THEN** the score becomes `rejected`, its `reviewed_by`/`reviewed_at` identify who
  rejected it and when, and the rejection reason is recorded for the proposer

#### Scenario: Re-queue to pending

- **WHEN** a moderator sets an `accepted` score back to `pending`
- **THEN** the score leaves the hub and returns to the review queue

#### Scenario: Unauthorized evaluate rejected

- **WHEN** a caller without `moderator`/`admin` invokes the evaluate operation
- **THEN** it is rejected with `PERMISSION_DENIED` and no status changes

### Requirement: Back-office catalog table with hub filters and a privileged status filter

The back office SHALL present all catalog scores in a simple table, filterable by the
**same filters available in the app hub** (free-text title/composer, author, difficulty,
and the musical facets), plus a **back-office-only moderation-status filter** that is
never exposed in the Flutter app and that only `moderator`/`admin` identities may use.
The table SHALL show each score's key fields including its moderation status. Selecting a
row SHALL open the score for review.

#### Scenario: Table lists scores with the hub filters

- **WHEN** a moderator filters the table by text, author, difficulty, or a facet
- **THEN** the results match those filters exactly as the app hub would, but across all statuses the moderator is allowed to see

#### Scenario: Status filter is back-office only

- **WHEN** a moderator filters the table by moderation status (e.g. `pending`)
- **THEN** only scores of that status are listed, using the privileged filter available only to moderators/admins

#### Scenario: Row shows status and opens for review

- **WHEN** a moderator selects a table row
- **THEN** the score opens for review showing its current moderation status

### Requirement: Review queue prioritizes work by review priority

The back office SHALL offer a default queue view that orders the work to do by a **review
priority**. The queue SHALL surface **first** the scores most in need of attention:
unreviewed (`pending`) scores and `accepted` scores flagged for re-review by the app
rating signal (when that signal is available). Re-review-flagged scores MUST be reachable
prominently — surfaced at the top of the queue and available as a dedicated filter — so a
moderator can jump straight to them.

Within that work set, the default ordering SHALL further prioritize the **most
substantial scores** — those with more musical content are presented first, so the most
relevant scores are reviewed before thin or trivial ones. Substance SHALL be derived from
the score's own musical facets (for example its length in measures, its staff count, and
the presence of richer musical features such as chords, tuplets, and dotted rhythms);
scores with more of these rank higher.

The table SHALL let a moderator **sort and filter** by these dimensions (moderation
status, re-review flag, and the substance/facet fields), so the default priority order can
be overridden on demand. When the rating signal is absent, the queue SHALL still function
on `pending` scores alone, ordered by substance.

#### Scenario: Flagged and pending scores lead the queue

- **WHEN** a moderator opens the queue view
- **THEN** `pending` scores and re-review-flagged `accepted` scores are presented first as
  the primary work to review

#### Scenario: Re-review-flagged scores are reachable prominently

- **WHEN** `accepted` scores have been flagged for re-review by ratings
- **THEN** they appear at the top of the queue and can also be isolated with a dedicated
  re-review filter

#### Scenario: Richer scores are reviewed first

- **WHEN** the queue lists several scores of the same review status
- **THEN** scores with more musical content (more measures, more staves, richer features)
  are ordered ahead of thinner ones

#### Scenario: Moderator overrides the order by sorting/filtering

- **WHEN** a moderator sorts or filters the table by status, the re-review flag, or a
  substance/facet field
- **THEN** the table reorders/narrows accordingly, overriding the default priority order

#### Scenario: Queue works without the rating signal

- **WHEN** the rating re-review signal is not yet available
- **THEN** the queue still lists `pending` scores to review, ordered by substance

### Requirement: Read-only score preview in the back office

Before deciding, a moderator SHALL be able to preview the selected score — its metadata
and its notation — in a **read-only** view, using score bytes served by the backend
(moderators may fetch bytes of non-`accepted` scores). The notation SHALL be rendered
faithfully to how the app renders it, so the moderator judges the score as users will
see it. The preview MUST NOT allow editing the score.

#### Scenario: Moderator previews before deciding

- **WHEN** a moderator opens a `pending` score in the console
- **THEN** its metadata and notation are shown read-only so the moderator can judge it

#### Scenario: Preview does not modify the score

- **WHEN** a moderator previews a score
- **THEN** no change is made to the score until an explicit accept/reject action

### Requirement: Back-office web application delivery

The back office SHALL be delivered as a client-rendered single-page web application
served at the back-office origin (`bo.cymbra.app`), authenticating through Cymbra
sign-in and calling the backend over the browser gRPC surface. Access to the application
SHALL be limited to `moderator`/`admin` identities: a signed-in user without those roles
MUST NOT be able to perform moderation actions, and MUST be shown an access-denied state.

#### Scenario: Authorized user reaches the console

- **WHEN** a `music/moderator` or `music/admin` signs in at the back-office origin
- **THEN** the console loads and its moderation actions are available

#### Scenario: Unauthorized signed-in user is denied

- **WHEN** a signed-in user without `moderator`/`admin` opens the back-office origin
- **THEN** the console shows an access-denied state and the moderation operations reject their calls

#### Scenario: Unauthenticated visitor cannot use it

- **WHEN** an unauthenticated visitor opens the back-office origin
- **THEN** they are prompted to sign in and cannot perform any moderation action

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

### Requirement: Review queue distinguishes and attributes user-proposed scores

The back-office review queue and catalog table SHALL visually **distinguish** a
user-proposed score from a crawler-ingested one (an origin indicator on the row), and for
a user-proposed score SHALL show the **proposer's pseudo** (display name), using the
privileged proposer fields the backend exposes only to `moderator`/`admin` reads. A
crawler-ingested score SHALL show its dataset origin as before and no proposer pseudo. The
proposer's identity MUST come from the privileged read only, so it never appears in any
surface available to a non-`moderator`/non-`admin` caller.

#### Scenario: User-proposed row shows origin and proposer

- **WHEN** a moderator views the queue and a row is a user-proposed score
- **THEN** the row is marked as a user proposal and shows the proposer's pseudo

#### Scenario: Crawler row is distinguished from a proposal

- **WHEN** a moderator views the queue and a row is a crawler-ingested score
- **THEN** the row shows its dataset origin and no proposer pseudo

#### Scenario: Proposer pseudo is not shown outside the console

- **WHEN** the same catalog is read by a non-`moderator`/non-`admin` caller
- **THEN** no proposer pseudo or id is present in the response

### Requirement: Catalog table defaults to all statuses and filters by source

The back-office catalog table SHALL default its moderation-status filter to **all
statuses** (a "Tous" option showing `pending` + `accepted` + `rejected` together), rather
than a single status, so a moderator sees the whole corpus by default; selecting a
specific status still narrows to it. The table SHALL also offer a **source filter** so the
moderator can restrict to an origin (e.g. user proposals vs a crawler dataset). Each row
SHALL show its own moderation status and its source, and — for a user-proposed row — the
**proposer's pseudo**. The score **detail** view SHALL likewise show the score's source
(and the proposer's pseudo when user-proposed). Both the all-statuses read and the source
filter are privileged (moderator/admin), authorised server-side.

#### Scenario: Catalog defaults to every status

- **WHEN** a moderator opens the catalog table without choosing a status
- **THEN** scores of every moderation status are listed (the "Tous" default)

#### Scenario: Filtering by source narrows the origin

- **WHEN** a moderator selects the user-proposal source
- **THEN** only user-proposed scores are listed

#### Scenario: Detail shows the source and proposer

- **WHEN** a moderator opens a user-proposed score's detail
- **THEN** its source and the proposer's pseudo are shown

### Requirement: Review queue surfaces a re-proposal justification

When a reopened (re-proposed) score appears in the review queue, the back office SHALL
show the proposer's **resubmission justification** so the moderator sees why it is being
resubmitted after a prior rejection. A score that has never been rejected/re-proposed
SHALL show no such justification.

#### Scenario: Reopened score shows its resubmission justification

- **WHEN** a moderator views a reopened score in the queue
- **THEN** the proposer's resubmission justification is shown

### Requirement: Download a catalog score's MusicXML from the back office

The back-office catalog table SHALL provide, per score row, a control that downloads the
score's canonical MusicXML file to the operator's local machine. Activating the control
SHALL fetch the score's decoded MusicXML bytes via the existing byte-serving operation and
save them as a file whose name is derived from the score (its title when available, else
its identifier) with a `.musicxml` extension. The download SHALL NOT require navigating to
the score detail view.

#### Scenario: Moderator downloads a score's MusicXML from the catalog table

- **WHEN** a `music` moderator/admin activates the download control on a catalog row
- **THEN** the score's canonical MusicXML bytes are fetched and the browser saves a
  `<title-or-id>.musicxml` file to the operator's machine

#### Scenario: File name falls back to the identifier

- **WHEN** the score has no usable title
- **THEN** the downloaded file is named from the score's identifier with a `.musicxml`
  extension

#### Scenario: Downloaded bytes are the decoded MusicXML

- **WHEN** the stored object is a compressed `.mxl`
- **THEN** the served bytes are the decompressed canonical MusicXML and the file is saved
  with the `.musicxml` extension (not `.mxl`)

### Requirement: Score download is authorized to back-office moderators/admins only

The download SHALL be served only to an authenticated back-office operator holding a
`music` `moderator` or `admin` role (or `global/admin`) — the same authorization that
already permits fetching the bytes of a score in any moderation status. The download
control SHALL be rendered only for such operators, and the byte-serving path SHALL reject
an unauthorized caller. This provenance check reuses the existing byte-serving guard; no
new unauthenticated or public download path is introduced.

#### Scenario: Moderator may download any-status score

- **WHEN** a `music` moderator/admin downloads a `pending`, `rejected`, or `accepted`
  score
- **THEN** the bytes are served and the file is saved

#### Scenario: Download control hidden from non-moderators

- **WHEN** an operator without `moderator`/`admin` views the catalog table
- **THEN** the download control is not rendered

#### Scenario: Unauthorized byte request refused

- **WHEN** a caller without `moderator`/`admin` requests a non-`accepted` score's bytes
- **THEN** the request is refused (permission denied / not found), as for the existing
  byte-serving operation

### Requirement: Per-row download feedback is localized and non-blocking

Each row's download SHALL surface its own loading and error state without blocking the
rest of the table or the catalog browse experience. On failure — including a score whose
underlying object is not yet available — the operator SHALL see a localized error message;
a raw gRPC/exception string SHALL NOT be shown. A download in progress on one row SHALL
NOT prevent viewing, sorting, or downloading other rows.

#### Scenario: Download in progress shows per-row loading

- **WHEN** the operator activates the download control on a row
- **THEN** that row indicates the download is in progress while other rows remain
  interactive

#### Scenario: Missing object reports a localized error

- **WHEN** the score's underlying MusicXML object is not available
- **THEN** a localized error message is shown and no file is saved, with the raw
  technical error only logged, not displayed

#### Scenario: One row's failure does not break the table

- **WHEN** a download fails on one row
- **THEN** the rest of the catalog table stays usable and other rows can still be
  downloaded

