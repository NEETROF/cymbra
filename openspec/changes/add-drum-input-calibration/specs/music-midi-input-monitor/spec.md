## ADDED Requirements

### Requirement: Live MIDI Input Monitor

The app SHALL offer a monitor that displays the MIDI note events it is currently
receiving, so a player can see what their instrument actually sends. Each entry
SHALL report the note number as received, the velocity, the transmitting
channel, and whether the event was a note-on or a note-off. The monitor SHALL
require no loaded score and SHALL be available whether or not a score is open.

#### Scenario: A stroke appears with its number
- **WHEN** a connected instrument sends a note-on while the monitor is open
- **THEN** an entry appears reporting that note number, its velocity and its
  channel

#### Scenario: Available with no score
- **WHEN** the monitor is opened with no score loaded
- **THEN** it opens and reports incoming events normally

#### Scenario: No device connected
- **WHEN** the monitor is opened with no MIDI device connected
- **THEN** it states that no device is connected rather than appearing broken or
  empty

### Requirement: Monitor Reports the App's Own Resolution

For every event shown, the monitor SHALL report how the app resolved it: the
General MIDI percussion name (or the note name, for a keyboard instrument) it was
understood as, and — when a score is loaded — whether it resolves to a piece of
that score's kit. An event that resolves to **no** piece SHALL be reported as
such and SHALL be visually distinguishable from one that resolves, because an
unresolved stroke is silent to the gate, the flash and the scorer alike.

#### Scenario: A resolved stroke names its piece
- **WHEN** a percussion score is loaded and a stroke arrives on a number that
  score's kit contains
- **THEN** the entry names the kit piece it resolved to

#### Scenario: An unresolved stroke says so
- **WHEN** a stroke arrives on a number that resolves to no piece of the loaded
  score's kit
- **THEN** the entry reports it as unmatched, distinguishably from a resolved one

#### Scenario: A number outside the audible range is called out
- **WHEN** a stroke arrives on a number the loaded instrument's SoundFont has no
  sample for
- **THEN** the entry reports that it will produce no sound

### Requirement: Monitor Does Not Disturb Playing

The monitor SHALL observe the input stream without changing it: opening it SHALL
NOT stop the app from sounding, gating or scoring incoming events, and closing it
SHALL leave the input path exactly as it was. Its history SHALL be bounded so a
long session cannot grow without limit.

#### Scenario: Playing continues while the monitor is open
- **WHEN** the monitor is opened during playback
- **THEN** strokes keep sounding, flashing and scoring exactly as before

#### Scenario: History is bounded
- **WHEN** more events arrive than the monitor retains
- **THEN** the oldest entries are dropped and the newest remain

### Requirement: Monitor Shows the Applied Mapping

When a device mapping is in force, the monitor SHALL show, for each translated
event, both the number as received and the General MIDI number it was translated
to. An event that was not translated SHALL show one number.

#### Scenario: A translated stroke shows both numbers
- **WHEN** a mapping translates an incoming number and the monitor is open
- **THEN** the entry reports the received number and the number it became

#### Scenario: An untranslated stroke shows one number
- **WHEN** no mapping applies to an incoming number
- **THEN** the entry reports that number alone, with no spurious translation
