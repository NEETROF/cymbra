## ADDED Requirements

### Requirement: Score audio preview rendered by a job at acceptance

The server SHALL enqueue a preview-render job when a catalog score is **accepted**
(the moderation transition that makes it playable), **in the same transaction** as
the status change (the job exists iff the acceptance commits). The job SHALL render
a short **audio-only** clip by taking the piece's playback schedule from the shared
MusicXML crate (the same note timing the app uses), clipping it to a bounded duration
(a runtime-configurable maximum, held notes truncated at the boundary), synthesizing it
with a **configured default catalog SoundFont** through the backend headless render
engine, and storing the clip as an object beside the score bytes under a score-preview
key distinct from the MusicXML. On success it SHALL stamp a **rendered marker** on the
catalog row (the listing truth for "has a sample"). The schedule→sequence conversion
SHALL be a pure, deterministic, host-testable helper. A render failure SHALL NOT block
or revert the acceptance — the job retries per policy, then leaves the preview absent,
recoverable via regenerate/backfill. An unset or unknown preview font SHALL leave
previews absent (feature dormant), never fail the acceptance.

#### Scenario: Acceptance enqueues and the job stores a preview
- **WHEN** a catalog score is accepted
- **THEN** a render job is enqueued with the acceptance, and once it runs an audio-only preview clip for that piece is stored and the row is marked as rendered

#### Scenario: Acceptance rollback leaves no job
- **WHEN** the acceptance transaction fails
- **THEN** no preview-render job exists for that piece

#### Scenario: Render failure does not block acceptance
- **WHEN** the preview render fails for an accepted piece after its retries
- **THEN** the piece stays accepted and no preview object / rendered marker is stored (recoverable)

#### Scenario: Sequence is bounded and deterministic
- **WHEN** the same schedule is converted twice with the same maximum duration
- **THEN** both sequences are identical, end at or before the maximum, and contain no note starting after it

### Requirement: Back-office preview regeneration

The server SHALL expose an **admin/moderator-gated** action that re-renders a catalog
piece's audio preview from its stored content **inline**, overwrites the preview
object, stamps the rendered marker, and returns success or failure. The back-office
score screen SHALL provide a **"Generate sample"** action that calls it through the
injectable client seam and reflects its in-flight / success / error state.

#### Scenario: Admin regenerates a missing preview
- **WHEN** an admin invokes "Generate sample" for an accepted piece with no preview
- **THEN** the server renders and stores the preview, marks the row, and the action reports success

#### Scenario: Non-privileged caller cannot regenerate
- **WHEN** a caller who is neither admin nor moderator invokes the regenerate action
- **THEN** the request is refused

### Requirement: Admin can find and backfill pieces missing a preview

The admin score listing SHALL expose whether each catalog piece has an audio preview,
derived from the rendered marker (never by probing storage per row), and the
back-office score screen SHALL provide a **filter** to show only the pieces
**without** a preview. An ops backfill SHALL enqueue the render job for every accepted
piece without a rendered marker, so the already-accepted corpus is covered without
manual clicks; the filter is for spot checks.

#### Scenario: Filtering to pieces without a preview
- **WHEN** an admin enables the "no sample" filter on the score screen
- **THEN** only accepted pieces without a rendered marker are listed

#### Scenario: A regenerated piece leaves the filter
- **WHEN** an admin generates the sample for a listed piece and re-applies the filter
- **THEN** that piece no longer appears among the pieces without a preview

#### Scenario: Backfill covers the accepted corpus
- **WHEN** the backfill is run
- **THEN** a render job is enqueued for each accepted piece lacking a rendered marker, and none for pieces already rendered

### Requirement: Public preview delivery and locked-piece audition

The server SHALL serve a piece's audio preview over the same authenticated HTTP shape
as the SoundFont previews (`GET /scores/{catalog_id}/preview`) with **no daily-quota or
points gate** (hearing the preview is the purpose), applying only moderation
visibility (accepted for normal callers; any status for moderator/admin). If no
preview object exists, it SHALL respond not-found. The public catalog read SHALL
expose a `has_preview` flag so the app knows without a probe. A **locked** catalog
piece (over quota, unpaid, unsubscribed) SHALL never deliver its MusicXML; instead the
app's locked flow SHALL offer to audition this clip through the existing clip-player
seam. When no preview exists, the app SHALL disable/grey the audition control rather
than erroring.

#### Scenario: A locked piece is auditionable via the audio clip
- **WHEN** a user taps "listen" on a locked catalog piece
- **THEN** the app plays the server-rendered audio preview clip
- **AND** the piece's MusicXML is never delivered to audition it

#### Scenario: Preview is served regardless of access state
- **WHEN** any user who may view an accepted piece requests its preview
- **THEN** the server serves the audio clip whether or not the user can fully open the piece

#### Scenario: Pending piece preview is moderator-only
- **WHEN** a normal caller requests the preview of a pending piece
- **THEN** the request is not-found, while a moderator/admin is served

#### Scenario: Absent preview degrades gracefully
- **WHEN** a piece has no preview object yet
- **THEN** the catalog read says no preview, the preview route responds not-found, and the app disables the audition control

### Requirement: Audition from the catalog card, once per request

When a catalog piece has an audio teaser, the app SHALL offer to audition it
**directly from the piece's card** (hub and library) through a small, labelled
control distinct from the card's main tap — so a user can hear a piece **before**
spending a free open or points, and the main tap still opens the piece as before.
An audition SHALL play the clip **once** (never looped) and stop by itself at the
end; at most one clip SHALL sound at a time (starting another stops the current
one); opening a piece or leaving the screen SHALL stop any audition. Cards without a
teaser SHALL show no control.

#### Scenario: Card audition never consumes the quota
- **WHEN** a user taps the card's audition control on a piece with a teaser
- **THEN** the clip plays once and stops by itself, and neither a free open nor points are consumed

#### Scenario: The main tap still opens the piece
- **WHEN** a user taps the card anywhere but its audition control
- **THEN** the piece opens (or the locked flow shows) exactly as before

#### Scenario: One clip at a time, stopped by opening
- **WHEN** a user starts a second card's audition, or opens a piece, while a clip is playing
- **THEN** the previous clip stops
