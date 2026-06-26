# audio-output Specification

## Purpose
TBD - created by archiving change piano-sound-output. Update Purpose after archive.
## Requirements
### Requirement: SoundFont Piano Synthesis

The Rust engine SHALL provide a polyphonic piano synthesizer that renders audio
from a bundled SoundFont (`.sf2`) to the system's default audio output device.
The synthesizer SHALL support multiple simultaneous voices (chords and overlapping
notes) and SHALL expose a minimal control surface — initialize with a SoundFont,
note-on (pitch, velocity), note-off (pitch), and all-notes-off — through the
flutter_rust_bridge FFI. The native audio output and synthesis thread SHALL be
kept behind that seam so the rest of the app does not depend on them directly.

#### Scenario: Note sounds a piano voice
- **WHEN** a note-on for a pitch is sent to the synthesizer
- **THEN** a piano voice for that pitch begins sounding from the SoundFont

#### Scenario: Polyphony
- **WHEN** several note-ons are sent before any note-off
- **THEN** all their voices sound together (a chord)

#### Scenario: Note-off releases the voice
- **WHEN** a note-off for a sounding pitch is sent
- **THEN** that voice enters its release and stops, leaving other voices sounding

### Requirement: Live Note Sounding From Any Input

Every player note-on SHALL sound a piano note and every player note-off SHALL
release it, regardless of the input source — the on-screen keyboard, the computer
keyboard, or a MIDI device — because all sources converge on the player's
note-on/note-off entry points. Sounding SHALL work both while playback is running
and while it is stopped, so the user can play freely at any time.

#### Scenario: On-screen tap sounds
- **WHEN** the user presses a key on the on-screen keyboard
- **THEN** the corresponding piano note sounds, and releasing it stops the note

#### Scenario: MIDI and computer keyboard sound
- **WHEN** a note arrives from a MIDI device or the computer-keyboard fallback
- **THEN** the same piano note sounds through the synthesizer

#### Scenario: Playable while stopped
- **WHEN** playback is stopped and the user presses a key
- **THEN** the note still sounds (audio does not require the playhead to advance)

### Requirement: Score Audio Playback

During playback the app SHALL sound the score's notes as the playhead reaches
each note's onset and SHALL release each note at its end, so the piece plays
audibly. Timing SHALL follow the playhead, honoring the speed multiplier and the
derived tempo. While Wait Mode is frozen at an onset, the not-yet-played notes
SHALL NOT pre-sound; they sound when the playhead actually advances past their
onset. Stopping, restarting, or seeking playback SHALL issue all-notes-off so no
voice is left hanging.

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

### Requirement: Injectable Audio Seam And Graceful Degradation

The audio engine SHALL be exposed to Flutter through an injectable provider
(`audioServiceProvider`) so tests can replace it with a fake via a provider
override rather than constructor injection, keeping state and widgets testable
without the native library. If audio initialization or SoundFont loading fails,
the app SHALL continue running silently — no sound, no crash — and the rest of the
player (visuals, feedback, Wait Mode) SHALL remain fully functional.

#### Scenario: Tests inject a fake audio service
- **WHEN** a test overrides `audioServiceProvider` with a fake
- **THEN** the player drives the fake (recording note-on/note-off) without loading
  the native audio library

#### Scenario: Audio init failure degrades gracefully
- **WHEN** the audio device or SoundFont cannot be initialized
- **THEN** the app keeps working with no sound and does not crash

#### Scenario: Visuals unaffected by audio
- **WHEN** audio is unavailable
- **THEN** the render modes, key feedback, and Wait Mode behave exactly as before

