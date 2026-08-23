## ADDED Requirements

### Requirement: A percussion score is declared unpreviewable until it can be drawn

The browser notation renderer SHALL detect that a score is percussion and present an
explicit "not previewable yet" state instead of drawing it, for as long as the
renderer lacks percussion support (percussion clef, alternative noteheads,
two-voice layout on a single staff).

This matters because opening the admission gate makes drum scores reachable in the
console immediately — moderators are staff, so they are inside the drum audience by
construction — while the renderer still assumes pitched notation on a treble or
bass staff. Drawing a drum score with those assumptions produces a confident,
wrong-looking rendering that a moderator would reasonably read as a corrupt file
and reject. An explicit refusal is honest; a plausible wrong drawing is not.

The state SHALL be distinguishable from the existing failure states (undecodable,
unparseable, oversized), because the cause and the remedy differ: the file is fine,
the renderer is not ready.

#### Scenario: Percussion score shows the unpreviewable state

- **WHEN** a moderator opens a percussion score in the console
- **THEN** the preview shows an explicit "not previewable yet" state rather than a
  rendering

#### Scenario: The state is not a parse failure

- **WHEN** that state is shown
- **THEN** it is distinguishable from an undecodable or unparseable file, so the
  score is not mistaken for corrupt

#### Scenario: Keyboard scores render unchanged

- **WHEN** a moderator opens a keyboard score
- **THEN** it renders exactly as before
