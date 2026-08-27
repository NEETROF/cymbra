## ADDED Requirements

### Requirement: One Translation Seam

An incoming MIDI note number SHALL be translated to the General MIDI number the
app reasons in at exactly one point, on the way in, before anything interprets
it. Everything downstream — what sounds, which surface flashes, whether the Wait
Mode gate opens, what the scorer credits — SHALL operate on the translated
number, so those four answers can never disagree about what was played.

#### Scenario: One stroke, one interpretation
- **WHEN** a mapping translates an incoming number to a kit piece's number
- **THEN** the sound, the flash, the gate and the scoring all resolve to that
  same piece

#### Scenario: Untranslated numbers pass through
- **WHEN** no mapping entry covers an incoming number
- **THEN** it is interpreted exactly as it arrives, as it was before any mapping
  existed

#### Scenario: The engine's own echo is translated too
- **WHEN** the engine is sounding live strokes from its MIDI callback and a
  mapping is in force
- **THEN** the echo sounds the translated number, not the raw one

### Requirement: Guided Calibration Pass

The app SHALL offer a calibration pass that learns a device's numbers by asking
the player to strike each piece in turn. Each step SHALL name the piece being
asked for, SHALL record the number of the stroke it receives, and SHALL be
skippable — a kit that has no such piece SHALL be able to move on without
inventing one. The pass SHALL be re-runnable at any time, and re-running it SHALL
replace the previous result only when it completes.

#### Scenario: A struck pad is learned
- **WHEN** the pass asks for the snare and a stroke arrives on number 31
- **THEN** 31 is recorded as this device's snare

#### Scenario: A piece the kit does not have is skipped
- **WHEN** the player skips a step
- **THEN** no entry is recorded for that piece and the pass continues

#### Scenario: Abandoning changes nothing
- **WHEN** the player leaves the pass before it completes
- **THEN** the previously stored mapping is left exactly as it was

#### Scenario: Two pieces cannot claim one number
- **WHEN** a stroke arrives on a number already recorded for an earlier piece
- **THEN** the conflict is reported and the player is asked to strike again or
  reassign, rather than silently overwriting

### Requirement: Mapping Is Per Device

A learned mapping SHALL be stored against the MIDI device it was learned from,
identified by its port name, and SHALL be applied only while that device is the
connected one. A device with no stored mapping SHALL behave exactly as an
uncalibrated app does: every number interpreted as it arrives.

#### Scenario: The right kit's mapping is applied
- **WHEN** a calibrated device is connected
- **THEN** its own mapping is applied and no other device's is

#### Scenario: An unknown device is untouched
- **WHEN** a device with no stored mapping is connected
- **THEN** incoming numbers are interpreted unchanged

#### Scenario: A mapping survives a restart
- **WHEN** a device is calibrated and the app is relaunched with it connected
- **THEN** the same mapping is in force without recalibrating

### Requirement: Mapping Is Reviewable and Editable

The stored mapping SHALL be viewable as a table of piece to number, and each
entry SHALL be individually editable and clearable without re-running the whole
pass. Clearing every entry SHALL return the device to uncalibrated behaviour.

#### Scenario: A single entry is corrected
- **WHEN** the player edits one piece's number
- **THEN** only that entry changes and the rest of the mapping stands

#### Scenario: The mapping is cleared
- **WHEN** the player clears the device's mapping
- **THEN** incoming numbers are interpreted unchanged from that point on

### Requirement: Calibration Never Silences Input

While the calibration pass is running, strokes SHALL still be audible, so the
player can hear that the instrument is reaching the app at all. A stroke recorded
by a step SHALL be the one the player just played, not a stale one from before
the step began.

#### Scenario: Strokes are audible during calibration
- **WHEN** the player strikes a pad during a calibration step
- **THEN** it sounds

#### Scenario: A stale stroke is not consumed
- **WHEN** a step begins after strokes have already been played
- **THEN** it waits for a fresh stroke rather than recording an earlier one
