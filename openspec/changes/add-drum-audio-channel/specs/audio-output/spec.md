## RENAMED Requirements

- FROM: `### Requirement: SoundFont Piano Synthesis`
  TO: `### Requirement: SoundFont Instrument Synthesis`

## MODIFIED Requirements

### Requirement: SoundFont Instrument Synthesis

The Rust engine SHALL provide a polyphonic SoundFont synthesizer that renders
audio from a SoundFont (`.sf2`) to the system's default audio output device,
sounding **two instrument families**: melodic notes on the melodic channel, and
percussion notes on the reserved drum channel where preset lookup resolves in
the font's bank 128 (see `music-drum-audio` for the channel discipline). The
synthesizer SHALL support multiple simultaneous voices (chords and overlapping
notes) and SHALL expose a minimal control surface — initialize with a SoundFont,
**load a different SoundFont at runtime**, note-on (pitch, velocity), note-off
(pitch), percussion note-on (General MIDI number, velocity), percussion
note-off (General MIDI number), and all-notes-off — through the
flutter_rust_bridge FFI. The melodic entry points SHALL behave byte-for-byte as
before the percussion entry points existed. Loading a different SoundFont SHALL
replace the active instrument **without tearing down the audio output stream**,
SHALL issue an all-notes-off across the swap so no voice is left hanging on
either channel, and SHALL expose a completion the caller can await so a caller
knows when the incoming font is actually sounding. The native audio output and
synthesis thread SHALL be kept behind that seam so the rest of the app does not
depend on them directly.

#### Scenario: Note sounds a piano voice

- **WHEN** a note-on for a pitch is sent to the synthesizer
- **THEN** a piano voice for that pitch begins sounding from the SoundFont

#### Scenario: Percussion note sounds a kit voice

- **WHEN** a percussion note-on for a General MIDI number is sent while a
  percussion-family font is loaded
- **THEN** that kit voice begins sounding on the drum channel

#### Scenario: Polyphony

- **WHEN** several note-ons are sent before any note-off
- **THEN** all their voices sound together (a chord)

#### Scenario: Note-off releases the voice

- **WHEN** a note-off for a sounding pitch is sent
- **THEN** that voice enters its release and stops, leaving other voices sounding

#### Scenario: Percussion note-off releases only the kit voice

- **WHEN** a percussion note-off is sent for a sounding General MIDI number
- **THEN** that drum-channel voice is released and melodic voices are untouched

#### Scenario: Runtime SoundFont swap

- **WHEN** a different SoundFont is loaded while the engine is running
- **THEN** subsequent note-ons sound with the new SoundFont and the audio stream
  keeps running (no device re-acquisition)

#### Scenario: Swap silences hanging voices

- **WHEN** a note is held and the SoundFont is swapped
- **THEN** an all-notes-off is applied across the swap so the held voice does not
  hang

#### Scenario: Swap completion is observable

- **WHEN** a caller loads a SoundFont and awaits the exposed completion
- **THEN** the completion resolves once the new font is installed and sounding
  (or the swap has failed and the previous font was kept)

### Requirement: Live Note Sounding From Any Input

Every player note-on SHALL sound a piano note and every player note-off SHALL
release it **while a keyboard-family font is active** — which is every keyboard
score and free play outside a percussion score, under the font-follows-score
rule of `music-drum-audio` — regardless of the input source: the on-screen
keyboard, the computer keyboard, or a MIDI device, because all sources converge
on the player's note-on/note-off entry points. The exception is unchanged: when
the instrument-sounds-itself setting is enabled, notes whose source is the
connected MIDI instrument SHALL NOT be synthesized while notes from the
on-screen keyboard and the computer keyboard SHALL still sound. That exception
SHALL affect sounding only: scoring, key feedback and Wait Mode gating SHALL be
identical for every source in either case. Sounding SHALL work both while
playback is running and while it is stopped, so the user can play freely at any
time.

While a **percussion** score is open, the active font is a kit and a melodic
pitch has no preset to resolve to, so this requirement's piano-note promise is
scoped to keyboard-family fonts; live sounding for a percussion score — strokes
through the percussion one-shot entry points this change ships — is delegated
to `add-drum-input-mapping`, which sounds strokes as kit voices.

#### Scenario: On-screen tap sounds
- **WHEN** the user presses a key on the on-screen keyboard while a
  keyboard-family font is active
- **THEN** the corresponding piano note sounds, and releasing it stops the note

#### Scenario: MIDI and computer keyboard sound
- **WHEN** a note arrives from a MIDI device or the computer-keyboard fallback on
  a keyboard score and the instrument-sounds-itself setting is off
- **THEN** the same piano note sounds through the synthesizer

#### Scenario: Playable while stopped
- **WHEN** playback is stopped on a keyboard score and the user presses a key
- **THEN** the note still sounds (audio does not require the playhead to advance)

#### Scenario: Instrument note is not synthesized when the instrument sounds itself
- **WHEN** the instrument-sounds-itself setting is on and a note arrives from the
  connected MIDI instrument on a keyboard score
- **THEN** the app does not synthesize it, while still scoring it and showing its
  key feedback

#### Scenario: On-screen keyboard sounds even when the instrument sounds itself
- **WHEN** the instrument-sounds-itself setting is on and the user presses a key on
  the on-screen keyboard or the computer keyboard on a keyboard score
- **THEN** the corresponding piano note sounds through the synthesizer

### Requirement: Score Audio Playback

During playback the app SHALL sound the score's notes as the playhead reaches
each note's onset and SHALL release each note at its end, so the piece plays
audibly. A keyboard score's notes sound as melodic pitches, exactly as before; a
**percussion** score's notes sound their General MIDI percussion numbers as kit
voices on the drum channel, routed per note through the percussion entry points
by the loaded score's family. This SHALL be independent of the
instrument-sounds-itself setting, which governs only notes originating from an
input source. Timing SHALL follow the playhead, honoring the speed multiplier
and the derived tempo. While Wait Mode is frozen at an onset, the
not-yet-played notes SHALL NOT pre-sound; they sound when the playhead actually
advances past their onset. Stopping, restarting, or seeking playback SHALL issue
all-notes-off so no voice is left hanging on either channel.

#### Scenario: Notes sound as the playhead reaches them

- **WHEN** playback advances across a note's onset
- **THEN** that note sounds, and it is released when the playhead passes its end

#### Scenario: A percussion score's playback sounds the kit

- **WHEN** playback of a percussion score advances across an unpitched note's
  onset with the kit font installed
- **THEN** that General MIDI number sounds as a kit voice on the drum channel,
  never as a melodic pitch

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
