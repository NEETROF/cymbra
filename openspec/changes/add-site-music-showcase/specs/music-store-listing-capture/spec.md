# Spec Delta

## ADDED Requirements

### Requirement: The harness records a showcase video from the real app
The capture harness SHALL record a short showcase video of the real app through a single documented command per locale, from the same deterministic, no-backend state as the screenshots: a connected instrument, a real bundled piece, and an earned, non-zero score. The recorded interface SHALL be in the requested locale. The command SHALL produce a video of 30 seconds or less and a poster image taken from the same run. The video's sound SHALL be the notes played during the run, rendered by the app's own sound engine with the SoundFont the run used, aligned with the picture to within 50 ms; no other audio SHALL be mixed in. The command SHALL fail rather than write a file when the scripted run does not reach its expected state.

#### Scenario: French video
- **WHEN** the video command runs for the French locale
- **THEN** it writes a video of 30 seconds or less with the app's piano sound, and its poster, both showing the app in French playing a piece with a connected keyboard

#### Scenario: Sound in time with the picture
- **WHEN** a note lights up on screen in the recorded video
- **THEN** its sound starts within 50 ms of that frame

#### Scenario: Seeded state missing
- **WHEN** the scripted run does not reach the player with a connected instrument
- **THEN** the command fails and leaves the previous video in place
