## ADDED Requirements

### Requirement: Score audio preview rendered at acceptance

The server SHALL, when a catalog score is **accepted** (the moderation transition
that makes it playable), render a short **audio-only** preview clip by
synthesizing the first bounded portion (~N seconds) of the piece with a default
soundfont, using the backend headless render engine, and store it as a **public**
object under a score-preview key (e.g. `score-preview/{catalogId}.wav`), distinct
from the score's MusicXML. The MusicXML→render-sequence conversion (bounded to N
seconds) SHALL be a host-testable helper. A render failure SHALL NOT block the
acceptance — it is logged and the preview is left absent, recoverable via the
back-office regenerate action.

#### Scenario: Acceptance renders and stores a public audio preview
- **WHEN** a catalog score is accepted
- **THEN** a public audio-only preview clip for that piece is rendered and stored

#### Scenario: Render failure does not block acceptance
- **WHEN** the preview render fails for an accepted piece
- **THEN** the acceptance still completes and no preview object is stored (recoverable)

### Requirement: Back-office preview regeneration

The server SHALL expose an **admin-gated** action that re-renders a catalog piece's
audio preview from its stored content and overwrites the public preview object,
returning success or failure. The back-office score admin screen SHALL provide a
**"Generate sample"** action that calls it through the injectable client seam and
reflects its in-flight / success / error state.

#### Scenario: Admin regenerates a missing preview
- **WHEN** an admin invokes "Generate sample" for an accepted piece with no preview (accepted before this change)
- **THEN** the server renders and stores the preview and the action reports success

#### Scenario: Non-admin cannot regenerate
- **WHEN** a non-admin caller invokes the regenerate action
- **THEN** the request is refused

### Requirement: Admin can find pieces missing a preview

The admin score listing SHALL expose whether each catalog piece has an audio
preview object, and the back-office score screen SHALL provide a **filter** to show
only the pieces **without** a preview, so an admin can find and backfill them (via
"Generate sample"). The missing-preview state SHALL be derivable for the listing
without downloading the clips.

#### Scenario: Filtering to pieces without a preview
- **WHEN** an admin enables the "no sample" filter on the score screen
- **THEN** only accepted pieces that have no preview object are listed

#### Scenario: A regenerated piece leaves the filter
- **WHEN** an admin generates the sample for a listed piece and re-applies the filter
- **THEN** that piece no longer appears among the pieces without a preview

### Requirement: Public preview delivery and locked-piece audition

The server SHALL serve a piece's audio preview at a dedicated RPC (e.g.
`GetScorePreviewBytes(catalogId)`) with **no daily-quota or points gate** (hearing
the preview is the purpose), applying only moderation visibility. If no preview
object exists, it SHALL respond not-found. A **locked** catalog piece (over quota,
unpaid, unsubscribed) SHALL never deliver its MusicXML; instead the app's play
control SHALL audition this audio preview clip. When no preview exists, the app
SHALL disable/grey the audition control rather than erroring.

#### Scenario: A locked piece is auditionable via the audio clip
- **WHEN** a user taps play on a locked catalog piece
- **THEN** the app plays the server-rendered audio preview clip
- **AND** the piece's MusicXML is never delivered to audition it

#### Scenario: Preview is served regardless of access state
- **WHEN** any user who may view an accepted piece requests its preview
- **THEN** the server serves the audio clip whether or not the user can fully open the piece

#### Scenario: Absent preview degrades gracefully
- **WHEN** a piece has no preview object yet
- **THEN** the preview RPC responds not-found and the app disables the audition control
