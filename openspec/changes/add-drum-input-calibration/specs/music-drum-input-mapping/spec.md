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

### Requirement: The Pass Asks For The Loaded Score's Own Kit

The pass SHALL ask for the pieces the **loaded score actually writes**, in the
standard kit's order, rather than for a fixed kit: a groove is played on the
pieces it is written for, and asking for a whole standard kit makes a pass over a
hi-hat-and-snare groove mostly answers of "this kit has none". A piece the score
writes that the standard order does not name SHALL still be asked for, so the one
piece a player cannot calibrate is never the one the file requires. With no
percussion score to read a kit from, the pass SHALL fall back to the standard kit
so the surface still works on its own.

#### Scenario: A groove asks for its own pieces
- **WHEN** a score written for kick, snare and hi-hat is loaded and the pass is
  started
- **THEN** it asks for those three, in the kit's order, and ends there

#### Scenario: A piece outside the standard order is still offered
- **WHEN** the loaded score writes a piece the standard kit list does not name
- **THEN** the pass asks for it too

#### Scenario: No score, standard kit
- **WHEN** the pass is started with no percussion score loaded
- **THEN** it asks for the standard kit

### Requirement: The Pass Covers Every Trigger A Kit Sends

The pass SHALL ask for every part of a kit an instrument can trigger on a number
of its own, not only the pieces the app draws as lanes: the zones of a piece a
module fires separately — the snare's rim, the open hi-hat, the hi-hat pedal, the
ride bell — and the auxiliary pads a module offers beside the kit. A zone learned
this way SHALL still resolve to the piece it belongs to everywhere downstream, so
asking for it separately SHALL change nothing about what flashes, what the gate
awaits or what is scored.

Because the standard-kit fallback runs past most kits, the pass SHALL offer a way
to **end it early keeping what it has learned**, distinct from abandoning it, and
SHALL ask for the auxiliary pads only after the kit itself.

#### Scenario: A zone the module triggers separately is learned
- **WHEN** the pass asks for the open hi-hat and the kit sends 26 for it
- **THEN** 26 is recorded, and an incoming 26 is thereafter read as an open
  hi-hat stroke

#### Scenario: A learned zone is still its piece
- **WHEN** a stroke arrives on the number recorded for the snare's rim
- **THEN** it resolves to the snare — the same pad flashes, the same onsets are
  satisfied and the same scoring applies as for any other snare stroke

#### Scenario: A kit smaller than the list is stored without tapping through it
- **WHEN** the player ends the pass early after recording some pieces
- **THEN** exactly those pieces are stored, and the pass counts as completed
  rather than abandoned

#### Scenario: Ending early with nothing learned is not offered
- **WHEN** no step has recorded anything yet
- **THEN** ending early is not offered, because it would be abandoning under
  another name

### Requirement: The Pass Is Offered Where The Mapping Applies

The settings SHALL offer **one** MIDI-input entry point per score: the
calibration pass on a percussion score, the input monitor on any other. The
mapping states which piece of a **kit** a pad is, and the seam that applies it is
the identity on any other score, so offering the pass on a keyboard score would
promise a calibration that provably does nothing there; and offering the raw
read-out beside the pass reads as an alternative to it when only one of the two
repairs anything. The monitor SHALL remain reachable on a percussion score from
the calibration surface itself, where a player already is when the pass did not
settle the question.

#### Scenario: A keyboard score is not offered the pass
- **WHEN** the settings are opened on a keyboard score
- **THEN** the calibration pass is not offered, and the monitor is

#### Scenario: A percussion score is offered the pass
- **WHEN** the settings are opened on a percussion score
- **THEN** the calibration pass is offered, and the monitor is not listed beside
  it

#### Scenario: The monitor stays one level down
- **WHEN** the calibration surface is open
- **THEN** the monitor can be reached from it

### Requirement: The Settings Name What This Score Has Yet To Teach

The settings SHALL name, under the calibration action, the pieces the loaded
score asks for that the connected device has **no mapping entry for**, and SHALL
state plainly when there are none left. The question a player has before playing
is not whether their kit is calibrated in the abstract but whether everything
this groove will ask them to hit is understood, and that question SHALL be
answerable without starting a pass. Nothing SHALL be reported when no device is
connected, there being no mapping for a piece to be missing from.

#### Scenario: What is missing is named
- **WHEN** the loaded score asks for a piece the connected device has no entry
  for
- **THEN** that piece is named under the calibration action

#### Scenario: A learned piece leaves the list
- **WHEN** a piece named there is calibrated
- **THEN** it is no longer named

#### Scenario: Nothing left to teach is said plainly
- **WHEN** every piece the loaded score asks for has an entry for the connected
  device
- **THEN** the settings say so, rather than showing an empty list

### Requirement: The Table Shows The Whole Score's Kit, Missing Pieces Included

The calibration surface SHALL list every piece the loaded score asks for — the
ones this device has been read on **and** the ones it has not, each marked as
such — rather than only the entries already stored. A table of what is already
known answers a question nobody arrives with: a player opens this because
something did not respond, and the pieces with no entry are exactly the ones
they came for. Entries stored for this device that belong to no piece of the
loaded score SHALL be reported as a count, so clearing the device's calibration
never removes something the surface never showed.

The surface SHALL offer both a pass over **every** piece of the score and a pass
over **only the pieces with no entry**, the second only when it differs from the
first. A pass that covers part of a kit SHALL leave every piece it did not ask
about exactly as it was.

#### Scenario: A piece with no entry is listed and marked
- **WHEN** the loaded score asks for a piece the connected device has no entry
  for
- **THEN** it appears in the table, marked as not calibrated

#### Scenario: Only the missing pieces are asked for
- **WHEN** the player starts a pass over the missing pieces
- **THEN** the pass asks for those pieces only

#### Scenario: A partial pass keeps what it never asked about
- **WHEN** a pass covering part of the kit completes
- **THEN** the entries for every other piece — including pieces of other scores'
  kits — are still in force

#### Scenario: A number an untouched piece holds still collides
- **WHEN** a stroke during a partial pass arrives on a number already stored for
  a piece the pass is not asking about
- **THEN** the conflict is reported rather than one number being given to two
  pieces

#### Scenario: Entries outside this score are accounted for
- **WHEN** the device has entries for pieces the loaded score does not ask for
- **THEN** the surface reports how many, rather than listing or hiding them

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
