# soundfont-preview Specification

## Purpose
TBD - created by archiving change add-soundfont-entitlement-previews. Update Purpose after archive.
## Requirements
### Requirement: Server-rendered preview clip

The server SHALL be able to render a short **preview clip** for a SoundFont by
synthesizing a **fixed sample sequence** (the same short musical phrase for every
font) with that font, headlessly (no audio device), to a PCM buffer, then encoding
it to a compact, universally playable audio container. The synthesis SHALL run on
the server; the raw font bytes SHALL NOT leave the server to produce a preview.

The sample sequence, PCM shaping, and encoding SHALL be host-testable pure
helpers (the device-free synthesizer call and object I/O may be coverage-excluded
glue), so rendering the same font with the same sequence is deterministic.

#### Scenario: Rendering produces a deterministic clip
- **WHEN** a font's bytes are rendered with the fixed sample sequence
- **THEN** a non-empty audio clip is produced
- **AND** rendering the same font with the same sequence again yields an equivalent clip (same duration/format)

### Requirement: Preview generated at upload

When a SoundFont is uploaded (admin `POST /soundfonts/{id}`), the server SHALL, after
storing the font object, render a preview clip from the uploaded bytes and store it
as a **public** object under a preview key (e.g. `{id}.preview.wav`), distinct from
the private font bytes. A preview-render failure SHALL NOT fail the upload — it is
logged and the preview is left absent, to be recovered via the back-office
regenerate action.

#### Scenario: Upload renders and stores a public preview
- **WHEN** an admin uploads a valid SoundFont
- **THEN** the font is stored AND a public preview clip for that font is stored

#### Scenario: Render failure does not fail the upload
- **WHEN** an admin uploads a font whose preview render fails
- **THEN** the upload still succeeds and the font is stored
- **AND** no preview object is stored (recoverable via regenerate)

### Requirement: Back-office preview regeneration

The server SHALL expose an **admin-gated** endpoint (e.g. `POST
/soundfonts/{id}/preview`) that re-reads the stored font bytes, renders the preview,
and overwrites the public preview object, returning success or failure. The
back-office SoundFonts admin screen SHALL provide a **"Generate sample"** action
that calls this endpoint through the injectable client seam and reflects its
in-flight / success / error state.

#### Scenario: Admin regenerates a missing preview
- **WHEN** an admin invokes "Generate sample" for a font that has no preview (e.g. seeded before this change)
- **THEN** the server renders and stores the preview and the action reports success

#### Scenario: Non-admin cannot regenerate
- **WHEN** a non-admin caller invokes the regenerate endpoint
- **THEN** the request is refused

### Requirement: A preview is mandatory to accept a font

The server SHALL refuse to accept a SoundFont (the `accepted` transition of
`SetSoundFontModerationStatus`) unless a preview object already exists for that font,
since accepting publishes it as publicly auditionable. The moderator generates the
preview first (the back-office "Generate sample" action) and auditions it, then
accepts. Only the `accepted` transition is gated; `pending`/`rejected` transitions need
no preview, and an unknown font id still resolves to not-found.

#### Scenario: Accepting without a preview is refused
- **WHEN** a moderator accepts a font that has no preview object
- **THEN** the request is refused (a failed-precondition), the font stays unaccepted, and the back office shows a hint to generate a sample first

#### Scenario: Accepting after generating a preview succeeds
- **WHEN** a moderator generates a font's preview and then accepts it
- **THEN** the font becomes accepted

#### Scenario: Rejecting needs no preview
- **WHEN** a moderator rejects a font that has no preview object
- **THEN** the rejection succeeds

### Requirement: Public preview delivery and in-app audition

The server SHALL serve a font's preview clip at `GET /soundfonts/{id}/preview`
**without** an entitlement gate (hearing the preview is the purpose), applying only
the moderation-visibility gate (a pending/rejected font's preview is not exposed to
non-moderators). If no preview object exists, the endpoint SHALL respond not-found.

The app's SoundFont catalog **play** button SHALL audition a font by fetching and
playing this preview clip, rather than downloading the font to synthesize locally,
so a **locked** font is auditionable without ever downloading its bytes.

#### Scenario: Anyone entitled to see the font can hear its preview
- **WHEN** any caller who may view an accepted font requests its preview
- **THEN** the server serves the preview clip regardless of entitlement (free/locked alike)

#### Scenario: Locked font is auditionable without download
- **WHEN** a user taps play on a locked (costed, un-redeemed) font in the catalog
- **THEN** the app plays the server-rendered preview clip
- **AND** the raw font bytes are never downloaded to audition it

#### Scenario: Absent preview degrades gracefully
- **WHEN** a font has no preview object yet
- **THEN** the preview endpoint responds not-found and the app disables/greys the play control rather than erroring

