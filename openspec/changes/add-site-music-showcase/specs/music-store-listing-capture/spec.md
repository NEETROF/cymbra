# Spec Delta

## ADDED Requirements

### Requirement: The harness records a showcase video from the real app
The capture harness SHALL record a short showcase video of the real app through a single documented command per locale, from the same deterministic, no-backend state as the screenshots: a connected instrument, a real bundled piece, and an earned, non-zero score. The recorded interface SHALL be in the requested locale. The command SHALL produce a muted video of 30 seconds or less and a poster image taken from the same run, and SHALL fail rather than write a file when the scripted run does not reach its expected state.

#### Scenario: French video
- **WHEN** the video command runs for the French locale
- **THEN** it writes a muted video of 30 seconds or less and its poster, both showing the app in French playing a piece with a connected keyboard

#### Scenario: Seeded state missing
- **WHEN** the scripted run does not reach the player with a connected instrument
- **THEN** the command fails and leaves the previous video in place
