## MODIFIED Requirements

### Requirement: SoundFont Piano Synthesis

The Rust engine SHALL provide a polyphonic piano synthesizer that renders audio
from a bundled SoundFont (`.sf2`) to a **selectable** audio output device,
defaulting to the system's default output device when no selection has been made.
The synthesizer SHALL support multiple simultaneous voices (chords and overlapping
notes) and SHALL expose a minimal control surface — initialize with a SoundFont,
note-on (pitch, velocity), note-off (pitch), all-notes-off, and selecting or
reporting the output device — through the flutter_rust_bridge FFI. Changing the
output device SHALL rebuild the audio stream without requiring the SoundFont to be
reloaded and SHALL silence sounding voices first so none is stranded on the
previous device. The native audio output and synthesis thread SHALL be kept behind
that seam so the rest of the app does not depend on them directly.

#### Scenario: Note sounds a piano voice
- **WHEN** a note-on for a pitch is sent to the synthesizer
- **THEN** a piano voice for that pitch begins sounding from the SoundFont

#### Scenario: Polyphony
- **WHEN** several note-ons are sent before any note-off
- **THEN** all their voices sound together (a chord)

#### Scenario: Note-off releases the voice
- **WHEN** a note-off for a sounding pitch is sent
- **THEN** that voice enters its release and stops, leaving other voices sounding

#### Scenario: Default output when nothing is selected
- **WHEN** no output device has been selected
- **THEN** the engine renders to the system's default output device

#### Scenario: Output device change rebuilds the stream
- **WHEN** a different output device is selected
- **THEN** the audio stream is rebuilt on that device, sounding voices are silenced
  first, and the SoundFont is not reloaded

### Requirement: Live Note Sounding From Any Input

Every player note-on SHALL sound a piano note and every player note-off SHALL
release it, regardless of the input source — the on-screen keyboard, the computer
keyboard, or a MIDI device — because all sources converge on the player's
note-on/note-off entry points, **except** when the instrument-sounds-itself
setting is enabled, in which case notes whose source is the connected MIDI
instrument SHALL NOT be synthesized while notes from the on-screen keyboard and
the computer keyboard SHALL still sound. That exception SHALL affect sounding
only: scoring, key feedback and Wait Mode gating SHALL be identical for every
source in either case. Sounding SHALL work both while playback is running and
while it is stopped, so the user can play freely at any time.

#### Scenario: On-screen tap sounds
- **WHEN** the user presses a key on the on-screen keyboard
- **THEN** the corresponding piano note sounds, and releasing it stops the note

#### Scenario: MIDI and computer keyboard sound
- **WHEN** a note arrives from a MIDI device or the computer-keyboard fallback and
  the instrument-sounds-itself setting is off
- **THEN** the same piano note sounds through the synthesizer

#### Scenario: Playable while stopped
- **WHEN** playback is stopped and the user presses a key
- **THEN** the note still sounds (audio does not require the playhead to advance)

#### Scenario: Instrument note is not synthesized when the instrument sounds itself
- **WHEN** the instrument-sounds-itself setting is on and a note arrives from the
  connected MIDI instrument
- **THEN** the app does not synthesize it, while still scoring it and showing its
  key feedback

#### Scenario: On-screen keyboard sounds even when the instrument sounds itself
- **WHEN** the instrument-sounds-itself setting is on and the user presses a key on
  the on-screen keyboard or the computer keyboard
- **THEN** the corresponding piano note sounds through the synthesizer

### Requirement: Score Audio Playback

During playback the app SHALL sound the score's notes as the playhead reaches
each note's onset and SHALL release each note at its end, so the piece plays
audibly. This SHALL be independent of the instrument-sounds-itself setting, which
governs only notes originating from an input source. Timing SHALL follow the
playhead, honoring the speed multiplier and the derived tempo. While Wait Mode is
frozen at an onset, the not-yet-played notes SHALL NOT pre-sound; they sound when
the playhead actually advances past their onset. Stopping, restarting, or seeking
playback SHALL issue all-notes-off so no voice is left hanging.

#### Scenario: Notes sound as the playhead reaches them
- **WHEN** playback advances across a note's onset
- **THEN** that note sounds, and it is released when the playhead passes its end

#### Scenario: Speed affects audio timing
- **WHEN** the speed multiplier is changed during playback
- **THEN** the score's notes sound at the adjusted spacing

#### Scenario: Frozen Wait Mode does not pre-sound
- **WHEN** Wait Mode is frozen at an onset waiting for the user
- **THEN** the awaited note does not sound until the playhead advances past it

#### Scenario: Stop silences all voices
- **WHEN** playback is stopped or restarted
- **THEN** an all-notes-off is issued and no voice keeps sounding

#### Scenario: Score playback ignores the instrument-sounds-itself setting
- **WHEN** the instrument-sounds-itself setting is on and the score is played back
- **THEN** the score's notes sound through the app exactly as when the setting is
  off

### Requirement: Metronome Click Synthesis

The Rust engine SHALL provide a metronome click that is mixed into the audio
output **independently of the loaded piano SoundFont**, so a beat can be sounded
without using a piano voice and the click sounds the same regardless of which
`.sf2` is active. The click SHALL also be independent of the
instrument-sounds-itself setting, which governs only notes originating from an
input source. The engine SHALL expose, through the flutter_rust_bridge FFI, a
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

#### Scenario: Click ignores the instrument-sounds-itself setting
- **WHEN** the instrument-sounds-itself setting is on and the metronome is enabled
- **THEN** clicks sound through the app exactly as when the setting is off
