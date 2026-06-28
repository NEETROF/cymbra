## ADDED Requirements

### Requirement: Metronome Click Synthesis

The Rust engine SHALL provide a metronome click that is mixed into the audio
output **independently of the loaded piano SoundFont**, so a beat can be sounded
without using a piano voice and the click sounds the same regardless of which
`.sf2` is active. The engine SHALL expose, through the flutter_rust_bridge FFI, a
single entry point to sound a click with an **accent** flag distinguishing the
downbeat from a normal beat (for example by pitch or level). The click SHALL be
short and self-terminating (it does not require a matching note-off) and SHALL be
kept behind the existing injectable audio seam. The pure click/mix logic SHALL be
host-testable (in `audio_core.rs`) so it is covered without the native audio
device.

#### Scenario: Click sounds without a piano voice
- **WHEN** a metronome click is requested
- **THEN** a short percussive tick is mixed into the output without sounding a
  piano note from the SoundFont

#### Scenario: Accent is distinct
- **WHEN** a click is requested with the accent flag set versus unset
- **THEN** the accented click is audibly distinct from a normal-beat click

#### Scenario: Independent of the active SoundFont
- **WHEN** the active piano SoundFont is changed
- **THEN** the metronome click is unchanged

#### Scenario: Click is self-terminating
- **WHEN** a click is sounded and no further calls are made
- **THEN** the click decays on its own and leaves no hanging voice
