# music-score-audio-preview — delta for add-repeat-unrolling

## ADDED Requirements

### Requirement: Preview audio reflects the played timeline

The server-rendered score audio preview SHALL be synthesized from the
engine's unrolled playback order: repeated sections sound as many times as
the score prescribes, the correct volta is selected per pass, and `%`
measures replay the referenced measure's content — so the public clip matches
what the piece actually sounds like. Previews of repeat-carrying pieces
rendered before this change are stale; the existing back-office regeneration
and backfill tools SHALL be usable to re-render them (no automatic mass
re-render is required by this change).

#### Scenario: Preview of a repeated piece covers the unrolled timeline

- **WHEN** the preview job renders a piece whose opening section is repeated
- **THEN** the rendered clip's source timeline contains the section twice and
  the clip window is chosen over the unrolled duration

#### Scenario: Stale previews can be regenerated

- **WHEN** an admin regenerates the preview of a repeat-carrying piece that
  was rendered before repeat support
- **THEN** the new clip reflects the unrolled timeline and replaces the stale
  one
