## MODIFIED Requirements

### Requirement: SoundFont Piano Synthesis

The Rust engine SHALL provide a polyphonic piano synthesizer that renders audio
from a SoundFont (`.sf2`) to the system's default audio output device. The
synthesizer SHALL support multiple simultaneous voices (chords and overlapping
notes) and SHALL expose a minimal control surface — initialize with a SoundFont,
**load a different SoundFont at runtime**, note-on (pitch, velocity), note-off
(pitch), and all-notes-off — through the flutter_rust_bridge FFI. Loading a
different SoundFont SHALL replace the active instrument **without tearing down the
audio output stream**, and SHALL issue an all-notes-off across the swap so no voice
is left hanging. The native audio output and synthesis thread SHALL be kept behind
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

#### Scenario: Runtime SoundFont swap
- **WHEN** a different SoundFont is loaded while the engine is running
- **THEN** subsequent note-ons sound with the new SoundFont and the audio stream
  keeps running (no device re-acquisition)

#### Scenario: Swap silences hanging voices
- **WHEN** a note is held and the SoundFont is swapped
- **THEN** an all-notes-off is applied across the swap so the held voice does not
  hang
