## MODIFIED Requirements

### Requirement: Score audio preview rendered by a job at acceptance

The server SHALL enqueue a preview-render job when a catalog score is **accepted**
(the moderation transition that makes it playable), **in the same transaction** as
the status change (the job exists iff the acceptance commits). The job SHALL render
a short **audio-only** clip by taking the piece's playback schedule from the shared
MusicXML crate (the same note timing the app uses), **starting at the first sounding
note** (leading rests / an empty pickup are skipped) and clipping it to a bounded
duration (a runtime-configurable maximum, held notes truncated at the boundary),
synthesizing it with a **configured catalog SoundFont chosen by the piece's
instrument family** through the backend headless render engine — a keyboard piece
with the configured keyboard preview font on the melodic channel, a percussion
piece with the configured **kit** preview font on the **drum channel**
(`music-drum-audio`; the kit font is a second configuration key beside the
existing one) — and storing the clip as an object beside the score bytes under a
score-preview key distinct from the MusicXML. On success it SHALL stamp a
**rendered marker** on the catalog row (the listing truth for "has a sample"). The
schedule→sequence conversion SHALL be a pure, deterministic, host-testable helper.
A render failure SHALL NOT block or revert the acceptance — the job retries per
policy, then leaves the preview absent, recoverable via regenerate/backfill. An
unset or unknown preview font SHALL leave that **family's** previews absent
(feature dormant), never fail the acceptance and never touch the other family: in
particular, while the kit font is unconfigured a percussion piece's preview is
simply absent and its row unmarked — so the existing backfill over unmarked
accepted pieces is the catch-up once a kit font is configured — and a percussion
piece is never rendered with a keyboard-family font.

#### Scenario: Acceptance enqueues and the job stores a preview

- **WHEN** a catalog score is accepted
- **THEN** a render job is enqueued with the acceptance, and once it runs an audio-only preview clip for that piece is stored and the row is marked as rendered

#### Scenario: A percussion piece renders with the kit font

- **WHEN** the render job runs for an accepted percussion piece while the kit
  preview font is configured and accepted
- **THEN** the clip is synthesized with the kit font on the drum channel and the
  row is marked as rendered

#### Scenario: An unconfigured kit font leaves percussion dormant

- **WHEN** the render job runs for a percussion piece while the kit preview font
  is unset, unknown, not accepted, or not percussion-family
- **THEN** no clip is stored and the row stays unmarked, while keyboard pieces
  keep rendering with the keyboard preview font as before

#### Scenario: The backfill catches up formerly skipped percussion pieces

- **WHEN** the kit preview font is configured and the backfill over accepted
  pieces without a rendered marker is run
- **THEN** the percussion pieces accepted while previews were skipped or dormant
  are enqueued and rendered

#### Scenario: Acceptance rollback leaves no job

- **WHEN** the acceptance transaction fails
- **THEN** no preview-render job exists for that piece

#### Scenario: Render failure does not block acceptance

- **WHEN** the preview render fails for an accepted piece after its retries
- **THEN** the piece stays accepted and no preview object / rendered marker is stored (recoverable)

#### Scenario: Sequence is bounded and deterministic

- **WHEN** the same schedule is converted twice with the same maximum duration
- **THEN** both sequences are identical, end at or before the maximum, and contain no note starting after it

#### Scenario: The clip starts on the first note

- **WHEN** a piece opens with rests (or a silent pickup bar) before its first note
- **THEN** the teaser's first sounding note is at its very start and the bounded window is spent on music, not silence
